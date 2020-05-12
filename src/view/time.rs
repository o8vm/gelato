// https://github.com/hecrj/iced/tree/master/examples/stopwatch
// stopwatchを移植してみた

use iced::futures;


pub fn every(duration: std::time::Duration) -> iced::Subscription<std::time::Instant> {
  iced::Subscription::from_recipe(Every(duration))
}

struct Every(std::time::Duration);
// https://docs.rs/iced_native/0.2.1/iced_native/subscription/trait.Recipe.html
impl<H, I> iced_native::subscription::Recipe<H, I> for Every
where
  H: std::hash::Hasher,
{
  type Output = std::time::Instant;
  fn hash(&self, state: &mut H) {
    use std::hash::Hash;

    std::any::TypeId::of::<Self>().hash(state);
    self.0.hash(state);
  }

  fn stream(
    self: Box<Self>,
    _input: futures::stream::BoxStream<'static, I>,
  ) -> futures::stream::BoxStream<'static, Self::Output> {
    use futures::stream::{StreamExt};
    async_std::stream::interval(self.0)
      .map(|_| std::time::Instant::now())
      .boxed()
  }
}
