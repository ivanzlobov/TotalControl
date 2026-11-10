//! A single channel strip: label + type tag, mute/solo, 48V/PAD, fader+VU,
//! dB readout (double-click to edit), pan readout.

use iced::advanced::text as advanced_text;
use iced::keyboard::Modifiers;
use iced::widget::canvas::{self, Canvas, Frame, Geometry};
use iced::widget::{button, column, container, mouse_area, row, text, text_input, tooltip, Pin, Stack};
use iced::{alignment, mouse, Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};
use std::time::Instant;
use totalcontrol_core::ChannelId;

use crate::app::{short_label, Message};
use crate::theme;
use crate::widgets::fader::{
    fader, vu_meter, BusSlot, Fader, FaderColumns, MeterFrame, CHANNEL_SLOT_W, FX_FADER_SCALE,
    FX_SLOT_W,
};
use crate::widgets::layout::{
    ChannelKind, ElementPlacement, Features, GuiLayoutConfig, Panel, Rect, StripLayout, TemplateId,
    SUBMIX_H,
};
use crate::widgets::knob::{knob, Knob};
use crate::widgets::pause_mark::pause_mark;

/// Base sizes at `scale == 1.0` (`theme::SCALE_DEFAULT`) — every dimension
/// in a strip is one of these times `StripParams::scale`, so the window's
/// adaptive scale (see `app::recompute_ui_scale`) resizes strips the same
/// way it resizes text.
/// Full-strip height at scale 1.0.  Every full strip is pinned to this
/// height (inputs, playback and outputs), and the Settings/Trim/EQ slideouts
/// use the same value, so a slideout always matches the channel beside it.
pub(crate) const STRIP_H: f32 = 350.0;

const SETTINGS_GLYPH: f32 = 14.0;
/// Width of a full channel strip at `scale == 1.0`.
///
/// The 78 px content box splits into a 55 px fader canvas and a 20 px tab
/// glyph column with its gaps.  The canvas holds the 24 px VU, a 2 px gap
/// and the 26 px track; the track stays on the content's centre line.
pub(crate) const STRIP_W: f32 = 90.0;

/// The message a Control Room strip's FX fader sends: FX Return on an
/// output, FX send on anything else.
fn fx_message(cid: ChannelId, db: f32) -> Message {
    match cid {
        ChannelId::Output(idx) => Message::FxReturnChanged(idx, db),
        _ => Message::FxSendChanged(cid, db),
    }
}

/// Linear amplitude to dB, for the FX fader's value field.
fn db_of(linear: f32) -> f32 {
    if linear <= 0.000_01 {
        -65.0
    } else {
        (20.0 * linear.log10()).clamp(-65.0, 6.0)
    }
}
/// Collapsed strips need to stand the same total height as a full strip
/// (see `collapsed_strip`'s own doc comment) despite having far fewer
/// rows — name/M/S/meter/expand vs. the full strip's
/// header/pan/M-S/fader/dB/route. The meter is what absorbs the
/// difference. Was `76.0`, re-tuned down after the name label became a
/// tall rotated canvas (see `ROTATED_LABEL_LEN`) instead of one short
/// text line — that canvas already eats most of the vertical slack the
/// old flat offset existed to add. Empirically measured the same way the
/// original number was (pixel-measuring a collapsed and an adjacent full
/// strip side by side in a live screenshot), not derived on paper.
/// Fixed chrome of a collapsed strip: top padding, the five row gaps, the
/// rotated name, the mute, solo and expand rows, and the unused bottom slot
/// that matches B2.  The meter absorbs the rest, so the expand button lands
/// on B1 and the strip keeps the full strip's `STRIP_H`.
const COLLAPSED_CHROME_H: f32 = theme::SPACE_SM
    + SUBMIX_H
    + theme::SPACE_HAIRLINE * 5.0
    + ROTATED_LABEL_LEN
    + ICON_BTN_H * 3.0;
const COLLAPSED_METER_H: f32 = STRIP_H - COLLAPSED_CHROME_H;
/// Collapsed strips are a glance-only readout: name, Mute/Solo (kept
/// live — see this function's own doc comment), and a VU meter; no
/// fader, no pan, no route. Width is set by the header (name + expand
/// button), not the meter, which is narrower than that on its own.
/// Deliberately the same `COLLAPSED_W` for every channel kind, output
/// strips included — collapsing is opting *out* of emphasis, so there's
/// nothing to differentiate here.
///
/// Narrowed from `60` after comparing against a live TotalMix screenshot
/// at pixel level (`Screenshot 2026-08-30 124715.png`): real collapsed
/// strips there measure ~29px against ~75px full strips — a ~0.39 ratio,
/// well under half, not the ~0.6 ours had. `44` keeps that same ratio
/// against the current (wider) `STRIP_W` while still leaving room for
/// the existing `vu_meter` widget's own fixed meter/ruler column plus
/// padding without clipping it — matching TotalMix's exact ~0.39 would
/// need a narrower meter widget too, not attempted this pass.
pub(crate) const COLLAPSED_W: f32 = 44.0;

/// Height of the collapsed strip's M/S/CUE and expand rows.
const ICON_BTN_H: f32 = 14.0;

/// Width (at `scale == 1.0`) of the Route flyout `app.rs` opens over a
/// strip's right neighbor — `pub(crate)` because `app.rs` needs it to
/// size the popover. Settings doesn't use this: it pushes the row instead
/// of overlaying it, sized to match the strip's own width (see the
/// reference design), not a fixed constant — see `FlyoutKind`.
pub(crate) const FLYOUT_W: f32 = 140.0;

/// Width (at `scale == 1.0`) of the EQ flyout — wide enough for the
/// three-column band matrix (type/gain/freq/Q) plus its row labels, and
/// the response plot above it.
pub(crate) const EQ_FLYOUT_W: f32 = 248.0;

/// Width of the Dynamics flyout — two control columns plus the Auto Level
/// block, matching the manual's panel grouping.
pub(crate) const DYN_FLYOUT_W: f32 = 230.0;

/// Which flyout (at most one, and only one per strip) is open — the gear
/// icon opens `Settings` (48V/PAD/Sensitivity, moved out of the strip's
/// own vertical flow so it doesn't grow/shrink the whole card; Gain stays
/// inline instead — see `app.rs::settings_popover`'s doc comment for why),
/// the route trigger opens `Route` (which output bus), the EQ trigger
/// (analog inputs only) opens `Eq` (3-band + low-cut, see
/// `app.rs::eq_popover`), and the "T" trigger (every hardware input, see
/// `StripParams::has_trim`) opens `Trim` (see `app.rs::trim_popover`).
/// All render as the same kind of animated panel sliding out over the
/// strip's right neighbor — see `app.rs::with_flyout` (Route) and
/// `mixer_view`'s input loop (Settings/Eq/Trim, which push the row
/// instead).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FlyoutKind {
    Route,
    Settings,
    Eq,
    Dynamics,
    Trim,
}

/// The full (uncollapsed) width for a given channel — every channel kind
/// renders at the same width. Takes `cid` (unused) rather than being a
/// bare constant so `app.rs::set_collapsed`/`rendered_strip_width` don't
/// need to change if a future kind ever needs to differ again.
pub(crate) fn full_width(_cid: ChannelId) -> f32 {
    STRIP_W
}

