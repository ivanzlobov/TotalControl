//! The About dialog: what the app is, which version of it is running, and
//! which device it is driving. Opened from the brand lockup in the top bar
//! (the mark + wordmark, see `app::top_bar`), it draws as a centered modal
//! above the whole window (see `about_modal`) rather than the docked drawer
//! it used to be: that drawer was pushed after the page content, so on the
//! Quick view it landed below the fold and on the Mixer view it came after
//! every strip row.

use iced::widget::{button, column, container, mouse_area, opaque, row, text, Space};
use iced::{Alignment, Background, Color, Element, Length};

use totalcontrol_core::RmeDevice;

use crate::app::{Message, TotalControl};
use crate::theme;

/// Where the source and the releases live: the same URL `totalcontrol.desktop`
/// carries as its homepage.
pub const REPO_URL: &str = "https://github.com/jspx-projects/TotalControl";

/// The card's width cap at `ui_scale == 1.0`: wide enough for the repo URL
/// on one line, narrow enough to read as a dialog, and capped rather than
/// fixed so a narrow window shrinks it instead of clipping it.
const CARD_W: f32 = 560.0;

fn header(scale: f32) -> Element<'static, Message> {
    row![
        text("About")
            .color(theme::text_primary())
            .size(theme::TEXT_DIALOG * scale),
        Space::new().width(Length::Fill),
        // Same title/Close pair as the device popover (`app::device_panel`).
        button(text("Close").size(theme::TEXT_DIALOG * scale))
            .padding([theme::SPACE_TIGHT * scale, theme::SPACE_MD * scale])
            .style(theme::ghost_button)
            .on_press(Message::ToggleAbout),
    ]
    .spacing(theme::SPACE_MD * scale)
    .align_y(Alignment::Center)
    .into()
}

/// Every gesture and key the app answers to, in one place: most of them
/// (double-clicks, Ctrl/Shift-clicks, the Matrix drags) are otherwise only
/// discoverable by accident or by a tooltip you have to hover first.
const SHORTCUTS: [(&str, &str); 12] = [
    ("Tab", "Next view (Quick, Mixer, Matrix)"),
    ("Ctrl+Z / Ctrl+Shift+Z", "Undo / redo"),
    ("Ctrl + / Ctrl - / Ctrl 0", "Zoom in / out / reset (also Ctrl+wheel)"),
    ("Ctrl ,", "TotalControl settings (tray, autostart)"),
    ("F11", "Full screen"),
    ("Esc", "Close a dialog, cancel an edit"),
    ("Ctrl / Shift + click", "Select strips; their faders move together"),
    ("Double-click a name", "Rename the strip"),
    ("Double-click a dB value", "Type a level"),
    ("Double-click a fader", "Reset it"),
    ("Double-click a strip", "Collapse / expand it"),
    ("Shift + drag", "Fine adjustment"),
];

fn shortcuts(scale: f32) -> Element<'static, Message> {
    let mut list = column![text("SHORTCUTS")
        .color(theme::text_sec())
        .size(theme::TEXT_DIALOG * scale)]
    .spacing(theme::SPACE_SM * scale);
    for (keys, action) in SHORTCUTS {
        list = list.push(
            row![
                text(keys)
                    .color(theme::text_primary())
                    .size(theme::TEXT_DIALOG * scale)
                    .width(Length::Fixed(150.0 * scale)),
                text(action)
                    .color(theme::text_sec())
                    .size(theme::TEXT_DIALOG * scale),
            ]
            .spacing(theme::SPACE_MD * scale),
        );
    }
    list.into()
}

/// The whole overlay: a dimming scrim that closes on click and blocks the
/// window behind it, with the card centered on it. The card is `opaque`,
/// so a click inside it (its padding included) is captured before the
/// scrim's `mouse_area` and never closes the dialog under the cursor.
pub fn about_modal(state: &TotalControl) -> Element<'_, Message> {
    let scale = state.ui_scale;

    // Which build, and which device it is driving right now, in which
    // state (the same four words the sidebar chip uses, disconnect
    // included). Between them they are most of what a bug report needs
    // to start with, and the sidebar chip that says the same thing
    // scrolls away with the mixer. The mock suffix goes: the word right
    // after it already says it, and "(mock) (simulated)" reads like a
    // bug of its own.
    let model = state
        .device
        .model_name()
        .strip_suffix(" (mock)")
        .unwrap_or_else(|| state.device.model_name());
    let device = format!(
        "Driving {model} ({})",
        if !state.connected {
            "disconnected"
        } else if state.device.is_mock() {
            "simulated"
        } else if state.device.class_compliant() {
            "class-compliant"
        } else {
            "proprietary"
        },
    );

    let identity = column![
        row![
            text("TotalControl")
                .color(theme::text_primary())
                .size(theme::TEXT_DIALOG * scale),
            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                .color(theme::text_sec())
                .size(theme::TEXT_DIALOG * scale),
        ]
        .spacing(theme::SPACE_LG)
        .align_y(Alignment::Center),
        text("Open-source TotalMix replacement for RME audio interfaces")
            .color(theme::text_sec())
            .size(theme::TEXT_DIALOG * scale),
        text(device)
            .color(theme::text_sec())
            .size(theme::TEXT_DIALOG * scale),
        button(
            text(REPO_URL)
                .color(theme::ACCENT)
                .size(theme::TEXT_DIALOG * scale)
        )
        .padding([theme::SPACE_TIGHT * scale, theme::SPACE_SM * scale])
        .style(theme::plain_button)
        .on_press(Message::OpenRepoUrl),
    ]
    .spacing(theme::SPACE_SM * scale);

    // The knob/T mark without its tile beside the identity block, decoded
    // by the crate's own `png` decoder and shown through iced's codec-free
    // image widget (see `icon::mark`).
    let identity_row: Element<'_, Message> = match crate::icon::mark() {
        Some((handle, ratio)) => {
            let mark_h = 44.0 * scale;
            row![
                iced::widget::image(handle)
                    .width(mark_h * ratio)
                    .height(mark_h)
                    .filter_method(iced::widget::image::FilterMethod::Linear),
                identity,
            ]
            .spacing(theme::SPACE_XL * scale)
            .align_y(Alignment::Center)
            .into()
        }
        None => identity.into(),
    };

    let card = opaque(
        container(
            column![header(scale), identity_row, shortcuts(scale)]
                .spacing(theme::SPACE_LG * scale)
                .width(Length::Fill),
        )
        .style(theme::dialog)
        .padding(theme::SPACE_XL * scale)
        .width(Length::Fill)
        .max_width(CARD_W * scale),
    );

    // The scrim is the invisible half of the modal: dim the window, catch
    // the click that closes the dialog, and keep every event from reaching
    // the window underneath. It wraps the card rather than sitting under
    // it in a `stack`: there, the full-window container on top made iced
    // levitate the cursor for the scrim below, so clicking outside the
    // card never closed anything. The card is `opaque`, so its own clicks
    // are captured before `mouse_area` sees them.
    opaque(
        mouse_area(
            container(card)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.55,
                        ..Color::BLACK
                    })),
                    ..container::Style::default()
                }),
        )
        .on_press(Message::ToggleAbout),
    )
}
