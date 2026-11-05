//! RME-style startup mismatch dialog.
//!
//! TotalMix FX shows this when the unit's saved mixer state and the host
//! state differ, and offers the same two directions.  The UC exposes only
//! the clock/optical/switch registers and queued front-panel gain/volume
//! changes for comparison; see `docs/UC-PARAMETER-SYNC.md`.

use iced::widget::{button, column, container, opaque, row, scrollable, text, Space};
use iced::{Alignment, Background, Color, Element, Length};

use crate::app::{Message, TotalControl};
use crate::theme;

const CARD_W: f32 = 560.0;
/// More rows than this would make the dialog taller than the window on a
/// small display, so the list gets a fixed height and its own scrollbar.
const MAX_VISIBLE_MISMATCHES: usize = 6;
const MISMATCH_ROW_H: f32 = 16.0;

fn uses_scroll_list(items: usize) -> bool {
    items > MAX_VISIBLE_MISMATCHES
}

pub fn mismatch_modal(state: &TotalControl) -> Element<'_, Message> {
    let scale = state.ui_scale;
    let Some(mismatch) = &state.startup_mismatch else {
        return Space::new().into();
    };

    let mut differences = column![].spacing(theme::SPACE_TIGHT * scale);
    for item in &mismatch.items {
        differences = differences.push(
            text(format!("• {item}"))
                .color(theme::TEXT_PRIMARY)
                .size(theme::TEXT_XS * scale),
        );
    }
    let differences: Element<'_, Message> = if uses_scroll_list(mismatch.items.len()) {
        scrollable(differences)
            .direction(scrollable::Direction::Vertical(theme::thin_scrollbar()))
            .height(Length::Fixed(
                MAX_VISIBLE_MISMATCHES as f32 * MISMATCH_ROW_H * scale,
            ))
            .width(Length::Fill)
            .style(theme::scrollable)
            .into()
    } else {
        differences.into()
    };

    let ucx = state.device.is_ucx_model();
    let card = opaque(
        container(
            column![
                text("Mismatch detected")
                    .color(theme::TEXT_PRIMARY)
                    .size(theme::TEXT_XL * scale),
                text(if ucx {
                    "Input gain and output volume of the device and TotalControl are different."
                } else {
                    "Channel and FX settings of the device and TotalControl are different."
                })
                .color(theme::TEXT_SEC)
                .size(theme::TEXT_XS * scale),
                differences,
                text(if ucx {
                    "The UCX reports input gain and output volume. Choosing Use device \
                     settings restores those values on the unit and in TotalControl. The options \
                     word, reference levels and the 48 V/Inst/Level switches are not read \
                     back and stay at the TotalControl scene."
                } else {
                    "The UC stores its matrix, faders and loopback, but only the clock, \
                     optical format, 48 V, PAD and Inst registers can be read back. \
                     Choosing Device -> TotalControl applies the readable values; the matrix, \
                     faders and loopback stay at the TotalControl scene."
                })
                .color(theme::TEXT_SEC)
                .size(theme::TEXT_MICRO * scale),
                row![
                    button(
                        text("Use TotalControl settings (default)")
                            .size(theme::TEXT_XS * scale)
                    )
                    .padding([theme::SPACE_TIGHT * scale, theme::SPACE_MD * scale])
                    .style(theme::segment_button(true))
                    .on_press(Message::MismatchUseTotalcontrol),
                    button(text("Use device settings").size(theme::TEXT_XS * scale))
                        .padding([theme::SPACE_TIGHT * scale, theme::SPACE_MD * scale])
                        .style(theme::ghost_button)
                        .on_press(Message::MismatchUseDevice),
                ]
                .spacing(theme::SPACE_MD * scale)
                .align_y(Alignment::Center),
                button(
                    text("Always use TotalControl settings and do not ask again")
                        .color(theme::TEXT_SEC)
                        .size(theme::TEXT_MICRO * scale)
                )
                .padding([theme::SPACE_TIGHT * scale, theme::SPACE_SM * scale])
                .style(theme::plain_button)
                .on_press(Message::MismatchAlwaysTotalcontrol),
            ]
            .spacing(theme::SPACE_LG * scale)
            .width(Length::Fill),
        )
        .style(theme::dialog)
        .padding(theme::SPACE_XL * scale)
        .width(Length::Fill)
        .max_width(CARD_W * scale),
    );

    opaque(
        container(card)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(Color {
                    a: 0.65,
                    ..Color::BLACK
                })),
                ..container::Style::default()
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_differences_fit_without_a_scroll_list() {
        assert!(!uses_scroll_list(MAX_VISIBLE_MISMATCHES));
    }

    #[test]
    fn seven_differences_use_the_scroll_list() {
        assert!(uses_scroll_list(MAX_VISIBLE_MISMATCHES + 1));
    }
}