/// Slideout speed, independent of the panel width: a 90 px Settings panel
/// still takes 90 ms, while a 248 px EQ panel takes 248 ms instead of
/// racing through the same 90 ms. One pixel per millisecond.
///
/// A width tween is a navigation aid, not an effect. A fixed duration made
/// wide panels slide visibly faster than narrow ones; a fixed speed keeps
/// the motion even.
const COLLAPSE_SPEED_PX_PER_MS: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct CollapseAnim {
    pub prev: f32,
    pub value: f32,
    pub since: Instant,
}

impl CollapseAnim {
    fn duration_ms(&self) -> f32 {
        (self.value - self.prev).abs() / COLLAPSE_SPEED_PX_PER_MS
    }

    pub fn at(&self, now: Instant) -> f32 {
        let duration = self.duration_ms();
        if duration <= f32::EPSILON {
            return self.value;
        }
        let t = (now.duration_since(self.since).as_secs_f32() * 1000.0 / duration)
            .clamp(0.0, 1.0);
        self.prev + (self.value - self.prev) * t
    }

    pub fn is_settling(&self, now: Instant) -> bool {
        let duration = self.duration_ms();
        duration > f32::EPSILON
            && now.duration_since(self.since).as_secs_f32() * 1000.0 < duration
    }
}

pub struct StripParams<'a> {
    pub cid: ChannelId,
    /// The layout source for this strip: the user document when it loads,
    /// otherwise the built-in defaults.
    pub layout: &'a GuiLayoutConfig,
    pub output_idx: usize,
    /// Human label of the current submix/output bus (device-derived).
    pub bus_label: String,
    /// Whether the connection this strip is drawn for can drive RME's own
    /// protocol (see `RmeDevice::protocol_features`). False on the
    /// class-compliant personality, where EQ, Trim, Phase, Split and
    /// Loopback would only be able to fail: they are left out entirely,
    /// the same thing the `has_*` flags above already do for features a
    /// channel does not have.
    pub protocol_features: bool,
    /// Whether this backend can toggle a playback pair's stereo split
    /// (see `RmeDevice::stereo_split_supported`).  Separate from
    /// `protocol_features`: the UCX v1 backend supports the split through
    /// its matrix model without the full RME feature set.
    pub stereo_split_supported: bool,
    pub name: String,
    pub type_tag: Option<(&'static str, Color)>,
    pub vol: f32,
    pub pan: i8,
    /// Hardware-output balance is available for this strip (UC: unequal
    /// left/right output levels; UCX/other backends not implemented).
    pub output_balance_supported: bool,
    /// FX send (inputs and software playback) or FX Return (outputs), in dB.
    /// `None` on a backend without the FX bus. Drawn as the thin fader to
    /// the right of the main one, as TotalMix does.
    pub fx_level: Option<f32>,
    pub meter: MeterFrame,
    /// Whether `meter` is a real reading on the active backend — see
    /// `fader::draw_meter`'s doc comment.
    pub meter_available: bool,
    /// Set only when this strip represents a LINKED pair — the right
    /// channel's own meter, splitting the meter column into two thin
    /// bars (see `fader::Fader::meter2`'s own doc comment). `None` for
    /// every unlinked/mono strip.
    pub meter2: Option<MeterFrame>,
    pub meter2_available: bool,
    /// Slow M1 readout values: left and right channel of the strip.  They
    /// are sampled by the app at a fixed interval, not at meter tick rate.
    pub readout: [Option<f32>; 2],
    /// When the device service owns metering, the main fader reads this
    /// shared bus from its own canvas instead of the view's meter frames.
    pub meter_bus: Option<
        std::sync::Arc<std::sync::Mutex<crate::device_service::MeterSnapshot>>,
    >,
    pub meter_slot: Option<BusSlot>,
    /// Trim in dB applied to the bus meter, so the strip shows the signal
    /// after trim exactly like the view-fed path.
    pub meter_trim: f32,
    /// Colour of this strip's fader cap when it belongs to a fader group:
    /// the temporary group, a stored group, or the group being edited.
    pub fader_group: Option<Color>,
    /// While a group-assignment session is open, a plain click on the fader
    /// picks the strip instead of dragging it.
    pub fader_select_only: bool,
    pub has_48v: bool,
    pub has_pad: bool,
    pub has_instrument: bool,
    pub phantom: bool,
    pub pad: bool,
    pub instrument: bool,
    /// Whether this input has a preamp gain control at all (Mic and
    /// Instrument only — Line/SPDIF/ADAT inputs have no gain knob).
    pub has_gain: bool,
    /// Raw hardware gain units, not dB — see `Message::Gain`.
    pub gain: u32,
    /// Right-side gain of a linked input pair, when it differs from the left.
    /// The Gain knob shows both values, left above right.
    pub gain_right: Option<u32>,
    pub gain_max: u32,
    /// Inputs 3/4 use 0.5 dB units; display `gain / 2.0` dB.
    pub gain_half: bool,
    /// Whether this input has a sensitivity switch (Instrument only).
    pub has_sensitivity: bool,
    /// `true` = +4dBu, `false` = -10dBV.
    pub sensitivity_plus4: bool,
    /// Whether this input has the UCX three-value Level control.
    pub has_level: bool,
    /// Current Level label (`-10dBV`, `+4dBu`, `LoGain`).
    pub level: totalcontrol_core::InputLevel,
    /// Whether this input has AutoSet (UCX inputs 1/2).
    pub has_autoset: bool,
    pub autoset: bool,
    /// Whether this channel has a Dynamics panel (UCX inputs/outputs).
    pub has_dynamics: bool,
    pub dynamics_on: bool,
    /// Whether this input has an EQ strip at all (the 4 analog inputs
    /// only — see `InputChannel::eq`).
    pub has_eq: bool,
    /// Whether the EQ is currently engaged — lights up the trigger button
    /// even when its flyout isn't open, like the 48V/PAD toggles.
    pub eq_enabled: bool,
    /// Whether this channel has a Trim control — every hardware input
    /// (analog and digital alike, unlike `has_gain`); Playback and Output
    /// keep an empty tab slot instead.
    pub has_trim: bool,
    /// Current trim value in dB (-65..+6) — see `InputChannel::trim`.
    pub trim: f32,
    /// Hover text for `T`: "Show trim" at 0 dB, otherwise the applied
    /// attenuation.
    pub trim_tip: String,
    /// Whether this input has a Phase Ø invert switch — the 4 analog
    /// inputs only, and only where the backend decodes phase (the UC does
    /// not yet; see `UC-TODO-06`).
    pub has_phase: bool,
    pub phase: bool,
    /// Output-only: `OutputChannel::loopback`.
    pub loopback: bool,
    /// Whether this connection can drive the loopback register (`0x15`).
    /// Separate from `protocol_features`: the UC has no EQ/FX but does
    /// have loopback.
    pub loopback_supported: bool,
    /// Whether this channel's pair is currently a linked stereo bus
    /// (`RmeDevice::{input_pair,playback,output}_linked`, whichever
    /// matches `cid`'s kind) — TotalMix's default, shown/toggled from
    /// the Settings flyout for every channel kind.
    pub stereo_linked: bool,
    /// Which flyout (if any) is currently open *for this strip* — lights
    /// up the matching trigger (gear icon for `Settings`, the route
    /// button for `Route`); `app.rs` owns the actual popover content.
    pub open_flyout: Option<FlyoutKind>,
    pub mute: bool,
    pub solo: bool,
    /// PFL replaces Solo for this input strip (Control Room setting).
    pub pfl: bool,
    /// Off, but held for the sidebar's Mute / Solo master to bring back
    /// (`theme::armed_outline_toggle`). Set by `app::finish_params`.
    pub mute_armed: bool,
    pub solo_armed: bool,
    pub cue: bool,
    pub default_vol: f32,
    pub editing: bool,
    pub edit_buf: &'a str,
    /// The name field is open for typing (`Message::RenameStart`), with
    /// what has been typed so far. Set by `app::finish_params`.
    pub renaming: bool,
    pub rename_buf: &'a str,
    pub drag_range: Option<(f32, f32)>,
    pub modifiers: Modifiers,
    pub collapsed: bool,
    pub collapse_anim: Option<CollapseAnim>,
    pub scale: f32,
    pub selected: bool,
    pub hovered: bool,
    /// This output strip is the current submix destination (TotalMix's
    /// lighter active-output card).
    pub active_surface: bool,
    /// `name` is the user's own label; do not strip a hardware "PCM "/"OUT "
    /// prefix from it, unlike the original names.
    pub name_custom: bool,
    /// The name is fixed by the app (Control Room destinations) and must not
    /// be replaced by a user label.
    pub name_locked: bool,
    /// Double-clicking the name may open the rename field. Control Room
    /// channels set this off: the name still selects the channel.
    pub renameable: bool,
}

