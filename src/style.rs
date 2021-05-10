use iced::{button, Background, Color, Vector};

pub enum Button {
    Start,
    Stop,
    Post,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
            Button::Start => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.1))),
                border_radius: 1.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Stop => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.6))),
                border_radius: 1.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Post => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.2, 0.8))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            text_color: match self {
                Button::Start => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }
}
