use iced::{button, container, Background, Color, Vector};

const SURFACE: Color = Color::from_rgb(
    0xF2 as f32 / 255.0,
    0xF3 as f32 / 255.0,
    0xF5 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

pub enum Button {
    Start,
    Stop,
    Post,
    Primary,
    Destructive,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let (background, text_color) = match self {
            Button::Primary => (Some(ACTIVE), Color::WHITE),
            Button::Destructive => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
            _ => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
        };
        match self {
            Button::Start => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.1))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Stop => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.6))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Post => button::Style {
                background: Some(Background::Color(ACTIVE)), //Some(Background::Color(Color::from_rgb(0.1, 0.2, 0.8))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            Button::Primary => button::Style {
                text_color,
                background: background.map(Background::Color),
                border_radius: 5.0,
                shadow_offset: Vector::new(0.0, 0.0),
                ..button::Style::default()
            },
            Button::Destructive => button::Style {
                text_color,
                background: background.map(Background::Color),
                border_radius: 5.0,
                shadow_offset: Vector::new(0.0, 0.0),
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        let background = match self {
          Button::Primary => Some(HOVERED),
          _ => Some(Color {
              a: 0.1,
              ..active.text_color
          })
      };

        button::Style {
            text_color: match self {
                Button::Start => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            background: background.map(Background::Color),
            ..active
        }
    }
}
pub struct TitleBar {
    pub is_focused: bool,
}

impl container::StyleSheet for TitleBar {
    fn style(&self) -> container::Style {
        let pane = Pane {
            is_focused: self.is_focused,
        }
        .style();

        container::Style {
            text_color: Some(Color::WHITE),
            background: Some(pane.border_color.into()),
            ..Default::default()
        }
    }
}
pub struct Pane {
    pub is_focused: bool,
}

impl container::StyleSheet for Pane {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(SURFACE)),
            border_width: 2.0,
            border_color: if self.is_focused {
                Color::from_rgb(0.0, 0.8, 0.8)
            } else {
                Color::from_rgb(0.8, 0.9, 0.9)
            },
            ..Default::default()
        }
    }
}