/// A button's own padding-based centering isn't reliable across glyphs of
/// different intrinsic width (e.g. "S" sat visibly left of center while "M"
/// looked fine) — force it explicitly instead of trusting the default. The
/// default 1.2x line-height also reserves descender space these glyphs
/// (M, S, no descenders) never use, which reads as "sitting too high" once
/// centered — tightening it to 1:1 removes that residual vertical bias.
pub(crate) fn centered_label<'a>(s: &'a str, size: f32) -> Element<'a, Message> {
    container(
        text(s)
            .size(size)
            .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
                size,
            ))),
    )
    .center(Length::Fill)
    .into()
}

/// `label` with the master's pause mark before or after it, centred in a
/// button; without `paused` it is the plain centred label.
fn centered_master_label<'a>(
    label: &'a str,
    size: f32,
    scale: f32,
    color: Color,
    paused: bool,
    pause_before: bool,
) -> Element<'a, Message> {
    if !paused {
        return centered_label(label, size);
    }
    let label = text(label)
        .size(size)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
            size,
        )));
    let mark = pause_mark(color, scale);
    let content: Element<'a, Message> = if pause_before {
        row![mark, label]
            .spacing(theme::SPACE_TIGHT * scale)
            .align_y(alignment::Vertical::Center)
            .into()
    } else {
        row![label, mark]
            .spacing(theme::SPACE_TIGHT * scale)
            .align_y(alignment::Vertical::Center)
            .into()
    };
    container(content).center(Length::Fill).into()
}

/// Wraps a control in a hover tooltip — for the abbreviations (M, S, 48V,
/// PAD) that read as pro-audio jargon to anyone not already fluent in it.
/// A short delay so it doesn't flash on every incidental mouse-over while
/// moving across the strip toward something else.
pub(crate) fn hint<'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl Into<std::borrow::Cow<'a, str>>,
    scale: f32,
) -> Element<'a, Message> {
    let label: std::borrow::Cow<'a, str> = label.into();
    tooltip(
        content,
        container(text(label).size(theme::TEXT_XS * scale).color(theme::text_primary()))
            .padding(theme::SPACE_SM * scale)
            .style(theme::panel),
        tooltip::Position::Top,
    )
    .gap(4.0 * scale)
    .delay(std::time::Duration::from_millis(400))
    .into()
}

/// Place one element at a slot from `StripLayout`.
fn pinned<'a>(
    content: impl Into<Element<'a, Message>>,
    rect: Rect,
    card: Rect,
) -> Element<'a, Message> {
    Pin::new(
        container(content.into())
            .width(Length::Fixed(rect.width))
            .height(Length::Fixed(rect.height)),
    )
    .x(rect.x)
    .y(rect.y)
    .width(Length::Fixed(card.width))
    .height(Length::Fixed(card.height))
    .into()
}

/// One of the two numerical strength readouts.  Right-aligned, text only.
fn strength_cell<'a>(value: Option<f32>, w: f32, h: f32, scale: f32) -> Element<'a, Message> {
    let label = value.map(strength_text).unwrap_or_default();
    container(
        text(label)
            .size(theme::TEXT_XS * scale)
            .color(theme::text_primary())
            .wrapping(advanced_text::Wrapping::None),
    )
    .width(Length::Fixed(w))
    .height(Length::Fixed(h))
    .padding(iced::Padding {
        right: theme::SPACE_TIGHT * scale,
        ..iced::Padding::ZERO
    })
    .align_x(iced::Alignment::End)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The lowest reading the readout shows.  The UCX's smallest non-silent
/// power word is `power_dbfs(32)` = -150.5 dBFS (power 31 and below is
/// "below range"), and TotalMix stops at the same place.  Anything lower is
/// the bar's decay tail, not a measurement, and reads as silence.
const READOUT_FLOOR_DB: f32 = -150.6;

fn strength_text(linear: f32) -> String {
    if linear <= 0.0 {
        return "-∞".to_string();
    }
    let db = 20.0 * linear.log10();
    if db < READOUT_FLOOR_DB {
        return "-∞".to_string();
    }
    if db.abs() >= 100.0 {
        format!("{db:.0}")
    } else {
        format!("{db:.1}")
    }
}

/// Empty tab slot: the glyph column is reserved, but no face is drawn.
fn empty_tab<'a>(w: f32, h: f32) -> Element<'a, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .style(theme::empty_slot)
        .into()
}

/// Slideout tab: the glyph is centred in its 20 px column, while the face
/// reaches the channel's right edge.
#[allow(clippy::too_many_arguments)]
fn slideout_tab<'a>(
    label: &'static str,
    active: bool,
    lit: bool,
    message: Message,
    tip: impl Into<std::borrow::Cow<'a, str>>,
    scale: f32,
    glyph_w: f32,
    face_w: f32,
    h: f32,
    glyph_size: f32,
) -> Element<'a, Message> {
    let glyph = container(centered_label(label, glyph_size * scale))
        .width(Length::Fixed(glyph_w))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    let face = row![glyph, iced::widget::Space::new().width(Length::Fill)]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center);
    hint(
        button(face)
            .padding(0)
            .width(Length::Fixed(face_w))
            .height(Length::Fixed(h))
            .style(theme::panel_tab(active, lit && !active))
            .on_press(message),
        tip,
        scale,
    )
}

/// Channel minimize button: glyph centred in its column, face at the edge.
fn minimize_button(
    cid: ChannelId,
    scale: f32,
    glyph_w: f32,
    face_w: f32,
    h: f32,
) -> Element<'static, Message> {
    let glyph = container(centered_label("−", theme::TEXT_SM * scale))
        .width(Length::Fixed(glyph_w))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    let face = row![glyph, iced::widget::Space::new().width(Length::Fill)]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center);
    hint(
        button(face)
            .padding(0)
            .width(Length::Fixed(face_w))
            .height(Length::Fixed(h))
            .style(theme::toggle_button(false, theme::ACCENT))
            .on_press(Message::ToggleCollapse(cid)),
        "Collapse",
        scale,
    )
}

