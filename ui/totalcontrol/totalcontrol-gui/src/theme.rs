//! Color palette shared across the whole GUI.

use iced::gradient::Linear;
use iced::widget::overlay::menu;
use iced::widget::slider::{self, Handle, HandleShape, Rail};
use iced::widget::{button, container, progress_bar};
use iced::{Background, Border, Color, Shadow};
use std::sync::OnceLock;
use totalcontrol_layout::ColorScheme;

// Identity: the app icon is a pure-black tile with a monochrome silver
// mark, so the UI is built from those same three ingredients and nothing
// else - graphite surfaces, silver text, white for anything that is
// "on" (engaged toggles, the selected strip, the active nav pill, the
// section ticks). Hue is information, not decoration: green = signal and
// connection, red = clip, amber = attention (solo, simulated/class
// compliant), azure = mute. Before this pass the palette was sampled 1:1
// from RME TotalMix FX (cool blue-gray cards, orange accent, cyan mute,
// see the reference-totalmix-ui-assets memory) and read as a different
// product than the icon on the taskbar.
struct Palette {
    bg_deep: Color,
    fader_slot: Color,
    button_face: Color,
    surface: Color,
    submix_active: Color,
    tab_face: Color,
    border: Color,
    text_primary: Color,
    text_sec: Color,
    knob_face: Color,
    knob_border: Color,
    mute: Color,
    solo: Color,
    cue: Color,
    green: Color,
    yellow: Color,
    red: Color,
}

static PALETTE: OnceLock<Palette> = OnceLock::new();

/// Install the colour scheme from the layout document.  Startup only: styles
/// read the parsed palette through the accessors below.
pub fn install_palette(scheme: ColorScheme) {
    let _ = PALETTE.set(Palette::from(&scheme));
}

fn palette() -> &'static Palette {
    PALETTE.get_or_init(|| Palette::from(&ColorScheme::default()))
}

impl From<&ColorScheme> for Palette {
    fn from(scheme: &ColorScheme) -> Self {
        Self {
            bg_deep: hex(&scheme.surface.app_background),
            fader_slot: hex(&scheme.control.fader_slot),
            button_face: hex(&scheme.control.button_face),
            surface: hex(&scheme.surface.card),
            submix_active: hex(&scheme.surface.card_active),
            tab_face: hex(&scheme.control.tab_face),
            border: hex(&scheme.surface.border),
            text_primary: hex(&scheme.text.primary),
            text_sec: hex(&scheme.text.secondary),
            knob_face: hex(&scheme.control.knob_face),
            knob_border: hex(&scheme.control.knob_border),
            mute: hex(&scheme.accent.mute),
            solo: hex(&scheme.accent.solo),
            cue: hex(&scheme.accent.cue),
            green: hex(&scheme.meter.green),
            yellow: hex(&scheme.meter.yellow),
            red: hex(&scheme.meter.red),
        }
    }
}

/// Parse a `#rrggbb` literal.  The loader validates the document, so the
/// fallback only covers a palette installed before validation.
fn hex(value: &str) -> Color {
    match u32::from_str_radix(value.trim_start_matches('#'), 16) {
        Ok(rgb) => Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
        Err(_) => Color::from_rgb8(0, 0, 0),
    }
}

pub fn fader_slot() -> Color {
    palette().fader_slot
}

pub fn button_face() -> Color {
    palette().button_face
}

pub fn surface() -> Color {
    palette().surface
}

pub fn submix_active() -> Color {
    palette().submix_active
}

pub fn tab_face() -> Color {
    palette().tab_face
}

pub fn border() -> Color {
    palette().border
}

pub fn text_primary() -> Color {
    palette().text_primary
}

pub fn text_sec() -> Color {
    palette().text_sec
}

pub fn knob_face() -> Color {
    palette().knob_face
}

pub fn knob_border() -> Color {
    palette().knob_border
}

pub fn mute_color() -> Color {
    palette().mute
}

pub fn solo_color() -> Color {
    palette().solo
}

pub fn cue_color() -> Color {
    palette().cue
}

/// Group marks.  Muted on purpose: four fader groups, the mute and solo
/// links and the temporary fader group share one panel and a handful of
/// caps, and they have to stay distinguishable without turning the mixer
/// into a colour chart.
pub fn group_fader(index: usize) -> Color {
    match index {
        0 => Color::from_rgb8(0xb5, 0x6d, 0x6d),
        1 => Color::from_rgb8(0x57, 0x84, 0xb8),
        2 => Color::from_rgb8(0x6f, 0x93, 0x6a),
        _ => Color::from_rgb8(0x8f, 0x74, 0xa3),
    }
}

pub fn group_mute() -> Color {
    Color::from_rgb8(0x6f, 0x86, 0x9e)
}

pub fn group_solo() -> Color {
    Color::from_rgb8(0xb0, 0x95, 0x4f)
}

