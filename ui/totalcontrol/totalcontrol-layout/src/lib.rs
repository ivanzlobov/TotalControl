//! Channel template schema and geometry for TotalControl.
//!
//! This crate is the shared design model for the channel strip.  It is
//! deliberately independent of iced and of the widget tree:
//!
//! - [`ChannelKind`] and [`Features`] choose a template;
//! - [`TemplateSpec`] is the small, named set of standard channels;
//! - [`StripLayout`] resolves a template to concrete rectangles at a zoom;
//! - the preview binary renders the same resolved rectangles without the
//!   application.
//!
//! The authoritative visual design remains
//! `docs/spec/TOTALCONTROL-GUI-SPEC.md` §2.8.  The tests here pin the
//! invariants so documentation, preview and runtime cannot drift silently.

use serde::{Deserialize, Serialize};

/// A rectangle in logical or scaled pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn centre_x(self) -> f32 {
        self.x + self.width / 2.0
    }

    pub fn centre_y(self) -> f32 {
        self.y + self.height / 2.0
    }

    pub fn contains(self, inner: Rect) -> bool {
        inner.x >= self.x
            && inner.y >= self.y
            && inner.right() <= self.right() + 0.01
            && inner.bottom() <= self.bottom() + 0.01
    }

    pub fn overlaps(self, other: Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Full strip width at zoom 1.0.
pub const STRIP_W: f32 = 90.0;
/// Full strip height at zoom 1.0.
pub const STRIP_H: f32 = 350.0;
/// Horizontal content inset at zoom 1.0.
pub const CONTENT_X: f32 = 6.0;
/// Content width at zoom 1.0.
pub const CONTENT_W: f32 = 78.0;
/// Top content inset at zoom 1.0.
pub const CONTENT_Y: f32 = 4.0;
/// Gap between stacked rows at zoom 1.0.
pub const ROW_GAP: f32 = 1.0;

pub const NAME_H: f32 = 13.0;
pub const PAN_X: f32 = 12.0;
pub const PAN_W: f32 = 54.0;
pub const PAN_H: f32 = 54.0;
pub const BUTTON_W: f32 = 38.0;
pub const BUTTON_GAP: f32 = 2.0;
pub const BUTTON_H: f32 = 18.0;
pub const READOUT_H: f32 = 18.0;
pub const MIDDLE_H: f32 = 205.0;
pub const DB_X: f32 = 20.0;
pub const DB_W: f32 = 38.0;
pub const DB_READOUT_H: f32 = 14.0;
pub const SUBMIX_H: f32 = 18.0;

pub const VU_X: f32 = 0.0;
pub const VU_W: f32 = 13.0;
/// A mono bar is centred and only wide enough to cover the level labels;
/// a linked pair still uses the full VU width split into two bars.
pub const MONO_BAR_RATIO: f32 = 0.60;
pub const TRACK_X: f32 = 26.0;
pub const TRACK_W: f32 = 26.0;
pub const FADER_CANVAS_W: f32 = 55.0;
pub const TAB_X: f32 = 56.0;
pub const TAB_W: f32 = 20.0;
pub const TAB_FACE_W: f32 = STRIP_W - (CONTENT_X + TAB_X);
pub const FX_W: f32 = 26.0 * 0.62;
pub const FX_H: f32 = 66.0;
pub const TAB_TOP_WITH_FX: f32 = FX_H + 8.0;
pub const TAB_H: f32 = 22.0;
pub const GEAR_H: f32 = 26.0;
pub const TAB_GAP: f32 = 6.0;
pub const COLLAPSE_X: f32 = 59.0;
pub const COLLAPSE_W: f32 = 19.0;

/// Logical geometry at zoom 1.0.  The defaults are the shipped GUI; a
/// configuration file may override any field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutMetrics {
    pub strip_w: f32,
    pub strip_h: f32,
    pub content_x: f32,
    pub content_w: f32,
    pub content_y: f32,
    pub row_gap: f32,
    pub name_h: f32,
    pub pan_x: f32,
    pub pan_w: f32,
    pub pan_h: f32,
    pub button_w: f32,
    pub button_gap: f32,
    pub button_h: f32,
    pub readout_h: f32,
    pub middle_h: f32,
    pub db_x: f32,
    pub db_w: f32,
    pub db_readout_h: f32,
    pub submix_h: f32,
    pub vu_x: f32,
    pub vu_w: f32,
    pub track_x: f32,
    pub track_w: f32,
    pub fader_canvas_w: f32,
    pub tab_x: f32,
    pub tab_w: f32,
    pub fx_w: f32,
    pub fx_h: f32,
    pub tab_top_with_fx: f32,
    pub tab_h: f32,
    pub gear_h: f32,
    pub tab_gap: f32,
    pub collapse_x: f32,
    pub collapse_w: f32,
}

