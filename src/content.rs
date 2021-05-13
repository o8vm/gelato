use crate::{message::*, style};
use iced::{
    button, pane_grid, scrollable, Align, Button, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Text,
};

#[derive(Debug, Clone)]
pub struct Content {
    pub id: usize,
    pub channel_name: String,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
}

impl Content {
    pub fn new(id: usize, channel_list: &Vec<String>) -> Self {
        Content {
            id,
            channel_name: channel_list[id % channel_list.len()].clone(),
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
        }
    }
    /*pub fn insert (&mut self, state: Scrollable<Message>) -> () {
        self.scroll = state;
    }*/
    pub fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        text: String,
    ) -> Element<Message> {
        let Content {
            scroll,
            split_horizontally,
            split_vertically,
            close,
            ..
        } = self;

        let button = |state, label, message, style| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(15),
            )
            .padding(5)
            .on_press(message)
            .style(style)
        };

        let mut controls = Column::new()
            .spacing(5)
            .max_width(150)
            .align_items(Align::End)
            .push(button(
                split_horizontally,
                "H",
                Message::Split(pane_grid::Axis::Horizontal, pane),
                style::Button::Primary,
            ))
            .push(button(
                split_vertically,
                "V",
                Message::Split(pane_grid::Axis::Vertical, pane),
                style::Button::Primary,
            ));

        if total_panes > 1 {
            controls = controls.push(button(
                close,
                "X",
                Message::Close(pane),
                style::Button::Primary,
            ));
        }

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10)
            .push(Text::new(text.to_string()));

        Container::new(
            Row::new()
                .push(content)
                .push(Column::new().align_items(Align::End).push(controls)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()
    }
}