/// Full-width submix selector.  The label stays centred on the channel while
/// the arrow sits at the right edge.
fn submix_button<'a>(
    cid: ChannelId,
    label: String,
    open: bool,
    scale: f32,
    w: f32,
    h: f32,
) -> Element<'a, Message> {
    let label = container(
        text(label)
            .size(theme::TEXT_XS * scale)
            .wrapping(advanced_text::Wrapping::None),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill);
    let arrow = container(text("▸").size(theme::TEXT_XS * scale))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::End)
        .align_y(iced::Alignment::Center);
    let face = Stack::new()
        .push(label)
        .push(arrow)
        .width(Length::Fill)
        .height(Length::Fill);
    hint(
        button(face)
            .padding([0.0, theme::SPACE_SM * scale])
            .width(Length::Fixed(w))
            .height(Length::Fixed(h))
            .style(theme::toggle_button(open, theme::ACCENT))
            .on_press(Message::ToggleFlyout(cid, FlyoutKind::Route)),
        "Change output bus",
        scale,
    )
}

/// How long (in canvas-local x, *before* the -90° rotation — i.e. how
/// tall the finished label actually stands once rotated) `rotated_label`
/// reserves for the name — sized for the longest real channel name this
/// app ever shows ("ADAT7/8 PB" and friends), not just the common case,
/// since unlike a plain `text()` widget this canvas has no shrink-to-fit:
/// anything longer than this would silently draw past its own bounds and
/// overlap the Mute button below it.
const ROTATED_LABEL_LEN: f32 = 64.0;

/// A read-only text canvas rotated -90° (reads bottom-to-top) — the
/// collapsed strip's own name label. Confirmed against a live TotalMix
/// screenshot: collapsed buses are narrow enough (see `COLLAPSED_W`'s own
/// doc comment) that a normal horizontal name doesn't fit at all, so
/// TotalMix rotates it instead of truncating or wrapping it. iced's plain
/// `text()` widget has no rotation of its own, hence a small dedicated
/// canvas rather than reusing `centered_label`.
struct RotatedLabel {
    text: String,
    size: f32,
}

impl<Message> canvas::Program<Message> for RotatedLabel {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            // -90°: in this canvas's clockwise-from-+x, y-down convention
            // (matching `iced_graphics::geometry::path::Arc` and
            // `widgets::knob`'s own `angle_of`), this maps local "reading
            // direction" (+x) to global "up" — i.e. the text climbs the
            // strip bottom-to-top, matching the reference exactly rather
            // than guessed-and-checked.
            frame.rotate(-std::f32::consts::FRAC_PI_2);
            frame.fill_text(canvas::Text {
                content: self.text.clone(),
                position: Point::ORIGIN,
                color: theme::text_primary(),
                size: self.size.into(),
                align_x: advanced_text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        });
        vec![frame.into_geometry()]
    }
}

fn rotated_label<'a, Msg: 'a + Clone>(text: String, size: f32, scale: f32) -> Element<'a, Msg> {
    Canvas::new(RotatedLabel { text, size })
        .width(Length::Fill)
        .height(Length::Fixed(ROTATED_LABEL_LEN * scale))
        .into()
}

/// Full-strip header — just the name (+ type tag), centered, TotalMix-style
/// (see the reference design: no icons in the header at rest). See
/// `collapsed_header_row` for the collapsed strip's own header, which
/// still needs its chevron.
///
/// Double-clicking the name renames the strip (`Message::RenameStart`):
/// while `renaming`, the name is a text field, Enter keeps the typed name
/// and Escape drops it. The name's own `mouse_area` captures its clicks
/// (iced's `mouse_area` captures every press once it has a double-click
/// handler), so it forwards single clicks as `StripClicked` itself; that
/// keeps Ctrl/Shift-click selection working on the name, and stops a
/// double-click there from also collapsing the strip.
fn header_row<'a>(
    cid: ChannelId,
    name: &str,
    name_custom: bool,
    type_tag: Option<(&'static str, Color)>,
    renameable: bool,
    renaming: bool,
    rename_buf: &'a str,
    scale: f32,
) -> Element<'a, Message> {
    let shown_name = if name_custom {
        name
    } else {
        short_label(name)
    };
    if renaming {
        return text_input(shown_name, rename_buf)
            .id(crate::app::RENAME_INPUT_ID)
            .on_input(Message::RenameChanged)
            .on_submit(Message::RenameCommit)
            .style(theme::text_input)
            .size(theme::TEXT_SM * scale)
            .padding([theme::SPACE_TIGHT * scale, theme::SPACE_SM * scale])
            .width(Length::Fill)
            .into();
    }
    // `.wrapping(Wrapping::None)` — without it, a long name ("Instr. 3/4",
    // "ADAT7/8") wraps to two lines partway through the collapse/expand
    // width animation, once the shrinking card gets narrower than the
    // name's natural width, growing the whole strip's *height* for the
    // ~90ms the animation is mid-flight before it settles back down —
    // user report: "pendant que ça se minimise le bus augmente de hauteur
    // un peu." No text in this card is meant to ever wrap (every label
    // here is a single short line by design), so this is a correctness
    // fix for the whole card, not just the header specifically — see the
    // matching `.wrapping(None)` added to every other `text()` in
    // `full_strip` below for the same reason.
    let mut header = row![text(shown_name.to_string())
        .color(theme::text_primary())
        .size(theme::TEXT_MD * scale)
        .wrapping(advanced_text::Wrapping::None)]
    .spacing(theme::SPACE_HAIRLINE);
    if let Some((tag, color)) = type_tag {
        header = header.push(
            text(tag)
                .color(color)
                .size(theme::TEXT_XS * scale)
                .wrapping(advanced_text::Wrapping::None),
        );
    }
    let header = mouse_area(header).on_press(Message::StripClicked(cid));
    let header = if renameable {
        header
            .on_double_click(Message::RenameStart(cid))
            .interaction(mouse::Interaction::Text)
    } else {
        header
    };
    let header = if renameable {
        hint(header, "Double-click to rename", scale)
    } else {
        header.into()
    };
    container(header)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .align_y(iced::Alignment::Center)
        .into()
}