/// The temporary fader group: RME marks its faders yellow, and it outranks
/// a stored group colour while it exists.
pub fn group_temporary() -> Color {
    Color::from_rgb8(0xc2, 0xae, 0x57)
}

pub fn mgreen() -> Color {
    palette().green
}

pub fn mred() -> Color {
    palette().red
}

pub fn ysim() -> Color {
    palette().yellow
}

/// Band identity in the EQ panel, matching the three columns and their
/// handles in the response plot.
pub fn eq_band() -> [Color; 3] {
    [mred(), mgreen(), mute_color()]
}

pub fn bg_deep() -> Color {
    palette().bg_deep
}

/// Hover surface for controls and strips.
pub const SURFACE_HI: Color = Color::from_rgb8(0x2d, 0x2e, 0x33);
/// VU well on a normal strip: one shade below `surface()`, not a black cut-out.
pub const METER_WELL: Color = Color::from_rgb8(0x1e, 0x1f, 0x23);
/// VU well on an active input/playback strip: one shade below
/// `submix_active()`.
pub const METER_WELL_ACTIVE: Color = Color::from_rgb8(0x33, 0x34, 0x3a);
/// Slideout-tab face on hover: lighter than the open face, still darker
/// than the channel surface.
pub const TAB_FACE_HOVER: Color = Color::from_rgb8(0x22, 0x23, 0x28);
/// VU scale labels: one step below the primary text tone.
pub const VU_LABEL: Color = Color::from_rgb8(0xd5, 0xd6, 0xda);
/// "Engaged": every lit toggle, the selected strip's border, the section
/// ticks, the active nav pill.
pub const ACCENT: Color = Color::from_rgb8(0xf2, 0xf3, 0xf5);
/// Quiet selected fill (menu highlight, text selection, the lit side of
/// a sidebar switch): a graphite step, not a hue.
pub const ACCENT_DIM: Color = Color::from_rgb8(0x37, 0x38, 0x3e);
/// Fader cap and rail silver, from the icon's highlight: the brightest
/// neutral in the chrome.
pub const FADER: Color = Color::from_rgb8(0xc9, 0xcb, 0xd1);

/// The filled part of a channel fader slot: the card blended a step toward
/// the cap, so it stays visible on both the normal and the active surface.
pub fn fader_fill(active_surface: bool) -> Color {
    let base = if active_surface {
        submix_active()
    } else {
        surface()
    };
    blend(base, FADER, 0.12)
}
/// 48V uses its own orange so phantom power is distinct from the
/// white "engaged" toggles around it.
pub const PHANTOM: Color = Color::from_rgb8(0xff, 0x9f, 0x43);
pub const GCONN: Color = Color::from_rgb8(0x3d, 0xd6, 0x8c);

/// Knob-rim colours: a shade under the button accents, so the arc reads as a
/// colour against a dark strip instead of washing out next to the value text.
pub const KNOB_BLUE: Color = Color::from_rgb8(0x2f, 0x7a, 0xc4);
pub const KNOB_GREEN: Color = Color::from_rgb8(0x2f, 0xa8, 0x6e);
/// Text on an engaged (white) toggle and on the active nav pill: the
/// near-black of the tile behind the mark.
pub const ON_ACTIVE: Color = Color::from_rgb8(0x10, 0x10, 0x12);

// ── Type scale ───────────────────────────────────────────────────
// Every text element in the app maps to one of these six tiers rather
// than a one-off pixel value chosen per call site — before this there
// were eight near-random sizes (6.5, 8, 9, 10, 11, 12, 13, 20) with no
// clear reason two of them (8 vs 9, 11 vs 12) were ever different.
//
// These are the sizes at `ui_scale == 1.0` — bumped up from the original
// pass (11px body text read as small for a desktop app) so the default,
// unscaled UI is already comfortable; `ui_scale` (see `SCALE_DEFAULT`)
// only moves off 1.0 when the window is too narrow/wide to fit the
// widest strip row at that baseline.

/// Canvas-drawn ruler tick labels on the fader — the smallest legible
/// size, only used where the fader canvas has no room for anything
/// bigger.
pub const TEXT_MICRO: f32 = 8.0;
/// Tertiary annotations: type tags, dB readouts, the pan L/R/C label,
/// matrix-view row/column labels.
pub const TEXT_XS: f32 = 10.0;
/// Buttons and compact controls: M/S, 48V/PAD, the collapse toggle,
/// the dB edit input.
pub const TEXT_SM: f32 = 11.0;
/// Default body text: channel names, section headers, top-bar labels,
/// pick lists, and the sidebar's device-chip model name (originally
/// sized a tier up for emphasis, but that read as too big for the
/// sidebar's dense 210px column — dropped back down to this tier).
pub const TEXT_MD: f32 = 13.0;
/// One size for menu items and dialog text, matching the Control Room
/// Settings dialog.
pub const TEXT_DIALOG: f32 = 16.0;
/// The "TotalControl" wordmark.
pub const TEXT_XL: f32 = 22.0;

