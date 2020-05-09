use iced_futures::futures;

// Just a little utility function
pub fn file<T: ToString>(url: T) -> iced::Subscription<Progress> {
    iced::Subscription::from_recipe(Download {
        url: url.to_string(),
    })
}

pub struct Download {
    url: String,
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for Download
where
    H: std::hash::Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.url.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            DownloadState::Ready(self.url),
            |state| async move {
                match state {
                    DownloadState::Ready(url) => {
                        let response = reqwest::get(&url).await;

                        match response {
                            Ok(response) => {
                                if let Some(total) = response.content_length() {
                                    Some((
                                        Progress::Started,
                                        DownloadState::Downloading {
                                            response,
                                            total,
                                            downloaded: 0,
                                        },
                                    ))
                                } else {
                                    Some((Progress::Errored, DownloadState::Finished))
                                }
                            }
                            Err(_) => {
                                Some((Progress::Errored, DownloadState::Finished))
                            }
                        }
                    }
                    DownloadState::Downloading {
                        mut response,
                        total,
                        downloaded,
                    } => match response.chunk().await {
                        Ok(Some(chunk)) => {
                            let downloaded = downloaded + chunk.len() as u64;

                            let percentage =
                                (downloaded as f32 / total as f32) * 100.0;

                            Some((
                                Progress::Advanced(percentage),
                                DownloadState::Downloading {
                                    response,
                                    total,
                                    downloaded,
                                },
                            ))
                        }
                        Ok(None) => Some((Progress::Finished, DownloadState::Finished)),
                        Err(_) => Some((Progress::Errored, DownloadState::Finished)),
                    },
                    DownloadState::Finished => {
                        // We do not let the stream die, as it would start a
                        // new download repeatedly if the user is not careful
                        // in case of errors.
                        let _: () = iced::futures::future::pending().await;

                        None
                    }
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum DownloadState {
    Ready(String),
    Downloading {
        response: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}