/// A collapsed strip is a glance-only readout, but per the reference
/// design keeps Mute/Solo live (unlike the fader/pan/route, still traded
/// away for space) — collapsing is for strips you're not actively
/// adjusting the level of, not ones you'd never want to silence in a
/// hurry. Only ever rendered fully settled (see `strip()`'s dispatch), so
/// `w` is always `COLLAPSED_W`, but it's threaded through rather than
/// hardcoded to keep this in lockstep with `full_strip`'s signature.
fn collapsed_strip<'a>(p: StripParams<'a>, w: f32) -> Element<'a, Message> {
    let cid = p.cid;
    let scale = p.scale;
    let btn_h = ICON_BTN_H * scale;

    let mute_btn = hint(
        button(centered_master_label(
            "M",
            theme::TEXT_XS * scale,
            scale,
            theme::mute_color(),
            p.mute_armed,
            true,
        ))
            .width(Length::Fill)
            .height(btn_h)
            .style(theme::armed_outline_toggle(p.mute, p.mute_armed, theme::mute_color()))
            .on_press(Message::Mute(cid, !p.mute)),
        if p.mute_armed {
            "Mute (held by the M master; its next press brings it back)"
        } else {
            "Mute"
        },
        scale,
    );
    let solo_btn = hint(
        button(centered_master_label(
            if p.pfl { "PFL" } else { "S" },
            theme::TEXT_XS * scale,
            scale,
            theme::solo_color(),
            !p.pfl && p.solo_armed,
            false,
        ))
        .width(Length::Fill)
        .height(btn_h)
        .style(theme::armed_outline_toggle(p.solo, p.solo_armed, theme::solo_color()))
        .on_press(Message::Solo(cid, !p.solo)),
        if p.pfl {
            "PFL (pre-fader listen) to the Cue/PFL target"
        } else if p.solo_armed {
            "Solo (held by the S master; its next press brings it back)"
        } else {
            "Solo"
        },
        scale,
    );
    // "+" rather than a chevron — the reference puts the expand trigger
    // at the *bottom* of the collapsed card, not a top corner (which is
    // also where the full strip's own gear/chevron column sits now, at
    // the fader's mid-height on the right — matching that position here
    // would put "expand" somewhere a collapsed card, with no fader, has
    // nothing to anchor to).
    let expand_btn = hint(
        button(centered_label("+", theme::TEXT_SM * scale))
            .padding(0)
            .width(Length::Fill)
            .height(btn_h)
            .style(theme::toggle_button(false, theme::ACCENT))
            .on_press(Message::ToggleCollapse(cid)),
        "Expand",
        scale,
    );

    // Inputs and software playbacks keep S; outputs keep CUE. Both occupy
    // the same second slot, so the collapsed cards stay aligned.
    let cue_btn = hint(
        button(centered_label("CUE", theme::TEXT_XS * scale))
            .width(Length::Fill)
            .height(btn_h)
            .style(theme::cue_button(p.cue))
            .on_press(Message::CueChanged(cid, !p.cue)),
        "CUE — exclusively monitor this output's submix through the Cue/PFL target",
        scale,
    );
    let second_slot: Element<'a, Message> = if matches!(cid, ChannelId::Output(_)) {
        cue_btn.into()
    } else {
        solo_btn.into()
    };
    let rows = column![
        rotated_label(
            if p.name_custom {
                p.name.to_string()
            } else {
                short_label(&p.name).to_string()
            },
            theme::TEXT_SM * scale,
            scale,
        ),
        mute_btn,
        second_slot,
        container(vu_meter(
            p.meter,
            p.meter2,
            p.meter2_available,
            COLLAPSED_METER_H * scale,
            scale,
            p.meter_available,
            p.meter_bus.clone().zip(p.meter_slot),
            p.meter_trim,
            if p.active_surface {
                theme::METER_WELL_ACTIVE
            } else {
                theme::METER_WELL
            },
        ))
            .width(Length::Fill)
            .center_x(Length::Fill),
        expand_btn,
    ]
    .spacing(theme::SPACE_HAIRLINE)
    .width(Length::Fill)
    .align_x(iced::Alignment::Center);

    let content = container(rows)
        .padding(iced::Padding {
            top: theme::SPACE_SM * p.scale,
            right: theme::SPACE_MD * p.scale,
            bottom: 0.0,
            left: theme::SPACE_MD * p.scale,
        })
        .width(Length::Fill)
        .height(Length::Fixed((STRIP_H - SUBMIX_H) * p.scale));
    // B2-sized display face: spans the collapsed card exactly like the
    // normal channel's submix selector, but carries no control yet.
    let footer = container(iced::widget::Space::new())
        .style(theme::display_face)
        .width(Length::Fill)
        .height(Length::Fixed(SUBMIX_H * p.scale));
    let card = column![content, footer]
        .width(Length::Fill)
        .height(Length::Fixed(STRIP_H * p.scale));

    mouse_area(
        container(card)
            .style(theme::strip_panel(
                p.selected,
                p.hovered,
                p.type_tag.map(|(_, c)| c),
                p.active_surface,
            ))
            .width(Length::Fixed(w * p.scale))
            .height(Length::Fixed(STRIP_H * p.scale))
            .clip(true),
    )
    .on_press(Message::StripClicked(cid))
    .on_double_click(Message::ToggleCollapse(cid))
    .on_enter(Message::StripHovered(Some(cid)))
    .on_exit(Message::StripHovered(None))
    .into()
}

/// Picks between the two strip layouts and, while a collapse/expand
/// animation is in flight, the width the outer card should be drawn at
/// this frame. The full (uncollapsed) content is shown not just when
/// resting expanded but for the whole transition in *either* direction —
/// shrinking, it's the thing visibly getting clipped down to
/// `COLLAPSED_W`; growing, it's what's being revealed. Only once a
/// collapse has fully settled does rendering switch to the lighter,
/// control-free `collapsed_strip`.
pub fn strip<'a>(p: StripParams<'a>) -> Element<'a, Message> {
    let now = Instant::now();
    let (w, show_full) = match &p.collapse_anim {
        Some(a) => (a.at(now), a.is_settling(now) || !p.collapsed),
        None => {
            if p.collapsed {
                (COLLAPSED_W, false)
            } else {
                (full_width(p.cid), true)
            }
        }
    };

    if show_full {
        full_strip(p)
    } else {
        collapsed_strip(p, w)
    }
}