/// Adaptive UI scale — recomputed on every window resize (see
/// `recompute_ui_scale`) so the widest strip row always fits the window
/// without horizontal scrolling, TotalMix-2.0-style; not a manual zoom
/// control. Multiplied into every text size and widget dimension in the
/// mixer/matrix views — see `TotalControl::ui_scale`. `SCALE_DEFAULT` is the
/// fallback when there's nothing to size against yet (no channels); `MIN`/
/// `MAX` bound how far the auto-fit math is allowed to shrink/grow it.
pub const SCALE_DEFAULT: f32 = 1.0;
/// Bounds for the *manual* UI zoom (Ctrl+molette / Ctrl+= / Ctrl+-). The
/// scale is no longer tied to window size at all — window resizing just
/// shows more or less via scroll, so these only constrain how far the user
/// can zoom by hand (a readability floor and a sane ceiling).
pub const SCALE_MIN: f32 = 0.75;
pub const SCALE_MAX: f32 = 1.75;

// ── Corner radius scale ─────────────────────────────────────────────
// Same idea as the type scale: before this, `panel` used 7 and `chip`
// used 6 with no reason they were ever different — just whatever felt
// fine in isolation when each was written.

/// The section-header accent tick - barely-there rounding on a thin bar.
pub const RADIUS_XS: f32 = 3.0;
/// Every interactive control: buttons, text inputs, pick lists/menus, the
/// scrollbar scroller.
pub const RADIUS_SM: f32 = 6.0;
/// Container chrome: strip cards (`panel`) and top-bar groups (`chip`).
pub const RADIUS_MD: f32 = 10.0;

// ── Spacing scale ────────────────────────────────────────────────────
// Same consolidation for `.spacing()`/`.padding()` call sites — before
// this there were eleven near-arbitrary values (1, 2, 3, 4, 5, 6, 8, 10,
// 12, 14, 16) scattered across app.rs/strip.rs/matrix.rs with no shared
// vocabulary. Values are the *smallest* unit of a given role, not a
// strict multiplier progression — the goal is a short, named list callers
// can pick from, not a formula.

/// Between rows stacked tightly inside one strip's control stack (header
/// → M/S → 48V/PAD → fader → dB → pan) and between a pan dot and its
/// label — these read as one continuous unit, not separate rows.
pub const SPACE_HAIRLINE: f32 = 1.0;
/// Standard layout gap between a knob box and its label.  The knob widget
/// adds its own transparent margin, so the visible gap is slightly larger.
pub const KNOB_LABEL_GAP: f32 = SPACE_HAIRLINE;
/// Between closely related inline siblings: a channel name and its type
/// tag, the M/S button pair, a matrix cell grid, the MIXER/MATRIX toggle.
pub const SPACE_TIGHT: f32 = 2.0;
/// Small control padding — e.g. the top-bar view-toggle buttons.
pub const SPACE_SM: f32 = 4.0;
/// Strip card padding, strip-to-strip gaps, a chip's internal row
/// spacing.
pub const SPACE_MD: f32 = 6.0;
/// Minimum gap between an inline slideout's absolute edge and its
/// contents.  Slightly wider than `SPACE_MD` so labels and focus rings
/// never touch the panel edge.
pub const FLYOUT_PAD: f32 = 8.0;
/// Section-level spacing: the gap between HARDWARE INPUTS / SOFTWARE
/// PLAYBACK / HARDWARE OUTPUTS blocks, a section header's own row.
pub const SPACE_LG: f32 = 8.0;
/// Container-level padding: a chip's horizontal padding, the mixer/matrix
/// view's outer padding.
pub const SPACE_XL: f32 = 12.0;
/// The top bar's outer padding and the gap between its identity/nav/
/// session groups.
pub const SPACE_XXL: f32 = 16.0;

/// The top bar's height at scale 1. The sidebar's title row is laid out
/// against it (`sidebar::title_row`), so the two read as one header line.
pub const TOP_BAR_H: f32 = 48.0;

/// A slideout card that visually merges with the active tab on its left.
/// No border: the tab background and the panel background are the same
/// surface, and the panel overlaps the strip's right padding by the tab's
/// width offset.  The outer surface still separates it from the window.
pub fn panel_attached(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(surface())),
        // Invisible border (same colour as the background) but the radius
        // still shapes the surface, giving the slideout the channel strips'
        // rounded top corners without drawing a line at the tab join.
        border: Border {
            color: surface(),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        text_color: Some(text_primary()),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn panel(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(surface())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        text_color: Some(text_primary()),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The inert dark face at the foot of a collapsed strip.  It uses the
/// normal button face and shape so future footer content has a defined
/// surface without adding a control today.
pub fn display_face(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(button_face())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        ..container::Style::default()
    }
}

/// A reserved but currently inactive tab slot.
pub fn empty_slot(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        ..container::Style::default()
    }
}