impl Default for LayoutMetrics {
    fn default() -> Self {
        Self {
            strip_w: STRIP_W,
            strip_h: STRIP_H,
            content_x: CONTENT_X,
            content_w: CONTENT_W,
            content_y: CONTENT_Y,
            row_gap: ROW_GAP,
            name_h: NAME_H,
            pan_x: PAN_X,
            pan_w: PAN_W,
            pan_h: PAN_H,
            button_w: BUTTON_W,
            button_gap: BUTTON_GAP,
            button_h: BUTTON_H,
            readout_h: READOUT_H,
            middle_h: MIDDLE_H,
            db_x: DB_X,
            db_w: DB_W,
            db_readout_h: DB_READOUT_H,
            submix_h: SUBMIX_H,
            vu_x: VU_X,
            vu_w: VU_W,
            track_x: TRACK_X,
            track_w: TRACK_W,
            fader_canvas_w: FADER_CANVAS_W,
            tab_x: TAB_X,
            tab_w: TAB_W,
            fx_w: FX_W,
            fx_h: FX_H,
            tab_top_with_fx: TAB_TOP_WITH_FX,
            tab_h: TAB_H,
            gear_h: GEAR_H,
            tab_gap: TAB_GAP,
            collapse_x: COLLAPSE_X,
            collapse_w: COLLAPSE_W,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKind {
    Input,
    Playback,
    Output,
}

impl ChannelKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Playback => "playback",
            Self::Output => "output",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Features {
    /// The unit has an FX bus, so the thin MR1 fader exists.
    pub fx: bool,
    /// The channel has a Trim control in the MR2 slot.
    pub trim: bool,
    /// The channel has an EQ control in the MR4 slot.
    pub eq: bool,
    /// The channel has a Dynamics control in the MR5 slot.
    pub dynamics: bool,
}

impl Features {
    pub const fn all() -> Self {
        Self {
            fx: true,
            trim: true,
            eq: true,
            dynamics: true,
        }
    }

    pub const fn no_fx() -> Self {
        Self {
            fx: false,
            trim: true,
            eq: false,
            dynamics: false,
        }
    }
}

impl Default for Features {
    fn default() -> Self {
        Self::all()
    }
}

/// The named standard channels used by the preview and, eventually, the
/// runtime template compiler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateId {
    InputMono,
    InputStereo,
    InputMonoNoFx,
    PlaybackStereo,
    OutputStereo,
    OutputMono,
    ControlRoom,
}

impl TemplateId {
    pub fn label(self) -> &'static str {
        match self {
            Self::InputMono => "Input mono",
            Self::InputStereo => "Input stereo",
            Self::InputMonoNoFx => "Input mono, no FX",
            Self::PlaybackStereo => "Playback stereo",
            Self::OutputStereo => "Output stereo",
            Self::OutputMono => "Output mono",
            Self::ControlRoom => "Control Room",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSpec {
    pub id: TemplateId,
    pub kind: ChannelKind,
    pub features: Features,
    pub stereo: bool,
}

impl TemplateSpec {
    pub const fn new(id: TemplateId, kind: ChannelKind, features: Features, stereo: bool) -> Self {
        Self {
            id,
            kind,
            features,
            stereo,
        }
    }

    pub fn layout(self, scale: f32) -> StripLayout {
        StripLayout::for_kind(self.kind, scale, self.features)
    }
}

pub const TEMPLATES: &[TemplateSpec] = &[
    TemplateSpec::new(
        TemplateId::InputMono,
        ChannelKind::Input,
        Features::all(),
        false,
    ),
    TemplateSpec::new(
        TemplateId::InputStereo,
        ChannelKind::Input,
        Features::all(),
        true,
    ),
    TemplateSpec::new(
        TemplateId::InputMonoNoFx,
        ChannelKind::Input,
        Features::no_fx(),
        false,
    ),
    TemplateSpec::new(
        TemplateId::PlaybackStereo,
        ChannelKind::Playback,
        Features {
            fx: true,
            trim: false,
            eq: false,
            dynamics: false,
        },
        true,
    ),
    TemplateSpec::new(
        TemplateId::OutputStereo,
        ChannelKind::Output,
        Features {
            fx: true,
            trim: false,
            eq: true,
            dynamics: true,
        },
        true,
    ),
    TemplateSpec::new(
        TemplateId::OutputMono,
        ChannelKind::Output,
        Features {
            fx: true,
            trim: false,
            eq: true,
            dynamics: true,
        },
        false,
    ),
    TemplateSpec::new(
        TemplateId::ControlRoom,
        ChannelKind::Output,
        Features {
            fx: true,
            trim: false,
            eq: true,
            dynamics: true,
        },
        true,
    ),
];

/// Resolved geometry for one full channel template at one zoom.
#[derive(Clone, Debug, PartialEq)]
pub struct StripLayout {
    pub scale: f32,
    pub card: Rect,
    pub content: Rect,
    pub name: Rect,
    pub pan: Rect,
    pub buttons: [Rect; 2],
    pub readouts: [Rect; 2],
    pub vu: Rect,
    /// Full meter+grab+track canvas; `fader` is the track inside it.
    pub fader_canvas: Rect,
    pub fader: Rect,
    pub fx: Option<Rect>,
    /// Glyph columns for T, gear, EQ, D.
    pub tab_glyphs: [Rect; 4],
    /// Faces for T, gear, EQ, D; each reaches the card's right edge.
    pub tab_faces: [Rect; 4],
    pub db_readout: Rect,
    pub collapse_glyph: Rect,
    pub collapse_face: Rect,
    pub submix: Option<Rect>,
}

impl StripLayout {
    pub fn for_kind(kind: ChannelKind, scale: f32, features: Features) -> Self {
        Self::for_metrics(kind, scale, features, &LayoutMetrics::default())
    }

    /// Resolve one template against explicit logical metrics.  This is the
    /// entry point used by the JSON configuration loader and the preview.
    pub fn for_metrics(
        kind: ChannelKind,
        scale: f32,
        features: Features,
        m: &LayoutMetrics,
    ) -> Self {
        let r =
            |x: f32, y: f32, w: f32, h: f32| Rect::new(x * scale, y * scale, w * scale, h * scale);

        let content = r(
            m.content_x,
            m.content_y,
            m.content_w,
            m.strip_h - m.content_y,
        );
        let name = r(m.content_x, m.content_y, m.content_w, m.name_h);
        let pan_y = m.content_y + m.name_h + m.row_gap;
        let pan = r(m.content_x + m.pan_x, pan_y, m.pan_w, m.pan_h);
        let buttons_y = pan_y + m.pan_h + m.row_gap;
        let buttons = [
            r(m.content_x, buttons_y, m.button_w, m.button_h),
            r(
                m.content_x + m.button_w + m.button_gap,
                buttons_y,
                m.button_w,
                m.button_h,
            ),
        ];
        let readouts_y = buttons_y + m.button_h + m.row_gap;
        let readouts = [
            r(m.content_x, readouts_y, m.button_w, m.readout_h),
            r(
                m.content_x + m.button_w + m.button_gap,
                readouts_y,
                m.button_w,
                m.readout_h,
            ),
        ];
        let middle_y = readouts_y + m.readout_h + m.row_gap;
        let vu = r(m.content_x + m.vu_x, middle_y, m.vu_w, m.middle_h);
        let fader_canvas = r(m.content_x, middle_y, m.fader_canvas_w, m.middle_h);
        let fader = r(m.content_x + m.track_x, middle_y, m.track_w, m.middle_h);
        let fx = features.fx.then(|| {
            r(
                m.content_x + m.tab_x + (m.tab_w - m.fx_w) / 2.0,
                middle_y,
                m.fx_w,
                m.fx_h,
            )
        });
        let tab_y = middle_y + if features.fx { m.tab_top_with_fx } else { 0.0 };
        let tab_glyphs = [
            r(m.content_x + m.tab_x, tab_y, m.tab_w, m.tab_h),
            r(
                m.content_x + m.tab_x,
                tab_y + m.tab_h + m.tab_gap,
                m.tab_w,
                m.gear_h,
            ),
            r(
                m.content_x + m.tab_x,
                tab_y + m.tab_h + m.gear_h + 2.0 * m.tab_gap,
                m.tab_w,
                m.tab_h,
            ),
            r(
                m.content_x + m.tab_x,
                tab_y + 2.0 * m.tab_h + m.gear_h + 3.0 * m.tab_gap,
                m.tab_w,
                m.tab_h,
            ),
        ];
        let tab_face_w = m.strip_w - (m.content_x + m.tab_x);
        let tab_faces = tab_glyphs.map(|g| Rect::new(g.x, g.y, tab_face_w * scale, g.height));

        let db_y = middle_y + m.middle_h + m.row_gap;
        let db_readout = r(m.content_x + m.db_x, db_y, m.db_w, m.db_readout_h);
        let collapse_glyph = r(
            m.content_x + m.collapse_x,
            db_y,
            m.collapse_w,
            m.db_readout_h,
        );
        let collapse_face = Rect::new(
            collapse_glyph.x,
            collapse_glyph.y,
            (m.strip_w - (m.content_x + m.collapse_x)) * scale,
            collapse_glyph.height,
        );
        let submix = match kind {
            ChannelKind::Input | ChannelKind::Playback => {
                Some(r(0.0, m.strip_h - m.submix_h, m.strip_w, m.submix_h))
            }
            ChannelKind::Output => None,
        };

        Self {
            scale,
            card: r(0.0, 0.0, m.strip_w, m.strip_h),
            content,
            name,
            pan,
            buttons,
            readouts,
            vu,
            fader_canvas,
            fader,
            fx,
            tab_glyphs,
            tab_faces,
            db_readout,
            collapse_glyph,
            collapse_face,
            submix,
        }
    }

    /// Content centre line in this layout's coordinate space.
    pub fn centre_x(&self) -> f32 {
        self.content.centre_x()
    }

    /// All visible element rectangles, for preview and invariant checks.
    pub fn element_rects(&self) -> Vec<Rect> {
        self.elements().into_iter().map(|(_, rect)| rect).collect()
    }

    /// Visible elements with stable names, used by the invariant checks and
    /// the configuration diagnostics.
    pub fn elements(&self) -> Vec<(&'static str, Rect)> {
        let mut elements = vec![
            ("name", self.name),
            ("pan", self.pan),
            ("button_left", self.buttons[0]),
            ("button_right", self.buttons[1]),
            ("readout_left", self.readouts[0]),
            ("readout_right", self.readouts[1]),
            ("vu", self.vu),
            ("fader_canvas", self.fader_canvas),
            ("fader", self.fader),
            ("db_readout", self.db_readout),
            ("collapse_glyph", self.collapse_glyph),
            ("collapse_face", self.collapse_face),
        ];
        if let Some(fx) = self.fx {
            elements.push(("fx", fx));
        }
        for (index, glyph) in self.tab_glyphs.iter().enumerate() {
            elements.push((TAB_GLYPH_NAMES[index], *glyph));
            elements.push((TAB_FACE_NAMES[index], self.tab_faces[index]));
        }
        if let Some(submix) = self.submix {
            elements.push(("submix", submix));
        }
        elements
    }
}

const TAB_GLYPH_NAMES: [&str; 4] = [
    "tab_glyph_trim",
    "tab_glyph_settings",
    "tab_glyph_eq",
    "tab_glyph_dynamics",
];
const TAB_FACE_NAMES: [&str; 4] = [
    "tab_face_trim",
    "tab_face_settings",
    "tab_face_eq",
    "tab_face_dynamics",
];

/// Shared visual tokens used by the preview and checked against the GUI by
/// parity tests.  Keeping them here prevents the preview from quietly
/// shrinking or re-proportioning controls that were tuned live.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct VisualTokens {
    pub name_size: f32,
    pub button_text_size: f32,
    pub readout_size: f32,
    pub micro_size: f32,
    pub tab_glyph_size: f32,
    pub gear_glyph_size: f32,
    pub knob_diameter: f32,
    pub knob_label_size: f32,
    pub fader_slot_w: f32,
    pub fader_cap_w: f32,
    pub fader_cap_h: f32,
    pub fx_cap_w: f32,
    pub fx_cap_h: f32,
}

pub const VISUAL: VisualTokens = VisualTokens {
    name_size: 13.0,
    button_text_size: 11.0,
    readout_size: 10.0,
    micro_size: 8.0,
    tab_glyph_size: 13.0,
    gear_glyph_size: 14.0,
    knob_diameter: 42.0,
    knob_label_size: 15.0,
    fader_slot_w: 4.34,
    fader_cap_w: 22.0,
    fader_cap_h: 15.0,
    fx_cap_w: 13.64,
    fx_cap_h: 9.3,
};

/// Ruler labels in the VU channel, top to bottom.
pub const VU_LABELS: [(f32, &str); 6] = [
    (0.0, "0"),
    (-6.0, "6"),
    (-10.0, "10"),
    (-20.0, "20"),
    (-40.0, "40"),
    (-60.0, "60"),
];

/// A compact set of semantic colour categories for the preview.  The
/// runtime theme will adopt the same categories in its own focused change.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SurfaceColors {
    pub app_background: String,
    pub card: String,
    pub card_active: String,
    pub border: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TextColors {
    pub primary: String,
    pub secondary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ControlColors {
    pub button_face: String,
    pub knob_face: String,
    pub knob_border: String,
    pub fader_slot: String,
    pub fader_fill: String,
    pub cap_ridge: String,
    pub cap_centre: String,
    pub tab_face: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AccentColors {
    pub mute: String,
    pub solo: String,
    pub cue: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MeterColors {
    pub green: String,
    pub yellow: String,
    pub red: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ColorScheme {
    pub surface: SurfaceColors,
    pub text: TextColors,
    pub control: ControlColors,
    pub accent: AccentColors,
    pub meter: MeterColors,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            surface: SurfaceColors {
                app_background: "#18191d".into(),
                card: "#24252a".into(),
                card_active: "#3a3b42".into(),
                border: "#36373d".into(),
            },
            text: TextColors {
                primary: "#f2f3f5".into(),
                secondary: "#a3a5ab".into(),
            },
            control: ControlColors {
                button_face: "#191a1e".into(),
                knob_face: "#1a1b1f".into(),
                knob_border: "#36373d".into(),
                fader_slot: "#131417".into(),
                fader_fill: "#36373c".into(),
                cap_ridge: "#97989d".into(),
                cap_centre: "#4a4c52".into(),
                tab_face: "#202126".into(),
            },
            accent: AccentColors {
                mute: "#3d9df6".into(),
                solo: "#f0b429".into(),
                cue: "#3dd68c".into(),
            },
            meter: MeterColors {
                green: "#3dd68c".into(),
                yellow: "#d8c34a".into(),
                red: "#d94a4a".into(),
            },
        }
    }
}

/// A category omitted from a document takes the shipped scheme, so a single
/// token can be overridden on its own.
impl Default for SurfaceColors {
    fn default() -> Self {
        ColorScheme::default().surface
    }
}

impl Default for TextColors {
    fn default() -> Self {
        ColorScheme::default().text
    }
}

impl Default for ControlColors {
    fn default() -> Self {
        ColorScheme::default().control
    }
}

impl Default for AccentColors {
    fn default() -> Self {
        ColorScheme::default().accent
    }
}

impl Default for MeterColors {
    fn default() -> Self {
        ColorScheme::default().meter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(kind: ChannelKind) -> StripLayout {
        StripLayout::for_kind(kind, 1.0, Features::default())
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 0.01, "expected {b}, got {a}");
    }

    #[test]
    fn rows_sum_to_the_standard_height() {
        let l = layout(ChannelKind::Input);
        approx(l.card.height, STRIP_H);
        approx(l.name.y, CONTENT_Y);
        approx(l.pan.y, 18.0);
        approx(l.buttons[0].y, 73.0);
        approx(l.readouts[0].y, 92.0);
        approx(l.vu.y, 111.0);
        approx(l.fader.y, 111.0);
        approx(l.db_readout.y, 317.0);
        approx(l.submix.unwrap().y, 332.0);
        approx(l.submix.unwrap().bottom(), STRIP_H);
    }

    #[test]
    fn vu_and_fader_share_top_bottom_height_and_centre() {
        let l = layout(ChannelKind::Input);
        approx(l.vu.y, l.fader.y);
        approx(l.vu.bottom(), l.fader.bottom());
        approx(l.vu.width, VU_W);
        approx(l.fader.width, TRACK_W);
        approx(l.fader_canvas.x, CONTENT_X);
        approx(l.fader_canvas.width, FADER_CANVAS_W);
        assert!(l.fader_canvas.contains(l.vu));
        assert!(l.fader_canvas.contains(l.fader));
        approx(l.fader.x, CONTENT_X + TRACK_X);
        assert!(l.vu.right() < l.fader.x);
        approx(l.fader.centre_x(), l.centre_x());
    }

    #[test]
    fn top_rows_and_readout_share_the_centre_line() {
        let l = layout(ChannelKind::Input);
        approx(l.name.centre_x(), l.centre_x());
        approx(l.pan.centre_x(), l.centre_x());
        approx((l.buttons[0].right() + l.buttons[1].x) / 2.0, l.centre_x());
        approx(
            (l.readouts[0].right() + l.readouts[1].x) / 2.0,
            l.centre_x(),
        );
        approx(l.db_readout.centre_x(), l.centre_x());
    }

    #[test]
    fn tab_faces_reach_the_right_edge_and_keep_the_glyph_centred() {
        let l = layout(ChannelKind::Input);
        for (glyph, face) in l.tab_glyphs.iter().zip(l.tab_faces.iter()) {
            approx(face.right(), STRIP_W);
            approx(glyph.centre_x(), face.x + TAB_W / 2.0);
            assert!(face.contains(*glyph));
        }
        approx(l.collapse_face.right(), STRIP_W);
        approx(
            l.collapse_glyph.centre_x(),
            l.collapse_face.x + COLLAPSE_W / 2.0,
        );
    }

    #[test]
    fn the_submix_bar_is_full_width_and_flush_with_the_bottom() {
        for kind in [ChannelKind::Input, ChannelKind::Playback] {
            let l = layout(kind);
            let s = l.submix.unwrap();
            approx(s.x, 0.0);
            approx(s.width, STRIP_W);
            approx(s.bottom(), STRIP_H);
        }
        assert!(layout(ChannelKind::Output).submix.is_none());
    }

    #[test]
    fn visible_elements_do_not_overlap() {
        let l = layout(ChannelKind::Input);
        let slots = vec![
            l.name,
            l.pan,
            l.buttons[0],
            l.buttons[1],
            l.readouts[0],
            l.readouts[1],
            l.vu,
            l.fader,
            l.db_readout,
            l.collapse_glyph,
            l.submix.unwrap(),
        ];
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                assert!(
                    !slots[i].overlaps(slots[j]),
                    "slots {} and {} overlap: {:?} / {:?}",
                    i,
                    j,
                    slots[i],
                    slots[j]
                );
            }
        }
    }

    #[test]
    fn every_element_stays_inside_the_card() {
        for kind in [
            ChannelKind::Input,
            ChannelKind::Playback,
            ChannelKind::Output,
        ] {
            let l = layout(kind);
            for slot in l.element_rects() {
                assert!(l.card.contains(slot), "{slot:?} leaves the card");
            }
        }
    }

    #[test]
    fn zoom_scales_every_rectangle() {
        let l = StripLayout::for_kind(ChannelKind::Input, 1.5, Features::default());
        approx(l.card.width, STRIP_W * 1.5);
        approx(l.card.height, STRIP_H * 1.5);
        approx(l.vu.height, MIDDLE_H * 1.5);
        approx(l.fader.centre_x(), l.centre_x());
        for (glyph, face) in l.tab_glyphs.iter().zip(l.tab_faces.iter()) {
            approx(face.right(), STRIP_W * 1.5);
            approx(glyph.centre_x(), face.x + TAB_W * 1.5 / 2.0);
        }
    }

    #[test]
    fn an_output_has_no_submix_and_no_trim_slot_in_the_spec() {
        let l = layout(ChannelKind::Output);
        assert!(l.fx.is_some());
        assert!(l.submix.is_none());
        approx(l.tab_glyphs[0].y, l.vu.y + FX_H + 8.0);
    }
}

impl Default for VisualTokens {
    fn default() -> Self {
        VISUAL
    }
}

/// Schema version of the shared GUI layout configuration.
pub const CONFIG_SCHEMA: u32 = 1;

/// Name the preview uses for the schema file written beside an example
/// document.
pub const SCHEMA_FILE_NAME: &str = "gui-layout.schema.json";

/// JSON Schema for [`GuiLayoutConfig`], embedded so the preview can write it
/// next to an example file and the tests can keep it in step with the
/// compiled defaults.
pub const SCHEMA_JSON: &str = include_str!("../../../../docs/schema/gui-layout.schema.json");

fn config_schema() -> u32 {
    CONFIG_SCHEMA
}

fn default_templates() -> Vec<TemplateSpec> {
    TEMPLATES.to_vec()
}

/// A `#rrggbb` colour literal; the preview writes the string straight into
/// SVG, so anything else is rejected rather than rendered as black.
fn is_hex_color(value: &str) -> bool {
    match value.strip_prefix('#') {
        Some(hex) => hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()),
        None => false,
    }
}

/// One JSON document for the complete GUI layout and its visual scheme.
///
/// The same document is consumed by the preview and, from the next runtime
/// step, by the GUI.  Missing fields fall back to the compiled defaults and
/// unknown fields are rejected.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct GuiLayoutConfig {
    /// Optional editor hint; it has no effect on rendering.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
    #[serde(default = "config_schema")]
    pub schema: u32,
    pub geometry: LayoutMetrics,
    pub visual: VisualTokens,
    pub colors: ColorScheme,
    #[serde(default = "default_templates")]
    pub templates: Vec<TemplateSpec>,
}

impl Default for GuiLayoutConfig {
    fn default() -> Self {
        Self {
            schema_url: None,
            schema: CONFIG_SCHEMA,
            geometry: LayoutMetrics::default(),
            visual: VISUAL,
            colors: ColorScheme::default(),
            templates: default_templates(),
        }
    }
}

impl GuiLayoutConfig {
    /// Starting document for the preview's `--example`; it points at the
    /// schema file written beside it.
    pub fn example() -> Self {
        Self {
            schema_url: Some(SCHEMA_FILE_NAME.to_string()),
            ..Self::default()
        }
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let config: Self =
            serde_json::from_str(text).map_err(|error| format!("invalid JSON: {error}"))?;
        config.validate()?;
        Ok(config)
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Self::from_json(&text)
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("layout config serializes")
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONFIG_SCHEMA {
            return Err(format!(
                "unsupported layout schema {} (expected {CONFIG_SCHEMA})",
                self.schema
            ));
        }
        let m = &self.geometry;
        let sizes = [
            ("strip_w", m.strip_w),
            ("strip_h", m.strip_h),
            ("content_w", m.content_w),
            ("name_h", m.name_h),
            ("pan_w", m.pan_w),
            ("pan_h", m.pan_h),
            ("button_w", m.button_w),
            ("button_h", m.button_h),
            ("readout_h", m.readout_h),
            ("middle_h", m.middle_h),
            ("db_w", m.db_w),
            ("db_readout_h", m.db_readout_h),
            ("submix_h", m.submix_h),
            ("vu_w", m.vu_w),
            ("track_w", m.track_w),
            ("fader_canvas_w", m.fader_canvas_w),
            ("tab_w", m.tab_w),
            ("fx_w", m.fx_w),
            ("fx_h", m.fx_h),
            ("tab_h", m.tab_h),
            ("gear_h", m.gear_h),
            ("collapse_w", m.collapse_w),
        ];
        for (name, value) in sizes {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("geometry.{name} must be a positive number"));
            }
        }
        let offsets = [
            ("content_x", m.content_x),
            ("content_y", m.content_y),
            ("row_gap", m.row_gap),
            ("pan_x", m.pan_x),
            ("button_gap", m.button_gap),
            ("db_x", m.db_x),
            ("vu_x", m.vu_x),
            ("track_x", m.track_x),
            ("tab_x", m.tab_x),
            ("tab_top_with_fx", m.tab_top_with_fx),
            ("tab_gap", m.tab_gap),
            ("collapse_x", m.collapse_x),
        ];
        for (name, value) in offsets {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("geometry.{name} must be zero or positive"));
            }
        }
        let tokens = [
            ("name_size", self.visual.name_size),
            ("button_text_size", self.visual.button_text_size),
            ("readout_size", self.visual.readout_size),
            ("micro_size", self.visual.micro_size),
            ("tab_glyph_size", self.visual.tab_glyph_size),
            ("gear_glyph_size", self.visual.gear_glyph_size),
            ("knob_diameter", self.visual.knob_diameter),
            ("knob_label_size", self.visual.knob_label_size),
            ("fader_slot_w", self.visual.fader_slot_w),
            ("fader_cap_w", self.visual.fader_cap_w),
            ("fader_cap_h", self.visual.fader_cap_h),
            ("fx_cap_w", self.visual.fx_cap_w),
            ("fx_cap_h", self.visual.fx_cap_h),
        ];
        for (name, value) in tokens {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("visual.{name} must be a positive number"));
            }
        }
        let c = &self.colors;
        let colors = [
            ("surface.app_background", &c.surface.app_background),
            ("surface.card", &c.surface.card),
            ("surface.card_active", &c.surface.card_active),
            ("surface.border", &c.surface.border),
            ("text.primary", &c.text.primary),
            ("text.secondary", &c.text.secondary),
            ("control.button_face", &c.control.button_face),
            ("control.knob_face", &c.control.knob_face),
            ("control.knob_border", &c.control.knob_border),
            ("control.fader_slot", &c.control.fader_slot),
            ("control.fader_fill", &c.control.fader_fill),
            ("control.cap_ridge", &c.control.cap_ridge),
            ("control.cap_centre", &c.control.cap_centre),
            ("control.tab_face", &c.control.tab_face),
            ("accent.mute", &c.accent.mute),
            ("accent.solo", &c.accent.solo),
            ("accent.cue", &c.accent.cue),
            ("meter.green", &c.meter.green),
            ("meter.yellow", &c.meter.yellow),
            ("meter.red", &c.meter.red),
        ];
        for (name, value) in colors {
            if !is_hex_color(value) {
                return Err(format!("colors.{name} must be a #rrggbb value"));
            }
        }
        if self.templates.is_empty() {
            return Err("templates must contain at least one template".into());
        }
        let mut ids = std::collections::HashSet::new();
        for template in &self.templates {
            if !ids.insert(template.id) {
                return Err(format!("duplicate template id {:?}", template.id));
            }
        }
        for template in &self.templates {
            let layout = self.layout(*template, 1.0);
            for (name, rect) in layout.elements() {
                if !layout.card.contains(rect) {
                    return Err(format!(
                        "template {:?}: the {} element leaves the {} x {} card",
                        template.id, name, m.strip_w, m.strip_h
                    ));
                }
            }
            let centred = |name: &str, rect: Rect, hint: &str| {
                if (rect.centre_x() - layout.centre_x()).abs() > 0.01 {
                    return Err(format!(
                        "template {:?}: {name} must sit on the content centre line, {hint}",
                        template.id
                    ));
                }
                Ok(())
            };
            centred(
                "geometry.pan_x and pan_w",
                layout.pan,
                "pan_x = (content_w - pan_w) / 2",
            )?;
            centred(
                "geometry.db_x and db_w",
                layout.db_readout,
                "db_x = (content_w - db_w) / 2",
            )?;
            centred(
                "geometry.track_x and track_w",
                layout.fader,
                "track_x = (content_w - track_w) / 2",
            )?;
            let pair = Rect::new(
                layout.buttons[0].x,
                layout.buttons[0].y,
                layout.buttons[1].right() - layout.buttons[0].x,
                layout.buttons[0].height,
            );
            centred(
                "geometry.button_w and button_gap",
                pair,
                "content_w = 2 * button_w + button_gap",
            )?;
        }
        Ok(())
    }

    pub fn layout(&self, template: TemplateSpec, scale: f32) -> StripLayout {
        StripLayout::for_metrics(template.kind, scale, template.features, &self.geometry)
    }

    pub fn template(&self, id: TemplateId) -> Option<TemplateSpec> {
        self.templates
            .iter()
            .copied()
            .find(|template| template.id == id)
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn the_example_config_is_valid_and_matches_the_built_in_defaults() {
        let text = include_str!("../../../../docs/examples/gui-layout.json");
        let config = GuiLayoutConfig::from_json(text).expect("example config loads");
        assert_eq!(config.schema, CONFIG_SCHEMA);
        assert_eq!(config.geometry, LayoutMetrics::default());
        assert_eq!(config.visual, VISUAL);
        assert_eq!(config.colors, ColorScheme::default());
        assert_eq!(config.templates, TEMPLATES);
    }

    #[test]
    fn a_geometry_override_reaches_the_resolved_layout() {
        let mut config = GuiLayoutConfig::default();
        config.geometry.strip_w = 100.0;
        config.geometry.strip_h = 360.0;
        let template = config.templates[0];
        let layout = config.layout(template, 1.0);
        assert_eq!(layout.card.width, 100.0);
        assert_eq!(layout.card.height, 360.0);
    }

    #[test]
    fn invalid_configs_are_rejected() {
        let mut config = GuiLayoutConfig::default();
        config.schema = 99;
        assert!(config.validate().is_err());

        let mut config = GuiLayoutConfig::default();
        config.geometry.middle_h = 0.0;
        assert!(config.validate().is_err());

        let mut config = GuiLayoutConfig::default();
        config.templates.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_round_trips_through_json() {
        let json = GuiLayoutConfig::default().to_pretty_json();
        let parsed = GuiLayoutConfig::from_json(&json).expect("round trip");
        assert_eq!(parsed, GuiLayoutConfig::default());
    }

    #[test]
    fn an_empty_document_uses_the_built_in_layout() {
        let config = GuiLayoutConfig::from_json("{}").expect("empty document loads");
        assert_eq!(config.geometry, LayoutMetrics::default());
        assert_eq!(config.visual, VISUAL);
        assert_eq!(config.colors, ColorScheme::default());
        assert_eq!(config.templates, TEMPLATES);
    }

    #[test]
    fn the_example_document_references_the_embedded_schema() {
        let config =
            GuiLayoutConfig::from_json(include_str!("../../../../docs/examples/gui-layout.json"))
                .expect("example config loads");
        let url = config.schema_url.expect("example names its schema");
        let example_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/examples")
            .canonicalize()
            .expect("docs/examples exists");
        let schema_path = example_dir.join(&url);
        assert!(
            schema_path.is_file(),
            "{} is missing",
            schema_path.display()
        );
        assert_eq!(
            std::fs::read_to_string(&schema_path).expect("schema reads"),
            SCHEMA_JSON,
            "the referenced schema and the embedded copy differ"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let text = r#"{"geometry": {"strip_w": 90.0, "stip_h": 350.0}}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("typo is rejected");
        assert!(error.contains("stip_h"), "{error}");

        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "features": {"fx": true, "trim": true, "eq": true, "dynamics": true, "extra": false}, "stereo": false}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("unknown feature is rejected");
        assert!(error.contains("extra"), "{error}");
    }

    #[test]
    fn offsets_visual_tokens_and_colours_are_validated() {
        let mut config = GuiLayoutConfig::default();
        config.geometry.tab_x = -1.0;
        let error = config.validate().expect_err("negative offset");
        assert!(error.contains("geometry.tab_x"), "{error}");

        let mut config = GuiLayoutConfig::default();
        config.visual.fader_slot_w = 0.0;
        let error = config.validate().expect_err("zero token");
        assert!(error.contains("visual.fader_slot_w"), "{error}");

        let mut config = GuiLayoutConfig::default();
        config.colors.accent.mute = "blue".into();
        let error = config.validate().expect_err("named colour");
        assert!(error.contains("colors.accent.mute"), "{error}");

        let mut config = GuiLayoutConfig::default();
        config.colors.meter.red = "#d94a4".into();
        assert!(config.validate().is_err(), "short hex colour");
    }

    #[test]
    fn partial_documents_override_single_fields() {
        let config = GuiLayoutConfig::from_json(r##"{"colors": {"accent": {"solo": "#ffcc00"}}}"##)
            .expect("partial colour document loads");
        assert_eq!(config.colors.accent.solo, "#ffcc00");
        assert_eq!(
            config.colors.accent.mute,
            ColorScheme::default().accent.mute
        );
        assert_eq!(config.colors.surface, SurfaceColors::default());
        assert_eq!(config.geometry, LayoutMetrics::default());

        let config = GuiLayoutConfig::from_json(r#"{"visual": {"knob_diameter": 46.0}}"#)
            .expect("partial visual document loads");
        assert_eq!(config.visual.knob_diameter, 46.0);
        assert_eq!(config.visual.fader_slot_w, VISUAL.fader_slot_w);
    }

    #[test]
    fn a_layout_that_leaves_the_card_is_rejected() {
        let mut config = GuiLayoutConfig::default();
        config.geometry.middle_h = 400.0;
        let error = config.validate().expect_err("taller middle overflows");
        assert!(error.contains("leaves the"), "{error}");

        let mut config = GuiLayoutConfig::default();
        config.geometry.track_x = 10.0;
        let error = config.validate().expect_err("off-centre track");
        assert!(error.contains("track_x"), "{error}");
    }

    /// The schema document is the editor-facing reference, so its property
    /// names and defaults must match the compiled model exactly.
    #[test]
    fn the_schema_document_matches_the_compiled_defaults() {
        let schema: serde_json::Value =
            serde_json::from_str(SCHEMA_JSON).expect("schema is valid JSON");
        let defaults = serde_json::to_value(GuiLayoutConfig::default()).expect("defaults");
        let properties = object_property(&schema, "properties");
        let mut documented: Vec<&str> = properties.keys().map(String::as_str).collect();
        documented.retain(|name| *name != "$schema");
        documented.sort_unstable();
        let mut compiled: Vec<&str> = defaults
            .as_object()
            .expect("defaults are an object")
            .keys()
            .map(String::as_str)
            .collect();
        compiled.sort_unstable();
        assert_eq!(documented, compiled, "top-level fields differ");

        for section in ["geometry", "visual", "colors"] {
            compare_schema_defaults(
                section,
                object_property(property(&properties, section), "properties"),
                &defaults[section],
            );
        }

        let template = value_property(property(&properties, "templates"), "items");
        let template_properties = object_property(template, "properties");
        let features = property(&template_properties, "features");
        let feature_properties = object_property(features, "properties");
        let mut documented: Vec<&str> = feature_properties.keys().map(String::as_str).collect();
        documented.sort_unstable();
        let mut required = string_list(value_property(features, "required"));
        required.sort_unstable();
        assert_eq!(
            documented, required,
            "feature fields and required list differ"
        );

        let mut documented = string_list(property(&template_properties, "id"));
        documented.sort_unstable();
        let mut compiled: Vec<String> = TEMPLATES
            .iter()
            .map(|spec| {
                serde_json::to_value(spec.id)
                    .expect("template id serializes")
                    .as_str()
                    .expect("template id is a string")
                    .to_string()
            })
            .collect();
        compiled.sort_unstable();
        assert!(!compiled.is_empty());
        assert_eq!(documented, compiled, "template ids differ");
    }

    fn property<'a>(
        object: &'a serde_json::Map<String, serde_json::Value>,
        key: &str,
    ) -> &'a serde_json::Value {
        object
            .get(key)
            .unwrap_or_else(|| panic!("schema is missing {key}"))
    }

    fn value_property<'a>(value: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
        value
            .get(key)
            .unwrap_or_else(|| panic!("schema is missing {key}"))
    }

    fn object_property<'a>(
        value: &'a serde_json::Value,
        key: &str,
    ) -> &'a serde_json::Map<String, serde_json::Value> {
        value
            .get(key)
            .and_then(|value| value.as_object())
            .unwrap_or_else(|| panic!("schema value is missing the {key} object"))
    }

    fn string_list(value: &serde_json::Value) -> Vec<String> {
        value
            .as_array()
            .or_else(|| {
                value
                    .get("enum")
                    .and_then(|enumeration| enumeration.as_array())
            })
            .unwrap_or_else(|| panic!("expected a string list, got {value}"))
            .iter()
            .map(|entry| entry.as_str().expect("string entry").to_string())
            .collect()
    }

    fn compare_schema_defaults(
        path: &str,
        properties: &serde_json::Map<String, serde_json::Value>,
        value: &serde_json::Value,
    ) {
        let object = value.as_object().expect("section is an object");
        let mut documented: Vec<&str> = properties.keys().map(String::as_str).collect();
        documented.sort_unstable();
        let mut compiled: Vec<&str> = object.keys().map(String::as_str).collect();
        compiled.sort_unstable();
        assert_eq!(documented, compiled, "{path}: fields differ");
        for (key, actual) in object {
            let property = property(properties, key);
            let full = format!("{path}.{key}");
            match actual {
                serde_json::Value::Object(_) => {
                    compare_schema_defaults(&full, object_property(property, "properties"), actual)
                }
                serde_json::Value::Number(number) => {
                    let documented = property
                        .get("default")
                        .and_then(|value| value.as_f64())
                        .unwrap_or_else(|| panic!("{full}: no documented default"));
                    assert_eq!(
                        documented as f32,
                        number.as_f64().expect("number") as f32,
                        "{full}: default differs"
                    );
                }
                serde_json::Value::String(text) => assert_eq!(
                    property.get("default").and_then(|value| value.as_str()),
                    Some(text.as_str()),
                    "{full}: default differs"
                ),
                other => panic!("{full}: unexpected default {other}"),
            }
        }
    }
}
