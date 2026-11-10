//! The pause mark on the Mute and Solo masters and their held channels.
//!
//! Two short, thick bars rather than a text glyph such as `||`: the text
//! pipes are full cap height and thin, which reads as "or" next to the
//! letter. These are half the label height and thicker, so they read as the
//! transport pause the masters use.

use iced::widget::{container, row, Space};
use iced::{Color, Element, Length};

/// A pause mark in `color` for a label drawn at `scale`.
pub fn pause_mark<'a, Message: 'a>(color: Color, scale: f32) -> Element<'a, Message> {
    let bar = move || {
        container(Space::new())
            .width(Length::Fixed((2.2 * scale).max(2.0)))
            .height(Length::Fixed((5.0 * scale).max(4.0)))
            .style(move |_theme| container::Style {
                background: Some(color.into()),
                ..container::Style::default()
            })
    };
    row![bar(), bar()]
        .spacing((1.6 * scale).max(1.0))
        .align_y(iced::Alignment::Center)
        .into()
}