/// The FX column's header band: the panel surface lifted one step, so the
/// effect name and its on switch read as that panel's title bar. TotalMix
/// draws the same three-step relationship inside one FX panel — lightest
/// header, darker controls, mid-tone tail.
pub fn fx_header(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_HI)),
        text_color: Some(text_primary()),
        ..container::Style::default()
    }
}

/// The FX column's controls band, recessed a step below its header so the
/// knobs sit in a well rather than on the same surface as the title.
pub fn fx_controls(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(blend(surface(), bg_deep(), 0.45))),
        text_color: Some(text_primary()),
        ..container::Style::default()
    }
}

pub fn overload(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(mred())),
        ..container::Style::default()
    }
}

pub fn fx_meter(_theme: &iced::Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(bg_deep()),
        bar: Background::Color(mgreen()),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
    }
}

/// A floating dialog card: the same graphite surface as `panel`, plus a
/// drop shadow so it reads as raised above the window instead of drawn
/// into it. First user: the About modal (`about::about_modal`).
pub fn dialog(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(surface())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        text_color: Some(text_primary()),
        shadow: Shadow {
            color: Color {
                a: 0.55,
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 30.0,
        },
        snap: false,
    }
}

/// Linear-interpolates `base` toward `tint` by `amount` (0 = pure `base`,
/// 1 = pure `tint`), keeping `base`'s own alpha — used to tint a strip
/// card just enough to read as "this channel's category" at a glance
/// without competing with the accent-colored selection border or making
/// the card itself look like a colored badge.
pub(crate) fn blend(base: Color, tint: Color, amount: f32) -> Color {
    Color {
        r: base.r + (tint.r - base.r) * amount,
        g: base.g + (tint.g - base.g) * amount,
        b: base.b + (tint.b - base.b) * amount,
        a: base.a,
    }
}

/// How strongly a strip's `tint` (its channel type's own color — the same
/// one already used for its type-tag badge) is blended into the card
/// background. Subtle on purpose: enough to sort channel types apart at a
/// glance in a long scrolled row, not so much it reads as a colored
/// button the way the type tag itself does.
///
/// 2026-10-20: down from `0.10`. At that strength one row read as three
/// differently-colored products (brown INST, violet ADAT, teal PB); the
/// section headers already sort the strips, and the type tag still carries
/// its color, so the card only needs the faintest hint.
const STRIP_TINT_AMOUNT: f32 = 0.03;

/// A strip's card — same as `panel`, but with an accent-colored border
/// when part of the active multi-selection (Ctrl/Shift+click), so grouped
/// mute/solo/collapse actions have a clear "this is what I'm about to
/// affect" indicator — and, if `tint` is given, a background nudged
/// toward that channel type's own color (the same color as its type-tag
/// badge), so a long row of strips sorts into visual groups before you
/// even read the labels.
/// `hovered` is ignored whenever `selected` is true — a strip that's part
/// of the active multi-selection already has the loudest border in the
/// row; a hover glow on top of that would just be visual noise competing
/// with itself. Plain `container::Style` closures don't get a `Status` the
/// way `button`'s do, so this hover state has to be tracked explicitly
/// (`TotalControl::hovered_strip`, driven by `mouse_area::on_enter`/`on_exit` in
/// `widgets/strip.rs`) rather than read off the widget itself.
pub fn strip_panel(
    selected: bool,
    hovered: bool,
    tint: Option<Color>,
    active_surface: bool,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |theme| {
        let base = panel(theme);
        let base = match tint {
            Some(t) => container::Style {
                background: Some(Background::Color(blend(surface(), t, STRIP_TINT_AMOUNT))),
                ..base
            },
            None => base,
        };
        // The selected hardware-output/submix strip sits one step lighter
        // than every other strip (TotalMix makes the current submix
        // obvious the same way).  It is a background cue, independent of
        // the accent border used for a Ctrl/Shift multi-selection.
        let base = if active_surface {
            container::Style {
                background: Some(Background::Color(submix_active())),
                border: Border {
                    color: blend(border(), FADER, 0.35),
                    ..base.border
                },
                ..base
            }
        } else {
            base
        };
        if selected {
            container::Style {
                border: Border {
                    color: ACCENT,
                    width: 2.0,
                    ..base.border
                },
                ..base
            }
        } else if hovered {
            container::Style {
                border: Border {
                    color: FADER,
                    width: 1.0,
                    ..base.border
                },
                shadow: Shadow {
                    color: Color { a: 0.12, ..ACCENT },
                    offset: iced::Vector::new(0.0, 0.0),
                    blur_radius: 10.0,
                },
                ..base
            }
        } else {
            base
        }
    }
}