fn full_strip<'a>(p: StripParams<'a>) -> Element<'a, Message> {
    let cid = p.cid;
    let out = p.output_idx;
    let scale = p.scale;
    let kind = match cid {
        ChannelId::Input(_) => ChannelKind::Input,
        ChannelId::Playback(_) => ChannelKind::Playback,
        ChannelId::Output(_) => ChannelKind::Output,
    };
    let template_id = match kind {
        ChannelKind::Input => {
            if p.stereo_linked {
                TemplateId::InputStereo
            } else {
                TemplateId::InputMono
            }
        }
        ChannelKind::Playback => TemplateId::PlaybackStereo,
        ChannelKind::Output => {
            if p.stereo_linked {
                TemplateId::OutputStereo
            } else {
                TemplateId::OutputMono
            }
        }
    };
    let layout = p
        .layout
        .template(template_id)
        .and_then(|template| p.layout.layout(template, scale).ok())
        .unwrap_or_else(|| {
            StripLayout::for_kind(
                kind,
                scale,
                Features {
                    fx: p.fx_level.is_some(),
                    ..Features::default()
                },
            )
        });
    let card = layout.card;
    let btn_h = layout
        .rect(ElementPlacement::ButtonLeft)
        .map_or(0.0, |rect| rect.height);

    let header = header_row(
        cid,
        &p.name,
        p.name_custom,
        p.type_tag,
        p.renameable,
        p.renaming,
        p.rename_buf,
        scale,
    );

    let mute_btn = hint(
        button(centered_master_label(
            "M",
            theme::TEXT_SM * scale,
            scale,
            theme::mute_color(),
            p.mute_armed,
            true,
        ))
            .width(Length::Fill)
            .height(btn_h)
            .style(theme::armed_outline_toggle(p.mute, p.mute_armed, theme::mute_color()))
            .on_press(Message::Mute(cid, !p.mute)),
        if p.mute_armed {
            "Mute (held by the M master; its next press brings it back)"
        } else {
            "Mute"
        },
        scale,
    );
    let solo_btn = hint(
        button(centered_master_label(
            if p.pfl { "PFL" } else { "S" },
            if p.pfl || p.solo_armed {
                theme::TEXT_XS * scale
            } else {
                theme::TEXT_SM * scale
            },
            scale,
            theme::solo_color(),
            !p.pfl && p.solo_armed,
            false,
        ))
        .width(Length::Fill)
        .height(btn_h)
        .style(theme::armed_outline_toggle(p.solo, p.solo_armed, theme::solo_color()))
        .on_press(Message::Solo(cid, !p.solo)),
        if p.pfl {
            "PFL (pre-fader listen) to the Cue/PFL target"
        } else if p.solo_armed {
            "Solo (held by the S master; its next press brings it back)"
        } else {
            "Solo"
        },
        scale,
    );
    let cue_btn = hint(
        button(centered_label("CUE", theme::TEXT_SM * scale))
            .width(Length::Fill)
            .height(btn_h)
            .style(theme::cue_button(p.cue))
            .on_press(Message::CueChanged(cid, !p.cue)),
        "CUE — exclusively monitor this output's submix through the Cue/PFL target",
        scale,
    );
    // Inputs and software playbacks pair M with S. Outputs, including the
    // Control Room channels, pair M with CUE; an output never shows M alone.
    let second_button = if matches!(cid, ChannelId::Output(_)) {
        cue_btn
    } else {
        solo_btn
    };

    // Pan sits directly under the header.
    //
    // Outputs get this row too.  On the UC it is a real output balance:
    // the two physical channels carry different 0x1a levels, and the knob
    // is derived from their difference.  Backends without a balance path
    // keep the knob dimmed (the same footprint, so every strip stays the
    // same height).
    let pan_str = match p.pan.cmp(&0) {
        std::cmp::Ordering::Less => format!("L{}", -p.pan),
        std::cmp::Ordering::Greater => format!("R{}", p.pan),
        std::cmp::Ordering::Equal => "C".to_string(),
    };
    let is_output = matches!(cid, ChannelId::Output(_));
    let balance_available = !is_output || p.output_balance_supported;
    let pan_knob = Knob {
        value: if balance_available { p.pan as f32 } else { 0.0 },
        range: (-100.0, 100.0),
        label: if balance_available {
            pan_str
        } else {
            "C".to_string()
        },
        arc_from_center: true,
        level_arc_color: Some(theme::PHANTOM),
        interactive: balance_available,
        log_scale: false,
        modifiers: p.modifiers,
        scale,
        on_change: if balance_available {
            Box::new(move |v| Message::PanChanged(cid, out, v.round() as i8))
        } else {
            Box::new(|_| unreachable!("balance unavailable suppresses all input"))
        },
        on_reset: if balance_available {
            Box::new(move || Message::PanReset(cid, out))
        } else {
            Box::new(|| unreachable!("balance unavailable suppresses all input"))
        },
    };
    let pan_hint = if is_output {
        if p.output_balance_supported {
            "Output balance — drag, scroll, Shift: fine, double-click to reset"
        } else {
            "Output balance — not implemented for this output/backend"
        }
    } else {
        "Pan — drag, scroll, Shift: fine, double-click to reset"
    };
    let pan_element = hint(knob(pan_knob), pan_hint, scale);
    // 48V/PAD/Sensitivity/Gain all live in the gear flyout now (see
    // `app.rs::settings_popover`), not inline here — keeps every strip's
    // own height fixed regardless of channel type, and matches the route
    // panel's own "opens beside the strip" shape instead of an inline
    // accordion that used to grow/shrink the card itself. Gain used to be
    // the one holdout (a `Knob` placed inside the old Stack-based flyout
    // broke click handling for the whole row), but the flyout isn't a
    // `Stack` anymore — it pushes the row instead of overlaying it — so
    // that failure mode doesn't apply and Gain moved in with the rest.

    let default_vol = p.default_vol;
    let fader_h = layout
        .rect(ElementPlacement::Fader)
        .map_or(0.0, |rect| rect.height);
    let fader_widget = fader(Fader {
        value: p.vol,
        range: p.drag_range.unwrap_or((0.0, 2.0)),
        default_value: default_vol,
        meter: p.meter,
        meter_available: p.meter_available,
        meter2: p.meter2,
        meter2_available: p.meter2_available,
        bus: p.meter_bus.clone().zip(p.meter_slot),
        meter_trim: p.meter_trim,
        height: fader_h,
        top_inset: 0.0,
        show_meter: true,
        fill: theme::fader_fill(p.active_surface),
        slot_w: CHANNEL_SLOT_W,
        meter_bg: if p.active_surface {
            theme::METER_WELL_ACTIVE
        } else {
            theme::METER_WELL
        },
        // The explicit columns below replace the old reserved-right
        // derivation; the strip layout owns the horizontal placement.
        reserved_right: 0.0,
        columns: Some(FaderColumns {
            meter_x: 0.0,
            track_x: 26.0,
        }),
        modifiers: p.modifiers,
        scale,
        show_track: true,
        label: None,
        label_color: theme::text_sec(),
        // Unused: only matters when `show_meter: false` (see its own
        // doc comment) — this strip fader is never that.
        compact_width: 0.0,
        on_press: Box::new(move |v, range| Message::FaderPressed(cid, out, v, range)),
        on_drag: Box::new(move |v| Message::VolumeChanged(cid, out, v)),
        on_release: Box::new(move || Message::RangeCleared(cid)),
        on_reset: Box::new(move || Message::Reset(cid, out, default_vol)),
        on_select: Some(Box::new(move || Message::StripClicked(cid))),
        cap_tint: p.fader_group,
        click_selects: p.fader_select_only,
    });

    // The four tab slots are fixed.  A slot with no control is still
    // reserved; its face does not extend to the edge.
    let tabs: [Rect; 4] = layout.stack_tabs.map(|(glyph, _)| glyph);
    let faces: [Rect; 4] = layout.stack_tabs.map(|(_, face)| face);
    let trim_open = p.open_flyout == Some(FlyoutKind::Trim);
    let trim_el: Element<'a, Message> = if p.has_trim {
        slideout_tab(
            "T",
            trim_open,
            p.trim < 0.0,
            Message::ToggleFlyout(cid, FlyoutKind::Trim),
            if trim_open {
                std::borrow::Cow::Borrowed("Hide trim")
            } else {
                std::borrow::Cow::Owned(p.trim_tip.clone())
            },
            scale,
            tabs[0].width,
            faces[0].width,
            tabs[0].height,
            theme::TEXT_MD,
        )
    } else {
        empty_tab(tabs[0].width, tabs[0].height)
    };
    let settings_open = p.open_flyout == Some(FlyoutKind::Settings);
    let gear_el = slideout_tab(
        "⚙",
        settings_open,
        false,
        Message::ToggleFlyout(cid, FlyoutKind::Settings),
        if settings_open {
            "Hide settings"
        } else {
            "Show settings (48V/PAD/Sensitivity)"
        },
        scale,
        tabs[1].width,
        faces[1].width,
        tabs[1].height,
        SETTINGS_GLYPH,
    );
    let eq_open = p.open_flyout == Some(FlyoutKind::Eq);
    let eq_el: Element<'a, Message> = if p.has_eq && p.protocol_features {
        slideout_tab(
            "EQ",
            eq_open,
            p.eq_enabled,
            Message::ToggleFlyout(cid, FlyoutKind::Eq),
            if eq_open {
                "Hide EQ"
            } else {
                "Show EQ (3-band + low cut)"
            },
            scale,
            tabs[2].width,
            faces[2].width,
            tabs[2].height,
            theme::TEXT_MD,
        )
    } else {
        empty_tab(tabs[2].width, tabs[2].height)
    };
    let dyn_open = p.open_flyout == Some(FlyoutKind::Dynamics);
    let dyn_el: Element<'a, Message> = if p.has_dynamics && p.protocol_features {
        slideout_tab(
            "D",
            dyn_open,
            p.dynamics_on,
            Message::ToggleFlyout(cid, FlyoutKind::Dynamics),
            if dyn_open {
                "Hide Dynamics"
            } else {
                "Show Dynamics (Compressor, Expander, Auto Level)"
            },
            scale,
            tabs[3].width,
            faces[3].width,
            tabs[3].height,
            theme::TEXT_MD,
        )
    } else {
        empty_tab(tabs[3].width, tabs[3].height)
    };
    // The FX fader: send on inputs and playbacks, return on outputs. A thin
    // track beside the main fader, no meter, its own value drag — measured
    // from the TotalMix strip (small cap, no scale of its own).
    // The FX fader: send on inputs and playbacks, return on outputs. A thin
    // track to the right of the main fader, starting at the same height,
    // with a short line under it — orange for a send, green for a return.
    let fx_rect = layout.rect(ElementPlacement::Fx);
    let fx_fader: Option<Element<'a, Message>> = p.fx_level.map(move |db| {
        let value = 10f32.powf(db / 20.0).clamp(0.0, 2.0);
        let inner = fader(Fader {
            value,
            range: (0.0, 1.0),
            default_value: 10f32.powf(-65.0 / 20.0),
            meter: MeterFrame::still(0.0),
            meter_available: false,
            meter2: None,
            meter2_available: false,
            bus: None,
            meter_trim: 0.0,
            // Leaves the T / gear / EQ / D buttons their room at the
            // bottom of the column.
            // 20 % taller than the earlier track: the FX send/return is
            // the only small fader on the strip, and the extra travel
            // matches the main fader's own feel.  Every strip on an FX
            // unit grows by the same amount, so the symbol grid below it
            // keeps its shared height.
            height: fx_rect.map_or(0.0, |r| r.height),
            top_inset: 0.0,
            show_meter: false,
            // Same dark slot as the main fader, but a smaller body.
            fill: match cid {
                ChannelId::Output(_) => theme::mgreen(),
                _ => theme::PHANTOM,
            },
            slot_w: FX_SLOT_W,
            meter_bg: theme::METER_WELL,
            reserved_right: 0.0,
            columns: None,
            modifiers: p.modifiers,
            // A smaller body: the cap and rail scale down with it.
            scale: scale * FX_FADER_SCALE,
            show_track: true,
            label: None,
            label_color: theme::text_sec(),
            compact_width: 26.0,
            on_press: Box::new(move |v, _| fx_message(cid, db_of(v))),
            on_drag: Box::new(move |v| fx_message(cid, db_of(v))),
            on_release: Box::new(move || Message::RangeCleared(cid)),
            on_reset: Box::new(move || fx_message(cid, -65.0)),
            on_select: Some(Box::new(move || Message::StripClicked(cid))),
            cap_tint: None,
            click_selects: p.fader_select_only,
        });
        // The fader's own coloured fill says everything; the dark channel
        // behind it is what marks the slot above the cap.
        column![container(inner).width(Length::Fixed(26.0 * 0.62 * scale))]
            .align_x(iced::Alignment::Center)
            .into()
    });

    let db_display: Element<'a, Message> = if p.editing {
        text_input("", p.edit_buf)
            .id(crate::app::DB_INPUT_ID)
            .on_input(Message::EditChanged)
            .on_submit(Message::EditCommit)
            .style(theme::text_input)
            .size(theme::TEXT_SM * scale)
            .width(Length::Fixed(
                layout
                    .rect(ElementPlacement::Level)
                    .map_or(0.0, |rect| rect.width),
            ))
            .into()
    } else {
        let initial = if p.vol > 0.0 {
            format!("{:.1}", 20.0 * p.vol.log10())
        } else {
            "-inf".into()
        };
        mouse_area(
            text(if p.vol > 0.0 {
                format!("{:.1}", 20.0 * p.vol.log10())
            } else {
                "-∞".into()
            })
                .color(theme::text_primary())
                .size(theme::TEXT_XS * scale)
                .wrapping(advanced_text::Wrapping::None),
        )
        .on_double_click(Message::EditStart(cid, initial))
        .into()
    };
    let db_display = container(db_display)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    // B1: minimize button at the right edge of the dB readout row.
    let collapse_btn = minimize_button(
        cid,
        scale,
        layout
            .rect(ElementPlacement::Minimize)
            .map_or(0.0, |rect| rect.width),
        layout
            .face(ElementPlacement::Minimize)
            .map_or(0.0, |rect| rect.width),
        layout
            .face(ElementPlacement::Minimize)
            .map_or(0.0, |rect| rect.height),
    );
    // B2: full-width submix selector; the flyout opens from the right edge.
    let route_button = layout.rect(ElementPlacement::Submix).map(|rect| {
        submix_button(
            cid,
            p.bus_label.clone(),
            p.open_flyout == Some(FlyoutKind::Route),
            scale,
            rect.width,
            rect.height,
        )
    });

    // Double-click anywhere on the card that isn't already claimed by a
    // specific control (the fader/pan canvases capture their own
    // double-click for reset-to-default, the dB readout for its edit
    // field, buttons for their own press) collapses the strip — a bigger,
    // more discoverable target than the tiny "-" button alone. A plain
    // click there is a no-op; Ctrl/Shift+click toggles multi-selection
    // (see `Message::StripClicked`) — mute/solo/collapse on any selected
    // strip then apply to the whole selection at once.
    let left_strength = p.meter_available.then_some(p.readout[0]).flatten();
    let right_strength = p
        .meter2_available
        .then_some(p.readout[1])
        .flatten();
    let mut stack = Stack::new()
        .width(Length::Fixed(card.width))
        .height(Length::Fixed(card.height));
    if let Some(rect) = layout.rect(ElementPlacement::Name) {
        stack = stack.push(pinned(header, rect, card));
    }
    if let Some(rect) = layout.rect(ElementPlacement::Pan) {
        stack = stack.push(pinned(pan_element, rect, card));
    }
    if let Some(rect) = layout.rect(ElementPlacement::ButtonLeft) {
        stack = stack.push(pinned(mute_btn, rect, card));
    }
    if let Some(rect) = layout.rect(ElementPlacement::ButtonRight) {
        stack = stack.push(pinned(second_button, rect, card));
    }
    if let Some(rect) = layout.rect(ElementPlacement::ReadoutLeft) {
        stack = stack.push(pinned(
            strength_cell(left_strength, rect.width, rect.height, scale),
            rect,
            card,
        ));
    }
    if let Some(rect) = layout.rect(ElementPlacement::ReadoutRight) {
        stack = stack.push(pinned(
            strength_cell(right_strength, rect.width, rect.height, scale),
            rect,
            card,
        ));
    }
    if layout.rect(ElementPlacement::Fader).is_some()
        || layout.rect(ElementPlacement::Meter).is_some()
    {
        stack = stack.push(pinned(fader_widget, layout.fader_canvas, card));
    }
    if let (Some(fx), Some(rect)) = (fx_fader, fx_rect) {
        stack = stack.push(pinned(fx, rect, card));
    }
    for (panel, element) in [
        (Panel::Trim, trim_el),
        (Panel::Settings, gear_el),
        (Panel::Eq, eq_el),
        (Panel::Dynamics, dyn_el),
    ] {
        if layout.rect(ElementPlacement::Stack(panel)).is_some() {
            stack = stack.push(pinned(element, faces[panel.stack_index()], card));
        }
    }
    if let Some(rect) = layout.rect(ElementPlacement::Level) {
        stack = stack.push(pinned(db_display, rect, card));
    }
    if let Some(face) = layout.face(ElementPlacement::Minimize) {
        stack = stack.push(pinned(collapse_btn, face, card));
    }
    if let (Some(route), Some(rect)) = (route_button, layout.rect(ElementPlacement::Submix)) {
        stack = stack.push(pinned(route, rect, card));
    }

    mouse_area(
        container(stack)
            .style(theme::strip_panel(
                p.selected,
                p.hovered,
                p.type_tag.map(|(_, c)| c),
                p.active_surface,
            ))
            .width(Length::Fixed(card.width))
            .height(Length::Fixed(card.height))
            .clip(true),
    )
    .on_press(Message::StripClicked(cid))
    .on_double_click(Message::ToggleCollapse(cid))
    .on_enter(Message::StripHovered(Some(cid)))
    .on_exit(Message::StripHovered(None))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    #[test]
    fn a_dead_meter_reads_silence_not_a_denormal_tail() {
        // The UCX reports down to -150.5 dBFS (its smallest non-silent
        // power word).  Readings below that are the bar's decay tail, which
        // used to print -122.0 and -900.0; the useful range must survive,
        // and magnitudes of 100 dB or more lose the fraction.
        assert_eq!(strength_text(10f32.powf(-150.4 / 20.0)), "-150");
        assert_eq!(strength_text(10f32.powf(-120.0 / 20.0)), "-120");
        assert_eq!(strength_text(10f32.powf(-99.9 / 20.0)), "-99.9");
        assert_eq!(strength_text(10f32.powf(-151.0 / 20.0)), "-∞");
        assert_eq!(strength_text(1.0e-40), "-∞");
        assert_eq!(strength_text(0.0), "-∞");
        assert_eq!(strength_text(10f32.powf(-80.0 / 20.0)), "-80.0");
        assert_eq!(strength_text(10f32.powf(-6.0 / 20.0)), "-6.0");
    }

    #[test]
    fn collapsed_strip_fills_the_standard_strip_height() {
        assert!((COLLAPSED_METER_H + COLLAPSED_CHROME_H - STRIP_H).abs() < f32::EPSILON);
        assert!(COLLAPSED_METER_H > 0.0);
    }

    #[test]
    fn preview_visual_tokens_match_the_gui() {
        let visual = totalcontrol_layout::VISUAL;
        assert_eq!(theme::TEXT_MD, visual.name_size);
        assert_eq!(theme::TEXT_SM, visual.button_text_size);
        assert_eq!(theme::TEXT_XS, visual.readout_size);
        assert_eq!(theme::TEXT_MICRO, visual.micro_size);
        assert_eq!(SETTINGS_GLYPH, visual.gear_glyph_size);
        let close = |a: f32, b: f32| assert!((a - b).abs() < 1e-4, "{a} vs {b}");
        close(crate::widgets::fader::CHANNEL_SLOT_W, visual.fader_slot_w);
        close(
            crate::widgets::fader::FX_SLOT_W * crate::widgets::fader::FX_FADER_SCALE,
            visual.fader_slot_w,
        );
        close(crate::widgets::fader::CAP_W, visual.fader_cap_w);
        close(crate::widgets::fader::CAP_H, visual.fader_cap_h);
        close(
            crate::widgets::fader::CAP_W * crate::widgets::fader::FX_FADER_SCALE,
            visual.fx_cap_w,
        );
        close(
            crate::widgets::fader::CAP_H * crate::widgets::fader::FX_FADER_SCALE,
            visual.fx_cap_h,
        );
        close(
            crate::widgets::knob::DIAMETER,
            visual.knob_diameter,
        );
    }

    #[test]
    fn collapsed_expand_button_sits_on_the_full_strip_b1_line() {
        let full = StripLayout::for_kind(ChannelKind::Input, 1.0, Features::default());
        let plus_y = theme::SPACE_SM
            + ROTATED_LABEL_LEN
            + theme::SPACE_HAIRLINE
            + ICON_BTN_H * 2.0
            + theme::SPACE_HAIRLINE * 2.0
            + COLLAPSED_METER_H
            + theme::SPACE_HAIRLINE;
        let level = full
            .rect(ElementPlacement::Level)
            .expect("the built-in layout has a level readout");
        assert!((plus_y - level.y).abs() < f32::EPSILON);
        let bottom = plus_y
            + ICON_BTN_H
            + theme::SPACE_HAIRLINE
            + SUBMIX_H;
        assert!((bottom - STRIP_H).abs() < f32::EPSILON);
    }

    #[test]
    fn compact_icon_column_leaves_room_for_the_fader_group() {
        let l = StripLayout::for_kind(ChannelKind::Input, 1.0, Features::default());
        assert!(l.fader_canvas.width >= crate::widgets::fader::FADER_GROUP_W);
        assert!(l.fader_canvas.x + l.fader_canvas.width <= l.stack_tabs[0].0.x);
    }

    #[test]
    fn settled_anim_is_never_settling() {
        let a = CollapseAnim { prev: STRIP_W, value: STRIP_W, since: Instant::now() };
        assert!(!a.is_settling(a.since));
        assert!(!a.is_settling(a.since + Duration::from_millis(10)));
    }

    #[test]
    fn transitioning_anim_settles_after_its_width_based_duration() {
        let a = CollapseAnim { prev: STRIP_W, value: COLLAPSED_W, since: Instant::now() };
        let duration = Duration::from_millis(a.duration_ms() as u64);
        assert!(a.is_settling(a.since), "just started — should still be settling");
        assert!(
            a.is_settling(a.since + duration / 2),
            "mid-transition — should still be settling"
        );
        assert!(
            !a.is_settling(a.since + duration + Duration::from_millis(1)),
            "past the width-based window — should have stopped requesting redraws"
        );
    }

    #[test]
    fn at_interpolates_linearly_from_prev_to_value() {
        let a = CollapseAnim { prev: STRIP_W, value: COLLAPSED_W, since: Instant::now() };
        let duration = Duration::from_millis(a.duration_ms() as u64);
        assert_eq!(a.at(a.since), STRIP_W);
        assert_eq!(a.at(a.since + duration), COLLAPSED_W);
        let mid = a.at(a.since + duration / 2);
        let expected_mid = (STRIP_W + COLLAPSED_W) / 2.0;
        assert!(
            (mid - expected_mid).abs() < 0.5,
            "expected ~{expected_mid} at the midpoint, got {mid}"
        );
    }

    #[test]
    fn slideout_speed_does_not_depend_on_panel_width() {
        let small = CollapseAnim { prev: 0.0, value: 90.0, since: Instant::now() };
        let large = CollapseAnim { prev: 0.0, value: 180.0, since: Instant::now() };
        assert!(
            (large.duration_ms() / small.duration_ms() - 2.0).abs() < 1e-4,
            "a panel twice as wide must take twice as long"
        );
    }
}