/// A flat `bg_deep()` read as an undifferentiated slab under the strip
/// cards. A barely-there top-to-bottom falloff (a few percent of white
/// blended in at the top, gone entirely by the bottom) reads as soft
/// ambient light instead — the same trick most DAW backgrounds use — without
/// the gradient itself ever being something you'd consciously notice.
pub fn root(_theme: &iced::Theme) -> container::Style {
    // `Radians::to_distance` subtracts another `FRAC_PI_2` internally
    // before turning this into a direction vector (its `0` is "to top",
    // matching CSS's `linear-gradient()` angle convention) — `PI`, not
    // `FRAC_PI_2`, is what actually points top-to-bottom. Verified against
    // `iced_core::angle::Radians::to_distance` source directly, not guessed
    // — a `FRAC_PI_2` first attempt rendered as a (barely visible,
    // near-flat) *horizontal* gradient instead.
    let gradient = Linear::new(std::f32::consts::PI)
        .add_stop(0.0, blend(bg_deep(), Color::WHITE, 0.035))
        .add_stop(1.0, bg_deep());
    container::Style {
        background: Some(Background::from(gradient)),
        text_color: Some(text_primary()),
        ..container::Style::default()
    }
}

/// Slightly darker than `surface()` (the strip-card background) — previously
/// identical, which flattened the top bar and the mixer strips into one
/// undifferentiated slab. Receding it toward `bg_deep()` reads as a frame the
/// strip cards sit in front of, without introducing a third named color.
pub fn top_bar(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(blend(surface(), bg_deep(), 0.45))),
        text_color: Some(text_primary()),
        ..container::Style::default()
    }
}

/// A small square toggle button (M / S / 48V / PAD) that lights up `active_color`
/// when `active` — with a soft glow to sell the "lit" look — and responds to
/// hover when inactive so it reads as clickable before you ever press it.
/// How much a `Pressed` button's background darkens versus its resting
/// `Hovered` look — the two used to render identically, so a click gave
/// no tactile "it went down" feedback, just the same hover state held a
/// little longer.
const PRESS_DARKEN: f32 = 0.30;

pub fn toggle_button(
    active: bool,
    active_color: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    armed_toggle_button(active, false, active_color)
}

/// Control Room value slider: a white rail and handle, matching the
/// neutral control chrome rather than the coloured strip controls.
pub fn db_slider(
    _theme: &iced::Theme,
    _status: slider::Status,
) -> slider::Style {
    let fill = Background::Color(text_primary());
    slider::Style {
        rail: Rail {
            // The empty part is the same near-black slot as a channel
            // fader; the filled part and handle carry the white chrome.
            backgrounds: (Background::Color(bg_deep()), fill),
            width: 6.0,
            border: Border {
                color: bg_deep(),
                width: 0.0,
                radius: RADIUS_SM.into(),
            },
        },
        handle: Handle {
            shape: HandleShape::Circle { radius: 8.0 },
            background: fill,
            border_width: 0.0,
            border_color: text_primary(),
        },
    }
}

/// Shared outline switch: when engaged the text and border take `color`
/// while the face stays dark, so it reads as a state outline rather than a
/// filled pill.
fn outline_toggle(
    active: bool,
    color: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let pressed = matches!(status, button::Status::Pressed);
        let mut background = if hovered { SURFACE_HI } else { button_face() };
        if pressed {
            background = blend(background, Color::BLACK, PRESS_DARKEN);
        }
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if matches!(status, button::Status::Disabled) {
                Color { a: 0.4, ..text_sec() }
            } else if active {
                color
            } else {
                text_sec()
            },
            border: Border {
                color: if active {
                    color
                } else if hovered {
                    FADER
                } else {
                    border()
                },
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// The 48 V switch is an outline, not a filled button: when engaged the
/// border and label turn orange, while the face stays dark.  This keeps
/// phantom power distinct without making a bright block in the strip.
pub fn phantom_button(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    outline_toggle(active, PHANTOM)
}

/// Phase is an outline switch, not a filled toggle: when engaged the text
/// and border turn green while the face stays dark, so a phase-inverted
/// channel reads as a signal-state warning rather than another bright pill.
pub fn phase_button(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    outline_toggle(active, mgreen())
}

/// The STEREO link switch uses the same orange outline as the 48 V switch
/// when the pair is engaged.
pub fn stereo_button(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    outline_toggle(active, PHANTOM)
}

/// The Control Strip masters (Mute, Solo/PFL, Fader): the same outline
/// switch as the stereo button, in the colour of the channel control they
/// stand for, with a muted version of that hue while the master is off.
pub fn master_button(
    active: bool,
    color: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let pressed = matches!(status, button::Status::Pressed);
        let mut background = if hovered { SURFACE_HI } else { button_face() };
        if pressed {
            background = blend(background, Color::BLACK, PRESS_DARKEN);
        }
        let idle = muted(color);
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if active { color } else { idle },
            border: Border {
                color: if active { color } else { idle },
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A dim version of a UI colour, for an idle master's outline and mark.
pub fn muted(color: Color) -> Color {
    Color { a: 0.42, ..color }
}

/// CUE uses the same outline language as Stereo, in green.
pub fn cue_button(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    outline_toggle(active, cue_color())
}

/// `stereo_button` with a locked form.  A Control Room channel's pair is
/// always stereo, so its control is permanently engaged and not pressable;
/// the locked style ignores the disabled state so the orange outline and
/// label stay lit instead of dimming.
pub fn stereo_button_state(
    active: bool,
    locked: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let engaged = outline_toggle(active, PHANTOM);
    move |theme, status| {
        engaged(
            theme,
            if locked {
                button::Status::Active
            } else {
                status
            },
        )
    }
}

/// A bare glyph button for the strip's settings gear.  It keeps the
/// original button's hit area and size but drops the filled pill, border
/// and shadow, so the gear reads as an icon rather than a labelled control.
/// An icon-only tab (Settings / Trim / EQ).  When its slideout is open the
/// glyph sits on the same dark surface as the panel, so the tab reads as
/// attached to the panel on its right.  `lit` keeps a non-open state (EQ
/// enabled, Trim non-zero) visible as an orange glyph without the box.
/// Hover brightens the glyph; it never borrows the orange.
pub fn panel_tab(active: bool, lit: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        // At rest the tab is only its glyph.  Hover gives it a light face;
        // an open panel keeps the darker face that reaches the strip edge.
        let background = if hovered {
            Some(Background::Color(TAB_FACE_HOVER))
        } else if active {
            Some(Background::Color(tab_face()))
        } else {
            None
        };
        button::Style {
            background,
            // Orange means "active setting", not "hovered" or "open".
            text_color: if lit {
                PHANTOM
            } else if hovered || active {
                text_primary()
            } else {
                text_sec()
            },
            // No border: the active tab's surface must merge with the
            // borderless attached panel it opens.
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}


/// `toggle_button` with a third, "armed" state: off right now, but set to
/// come back. A channel's M (or S) whose mute (solo) the sidebar's Mute
/// (Solo) master has switched off shows it, TotalMix-style, so the
/// channels a press on the master will restore stay visible: the button's
/// own color, dimmed, as a tinted outline rather than a fill. `armed` is
/// ignored while `active`.
pub fn armed_toggle_button(
    active: bool,
    armed: bool,
    active_color: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme, status| {
        if armed && !active {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let mut background = blend(bg_deep(), active_color, if hovered { 0.28 } else { 0.18 });
            if matches!(status, button::Status::Pressed) {
                background = blend(background, Color::BLACK, PRESS_DARKEN);
            }
            return button::Style {
                background: Some(Background::Color(background)),
                text_color: blend(text_sec(), active_color, 0.65),
                border: Border {
                    color: blend(border(), active_color, 0.6),
                    width: 1.0,
                    radius: RADIUS_SM.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            };
        }
        plain_toggle(active, active_color, theme, status)
    }
}

/// The channel M/S/CUE family: an engaged button is an outline like the
/// Stereo switch, not a filled pill.  `armed` keeps the dim tint used when
/// a sidebar master has switched the function off.
pub fn armed_outline_toggle(
    active: bool,
    armed: bool,
    active_color: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let outline = outline_toggle(active, active_color);
    move |theme, status| {
        if armed && !active {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let mut background =
                blend(button_face(), active_color, if hovered { 0.28 } else { 0.18 });
            if matches!(status, button::Status::Pressed) {
                background = blend(background, Color::BLACK, PRESS_DARKEN);
            }
            return button::Style {
                background: Some(Background::Color(background)),
                text_color: blend(text_sec(), active_color, 0.65),
                border: Border {
                    color: blend(border(), active_color, 0.6),
                    width: 1.0,
                    radius: RADIUS_SM.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            };
        }
        outline(theme, status)
    }
}

fn plain_toggle(
    active: bool,
    active_color: Color,
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let pressed = matches!(status, button::Status::Pressed);

    let mut background = if active {
        active_color
    } else if hovered {
        SURFACE_HI
    } else {
        button_face()
    };
    if pressed {
        background = blend(background, Color::BLACK, PRESS_DARKEN);
    }
    let border_color = if active {
        active_color
    } else if hovered {
        FADER
    } else {
        border()
    };
    let shadow = if active {
        Shadow {
            color: Color {
                a: if pressed { 0.25 } else { 0.45 },
                ..active_color
            },
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: if pressed { 3.0 } else { 6.0 },
        }
    } else {
        Shadow::default()
    };

    button::Style {
        background: Some(Background::Color(background)),
        // A button with no `on_press` at all is `Disabled` in iced: same
        // box, same place, dimmed label. Two things ride on that here —
        // the skeleton `F` in the sidebar's M/S/F row, and every control
        // the attached device cannot drive (the class-compliant
        // personality has no RME-protocol features). Before this they
        // rendered as live toggles that simply never lit.
        text_color: if matches!(status, button::Status::Disabled) {
            Color { a: 0.4, ..text_sec() }
        } else if active {
            ON_ACTIVE
        } else {
            text_sec()
        },
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        shadow,
        snap: false,
    }
}

/// The QUICK/MIXER/MATRIX view switch in the top bar, the app's primary
/// navigation: the active tab is the icon's own white pill with the
/// tile's near-black label, inactive tabs stay flat text until hovered.
/// It is the same `ACCENT`/`ON_ACTIVE` pair every engaged toggle uses,
/// so "this is the active view" reads in the same language as "this
/// switch is on".
pub fn tab_toggle(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let pressed = matches!(status, button::Status::Pressed);
        let background = if active {
            if pressed {
                blend(ACCENT, Color::BLACK, PRESS_DARKEN)
            } else {
                ACCENT
            }
        } else if pressed {
            // The resting/hover states here are SURFACE_HI/TRANSPARENT, and
            // TRANSPARENT has nothing for `blend` toward black to darken -
            // give the inactive tab its own distinct, visible press color
            // instead of falling through to "looks like hover held longer."
            Color {
                a: 0.35,
                ..ACCENT_DIM
            }
        } else if hovered {
            SURFACE_HI
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if active { ON_ACTIVE } else { text_sec() },
            border: Border {
                radius: RADIUS_SM.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// One cell of a 2-way segmented switch (`sidebar::segmented_row`).
///
/// The option the switch currently points at gets a raised graphite fill
/// (`ACCENT_DIM` plus a `FADER` border), not `toggle_button`'s
/// full-strength `ACCENT` plus glow: a switch resting on one of its sides
/// must not look as "engaged" as a channel that really is muted, or
/// neither state carries meaning. Only what is actually *on* takes a
/// state color (the mute azure, solo amber, a group link's white) while
/// switch positions stay in the graphite family.
///
/// A fixed pair (`routing: submix`) renders as `segment_button(true)` with
/// no `on_press` on either side and is still drawn lit, unlike
/// `toggle_button`'s disabled look, which stays flat regardless of state.
pub fn segment_button(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let pressed = matches!(status, button::Status::Pressed);

        let background = if active {
            SURFACE_HI
        } else if hovered {
            surface()
        } else {
            button_face()
        };
        let border_color = if active || hovered { FADER } else { border() };

        button::Style {
            background: Some(Background::Color(if pressed {
                blend(background, Color::BLACK, PRESS_DARKEN)
            } else {
                background
            })),
            text_color: if active { text_primary() } else { text_sec() },
            border: Border {
                color: border_color,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// The shared track of a joined 2-way switch (`sidebar::segmented_row`):
/// one recessed rounded body that both cells sit inside, so the pair
/// reads as a single control with a sliding thumb instead of two
/// unrelated pills.
pub fn segment_track(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(bg_deep())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        ..container::Style::default()
    }
}

/// One cell inside a `segment_track`: the lit side is a raised graphite
/// thumb, the other is bare text on the track that only gains a fill
/// under the cursor. Same semantics as `segment_button` (a position, not
/// an "on" state), minus the per-cell outline the track now provides.
pub fn segment_cell(active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let pressed = matches!(status, button::Status::Pressed);
        let background = if active {
            SURFACE_HI
        } else if hovered {
            surface()
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(Background::Color(if pressed {
                blend(background, Color::BLACK, PRESS_DARKEN)
            } else {
                background
            })),
            // A side with no `on_press` is inert: dimmed like every other
            // disabled control, so a switch that cannot act does not look
            // like one that just did nothing.
            text_color: match (status, active) {
                (button::Status::Disabled, _) => Color { a: 0.4, ..text_sec() },
                (_, true) => text_primary(),
                (_, false) => text_sec(),
            },
            border: Border {
                color: if active { border() } else { Color::TRANSPARENT },
                width: 1.0,
                radius: (RADIUS_SM - 2.0).into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn plain_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Pressed => blend(SURFACE_HI, Color::BLACK, PRESS_DARKEN),
        button::Status::Hovered => SURFACE_HI,
        _ => surface(),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: text_primary(),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A collapsible section header (`sidebar::panel_header`): no fill at all,
/// even under the cursor, since a hover box around a whole header row
/// would outweigh the panel it heads. Hover only brightens the chevron;
/// the tick, label and rule keep their own colors.
pub fn header_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered | button::Status::Pressed => text_primary(),
            _ => text_sec(),
        },
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The sidebar's device chip as a button: the `chip` look at rest, the
/// field-style ring under the cursor and while the popover it opens is up.
pub fn chip_button(open: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = match status {
            button::Status::Pressed => blend(button_face(), Color::BLACK, PRESS_DARKEN),
            button::Status::Hovered => blend(button_face(), SURFACE_HI, 0.5),
            _ => button_face(),
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: text_primary(),
            border: Border {
                color: if open || hovered { FADER } else { border() },
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A flat, textual button: no box at rest, brightening to `SURFACE_HI`
/// under the cursor.
///
/// For secondary actions and utilities (undo/redo, a panel's "store", its
/// "−"/"+") that were previously the same bordered box as a real control.
/// In a 210px column that added up to nothing but boxes, and a row of
/// equally-sized boxes carries no information about which of them matters.
///
/// `Disabled` (a button with no `on_press`, e.g. undo with an empty stack)
/// drops to a quarter-opacity label: the flat look has no border to gray
/// out, so that is what keeps "nothing to undo" from reading as "click
/// me".
pub fn ghost_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let (background, text_color) = match status {
        button::Status::Pressed => (
            Background::Color(blend(surface(), Color::BLACK, PRESS_DARKEN)),
            text_primary(),
        ),
        button::Status::Hovered => (Background::Color(SURFACE_HI), text_primary()),
        button::Status::Disabled => (
            Background::Color(Color::TRANSPARENT),
            Color { a: 0.4, ..text_sec() },
        ),
        _ => (Background::Color(Color::TRANSPARENT), text_sec()),
    };
    button::Style {
        background: Some(background),
        text_color,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The small white tick to the left of a section header label.
pub fn accent_bar(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ACCENT)),
        border: Border {
            radius: RADIUS_XS.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// A recessed group container — used to visually cluster related top-bar
/// controls (e.g. the scene name/save/load trio) against the flatter
/// top-bar background.
/// Background used when a knob label is deliberately overlapped with the
/// unused bottom margin of its knob.
pub fn label_backdrop(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(surface())),
        border: Border {
            color: Color { a: 0.0, ..surface() },
            width: 0.0,
            radius: 2.0.into(),
        },
        text_color: Some(text_primary()),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn chip(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(bg_deep())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        text_color: Some(text_primary()),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn text_input(
    _theme: &iced::Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    use iced::widget::text_input::Status;
    let border_color = match status {
        Status::Focused { .. } => ACCENT,
        _ => border(),
    };
    iced::widget::text_input::Style {
        background: Background::Color(surface()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        icon: text_sec(),
        placeholder: text_sec(),
        value: text_primary(),
        selection: ACCENT_DIM,
    }
}

pub fn pick_list(
    _theme: &iced::Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    use iced::widget::pick_list::Status;
    let border_color = match status {
        Status::Opened { .. } | Status::Hovered => FADER,
        Status::Active => border(),
    };
    iced::widget::pick_list::Style {
        text_color: text_primary(),
        placeholder_color: text_sec(),
        handle_color: text_sec(),
        background: Background::Color(surface()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
    }
}

/// The same pick list surface with the selected label hidden: the Settings
/// panel draws its own centred label on top, so long values can clip
/// symmetrically without running under the handle.
pub fn pick_list_no_text(
    theme: &iced::Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let mut style = pick_list(theme, status);
    style.text_color = Color::TRANSPARENT;
    style
}

/// The default 10px scrollbar/scroller reads chunky next to everything
/// else we've thinned down — 4px is still comfortably grabbable but looks
/// modern rather than like default browser chrome.
pub fn thin_scrollbar() -> iced::widget::scrollable::Scrollbar {
    iced::widget::scrollable::Scrollbar::new()
        .width(4)
        .scroller_width(4)
}

/// A scrollbar with no visible rail or scroller.  Slideout panels use it so
/// a wheel can still reach clipped content at extreme zoom without ever
/// drawing a scrollbar inside the panel.
pub fn hidden_scrollbar() -> iced::widget::scrollable::Scrollbar {
    iced::widget::scrollable::Scrollbar::new()
        .width(0)
        .scroller_width(0)
}

/// A thin, minimal scrollbar rail — the scroller brightens on hover/drag
/// instead of the default Iced chrome, matching the rest of the app.
pub fn scrollable(
    _theme: &iced::Theme,
    status: iced::widget::scrollable::Status,
) -> iced::widget::scrollable::Style {
    use iced::widget::scrollable::{AutoScroll, Rail, Scroller, Status};

    let (h_active, v_active) = match status {
        Status::Active { .. } => (false, false),
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        ),
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        ),
    };

    let rail = |active: bool| Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        scroller: Scroller {
            background: Background::Color(if active { FADER } else { border() }),
            border: Border {
                radius: RADIUS_SM.into(),
                ..Border::default()
            },
        },
    };

    iced::widget::scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail(v_active),
        horizontal_rail: rail(h_active),
        gap: None,
        auto_scroll: AutoScroll {
            background: Background::Color(surface()),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: text_primary(),
        },
    }
}

pub fn menu(_theme: &iced::Theme) -> menu::Style {
    menu::Style {
        background: Background::Color(surface()),
        border: Border {
            color: border(),
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        text_color: text_primary(),
        selected_text_color: text_primary(),
        selected_background: Background::Color(ACCENT_DIM),
        shadow: Shadow::default(),
    }
}
