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

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

    pub const ALL: [Self; 7] = [
        Self::InputMono,
        Self::InputStereo,
        Self::InputMonoNoFx,
        Self::PlaybackStereo,
        Self::OutputStereo,
        Self::OutputMono,
        Self::ControlRoom,
    ];

    /// The document token for this template.
    pub fn token(self) -> &'static str {
        match self {
            Self::InputMono => "InputMono",
            Self::InputStereo => "InputStereo",
            Self::InputMonoNoFx => "InputMonoNoFx",
            Self::PlaybackStereo => "PlaybackStereo",
            Self::OutputStereo => "OutputStereo",
            Self::OutputMono => "OutputMono",
            Self::ControlRoom => "ControlRoom",
        }
    }

    /// Parse a document token or the display label.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|id| id.token() == name || id.label() == name)
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

/// A panel that opens beside the strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Panel {
    Trim,
    Settings,
    Eq,
    Dynamics,
}

impl Panel {
    pub const ALL: [Self; 4] = [Self::Trim, Self::Settings, Self::Eq, Self::Dynamics];

    pub fn label(self) -> &'static str {
        match self {
            Self::Trim => "Trim",
            Self::Settings => "Settings",
            Self::Eq => "EQ",
            Self::Dynamics => "Dynamics",
        }
    }

    /// Index in the right-hand stack, `MR2` to `MR5`.
    pub fn stack_index(self) -> usize {
        match self {
            Self::Trim => 0,
            Self::Settings => 1,
            Self::Eq => 2,
            Self::Dynamics => 3,
        }
    }
}

/// The logical position of a channel element in the strip grid.
///
/// Labels follow `TOTALCONTROL-GUI-SPEC.md` §2.8: `T` rows on top, `M` in the
/// middle and `B` at the bottom; `L`, `C` and `R` are the left, centre and
/// right columns.  The middle area has a readout row (`M1L`, `M1R`) and a
/// second row whose columns are the VU (`ML`), the fader (`MC`) and the
/// right-hand stack (`MR1` to `MR5`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElementPlacement {
    /// T1: channel name and type tag.
    Name,
    /// T2: pan or balance knob.
    Pan,
    /// TL3: left button.
    ButtonLeft,
    /// TR3: right button.
    ButtonRight,
    /// M1L: left channel readout.
    ReadoutLeft,
    /// M1R: right channel readout.
    ReadoutRight,
    /// ML: the VU element.
    Meter,
    /// MC: the channel fader.
    Fader,
    /// MR1: the FX send or return fader.
    Fx,
    /// MR2 to MR5: a tab in the right-hand stack.
    Stack(Panel),
    /// B1: the dB readout.
    Level,
    /// B1R: the minimize face.
    Minimize,
    /// B2: the submix bar of an input or playback.
    Submix,
}

impl ElementPlacement {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "T1",
            Self::Pan => "T2",
            Self::ButtonLeft => "TL3",
            Self::ButtonRight => "TR3",
            Self::ReadoutLeft => "M1L",
            Self::ReadoutRight => "M1R",
            Self::Meter => "ML",
            Self::Fader => "MC",
            Self::Fx => "MR1",
            Self::Stack(Panel::Trim) => "MR2",
            Self::Stack(Panel::Settings) => "MR3",
            Self::Stack(Panel::Eq) => "MR4",
            Self::Stack(Panel::Dynamics) => "MR5",
            Self::Level => "B1",
            Self::Minimize => "B1R",
            Self::Submix => "B2",
        }
    }

    pub const ALL: [Self; 16] = [
        Self::Name,
        Self::Pan,
        Self::ButtonLeft,
        Self::ButtonRight,
        Self::ReadoutLeft,
        Self::ReadoutRight,
        Self::Meter,
        Self::Fader,
        Self::Fx,
        Self::Stack(Panel::Trim),
        Self::Stack(Panel::Settings),
        Self::Stack(Panel::Eq),
        Self::Stack(Panel::Dynamics),
        Self::Level,
        Self::Minimize,
        Self::Submix,
    ];

    /// Parse the token used by the layout document.
    pub fn parse(token: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|placement| placement.label() == token)
    }
}

impl Serialize for ElementPlacement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}

impl<'de> Deserialize<'de> for ElementPlacement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let token = String::deserialize(deserializer)?;
        Self::parse(&token).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown placement {token:?}; expected one of {}",
                Self::ALL.map(|placement| placement.label()).join(", ")
            ))
        })
    }
}

/// A channel side, for the two-segment readouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelSide {
    Left,
    Right,
}

/// The action of a button element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Mute,
    Solo,
    Cue,
}

/// What a channel element is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElementKind {
    /// The channel name and type tag.
    Name,
    /// The pan or balance knob.
    Pan,
    /// A mute, solo or cue button.
    Button(Action),
    /// A channel-strength readout cell.
    Readout(ChannelSide),
    /// The VU element.
    Meter,
    /// The channel fader.
    Fader,
    /// The FX send or return fader.
    FxFader,
    /// A tab that opens a panel.
    Tab(Panel),
    /// The dB readout of the fader level.
    Level,
    /// The strip collapse face.
    Minimize,
    /// The submix selector.
    Submix,
}

impl ElementKind {
    /// What the element shows or controls.
    pub fn binding(self) -> Binding {
        match self {
            Self::Name => Binding::Name,
            Self::Pan => Binding::Pan,
            Self::Button(action) => Binding::Toggle(action),
            Self::Readout(side) => Binding::Strength(side),
            Self::Meter => Binding::Meter,
            Self::Fader => Binding::Level,
            Self::FxFader => Binding::Fx,
            Self::Tab(panel) => Binding::Panel(panel),
            Self::Level => Binding::Output,
            Self::Minimize => Binding::Collapse,
            Self::Submix => Binding::Submix,
        }
    }

    /// The token used by the layout document.
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Pan => "pan",
            Self::Button(Action::Mute) => "mute",
            Self::Button(Action::Solo) => "solo",
            Self::Button(Action::Cue) => "cue",
            Self::Readout(ChannelSide::Left) => "readout_left",
            Self::Readout(ChannelSide::Right) => "readout_right",
            Self::Meter => "meter",
            Self::Fader => "fader",
            Self::FxFader => "fx_fader",
            Self::Tab(Panel::Trim) => "trim_tab",
            Self::Tab(Panel::Settings) => "settings_tab",
            Self::Tab(Panel::Eq) => "eq_tab",
            Self::Tab(Panel::Dynamics) => "dynamics_tab",
            Self::Level => "level_readout",
            Self::Minimize => "minimize",
            Self::Submix => "submix",
        }
    }

    pub const ALL: [Self; 17] = [
        Self::Name,
        Self::Pan,
        Self::Button(Action::Mute),
        Self::Button(Action::Solo),
        Self::Button(Action::Cue),
        Self::Readout(ChannelSide::Left),
        Self::Readout(ChannelSide::Right),
        Self::Meter,
        Self::Fader,
        Self::FxFader,
        Self::Tab(Panel::Trim),
        Self::Tab(Panel::Settings),
        Self::Tab(Panel::Eq),
        Self::Tab(Panel::Dynamics),
        Self::Level,
        Self::Minimize,
        Self::Submix,
    ];

    /// Parse the token used by the layout document.
    pub fn parse(token: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.label() == token)
    }
}

impl Serialize for ElementKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}

impl<'de> Deserialize<'de> for ElementKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let token = String::deserialize(deserializer)?;
        Self::parse(&token).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown element kind {token:?}; expected one of {}",
                Self::ALL.map(|kind| kind.label()).join(", ")
            ))
        })
    }
}

/// The state a channel element is bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Binding {
    /// The channel name.
    Name,
    /// The pan or balance value.
    Pan,
    /// An on/off state.
    Toggle(Action),
    /// A channel-strength readout.
    Strength(ChannelSide),
    /// The live meter level.
    Meter,
    /// The send or output level the fader sets.
    Level,
    /// The FX send or return level.
    Fx,
    /// The panel the tab opens.
    Panel(Panel),
    /// The dB readout of the fader level.
    Output,
    /// The strip collapse.
    Collapse,
    /// The submix assignment.
    Submix,
}

/// One element of a channel template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelElement {
    pub kind: ElementKind,
    #[serde(rename = "at")]
    pub placement: ElementPlacement,
}

impl ChannelElement {
    pub const fn new(kind: ElementKind, placement: ElementPlacement) -> Self {
        Self { kind, placement }
    }

    /// Whether the kind belongs at this placement.  `TR3` accepts solo and
    /// cue; every other placement has exactly one kind.
    pub fn is_coherent(self) -> bool {
        use ElementKind as K;
        use ElementPlacement as P;
        if let (P::Stack(placed), K::Tab(panel)) = (self.placement, self.kind) {
            return placed == panel;
        }
        matches!(
            (self.placement, self.kind),
            (P::Name, K::Name)
                | (P::Pan, K::Pan)
                | (P::ButtonLeft, K::Button(Action::Mute))
                | (P::ButtonRight, K::Button(Action::Solo | Action::Cue))
                | (P::ReadoutLeft, K::Readout(ChannelSide::Left))
                | (P::ReadoutRight, K::Readout(ChannelSide::Right))
                | (P::Meter, K::Meter)
                | (P::Fader, K::Fader)
                | (P::Fx, K::FxFader)
                | (P::Level, K::Level)
                | (P::Minimize, K::Minimize)
                | (P::Submix, K::Submix)
        )
    }

    pub fn binding(self) -> Binding {
        self.kind.binding()
    }

    pub fn label(self) -> &'static str {
        self.placement.label()
    }
}

/// A channel template: which channel elements exist and where they sit.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelTemplate {
    pub spec: TemplateSpec,
    pub elements: Vec<ChannelElement>,
}

impl ChannelTemplate {
    /// The standard composition of one template.
    pub fn of(spec: TemplateSpec) -> Self {
        Self {
            spec,
            elements: standard_elements(spec.kind, spec.features),
        }
    }

    pub fn has_fx(&self) -> bool {
        self.elements
            .iter()
            .any(|element| element.kind == ElementKind::FxFader)
    }

    pub fn element(&self, placement: ElementPlacement) -> Option<&ChannelElement> {
        self.elements
            .iter()
            .find(|element| element.placement == placement)
    }
}

/// The elements of a standard template.
///
/// An absent feature leaves its row empty: the tab is missing from the list
/// but the rows below it keep their position.
pub fn standard_elements(kind: ChannelKind, features: Features) -> Vec<ChannelElement> {
    use ElementKind as K;
    use ElementPlacement as P;

    let button = match kind {
        ChannelKind::Output => Action::Cue,
        ChannelKind::Input | ChannelKind::Playback => Action::Solo,
    };
    let mut elements = vec![
        ChannelElement::new(K::Name, P::Name),
        ChannelElement::new(K::Pan, P::Pan),
        ChannelElement::new(K::Button(Action::Mute), P::ButtonLeft),
        ChannelElement::new(K::Button(button), P::ButtonRight),
        ChannelElement::new(K::Readout(ChannelSide::Left), P::ReadoutLeft),
        ChannelElement::new(K::Readout(ChannelSide::Right), P::ReadoutRight),
        ChannelElement::new(K::Meter, P::Meter),
        ChannelElement::new(K::Fader, P::Fader),
        ChannelElement::new(K::Level, P::Level),
        ChannelElement::new(K::Minimize, P::Minimize),
    ];
    if features.fx {
        elements.push(ChannelElement::new(K::FxFader, P::Fx));
    }
    for (panel, present) in [
        (Panel::Trim, features.trim),
        (Panel::Settings, true),
        (Panel::Eq, features.eq),
        (Panel::Dynamics, features.dynamics),
    ] {
        if present {
            elements.push(ChannelElement::new(K::Tab(panel), P::Stack(panel)));
        }
    }
    if kind != ChannelKind::Output {
        elements.push(ChannelElement::new(K::Submix, P::Submix));
    }
    elements
}

/// A template in the layout document: its identity and its elements.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutTemplate {
    pub id: TemplateId,
    pub kind: ChannelKind,
    pub stereo: bool,
    pub elements: Vec<ChannelElement>,
}

impl LayoutTemplate {
    /// The document form of a standard template.
    pub fn from_spec(spec: TemplateSpec) -> Self {
        Self {
            id: spec.id,
            kind: spec.kind,
            stereo: spec.stereo,
            elements: standard_elements(spec.kind, spec.features),
        }
    }

    /// The optional controls that the element list provides.
    pub fn features(&self) -> Features {
        let has = |kind: ElementKind| self.elements.iter().any(|element| element.kind == kind);
        Features {
            fx: has(ElementKind::FxFader),
            trim: has(ElementKind::Tab(Panel::Trim)),
            eq: has(ElementKind::Tab(Panel::Eq)),
            dynamics: has(ElementKind::Tab(Panel::Dynamics)),
        }
    }

    pub fn template(&self) -> ChannelTemplate {
        ChannelTemplate {
            spec: TemplateSpec {
                id: self.id,
                kind: self.kind,
                features: self.features(),
                stereo: self.stereo,
            },
            elements: self.elements.clone(),
        }
    }
}

/// The resolved rectangle of one channel element at one zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedElement {
    pub element: ChannelElement,
    /// Content or glyph rectangle of the element.
    pub rect: Rect,
    /// Face of an element that reaches the card's right edge.
    pub face: Option<Rect>,
}

/// Resolved geometry for one channel template at one zoom.
#[derive(Clone, Debug, PartialEq)]
pub struct StripLayout {
    pub scale: f32,
    pub card: Rect,
    pub content: Rect,
    /// Drawing surface of the channel fader, from the VU through the cap.
    pub fader_canvas: Rect,
    /// Resolved elements in template order.
    pub elements: Vec<ResolvedElement>,
    /// Glyph and face of `MR2` to `MR5`, drawn even when the template has no
    /// tab there so the reserved row stays visible.
    pub stack_tabs: [(Rect, Rect); 4],
}

impl StripLayout {
    /// Resolve every element of a template, or report the first element the
    /// channel kind cannot place.
    pub fn for_template(
        template: &ChannelTemplate,
        m: &LayoutMetrics,
        scale: f32,
    ) -> Result<Self, String> {
        Self::resolve(template.spec.kind, &template.elements, m, scale)
    }

    /// Resolve the standard composition of a channel kind.
    pub fn for_metrics(
        kind: ChannelKind,
        scale: f32,
        features: Features,
        m: &LayoutMetrics,
    ) -> Self {
        Self::resolve(kind, &standard_elements(kind, features), m, scale)
            .expect("a standard template always resolves")
    }

    pub fn for_kind(kind: ChannelKind, scale: f32, features: Features) -> Self {
        Self::for_metrics(kind, scale, features, &LayoutMetrics::default())
    }

    fn resolve(
        kind: ChannelKind,
        elements: &[ChannelElement],
        m: &LayoutMetrics,
        scale: f32,
    ) -> Result<Self, String> {
        let has_fx = elements
            .iter()
            .any(|element| element.kind == ElementKind::FxFader);
        let grid = Grid::resolve(kind, has_fx, m, scale);
        let mut resolved = Vec::with_capacity(elements.len());
        for element in elements {
            let (rect, face) = grid.placement(element.placement).ok_or_else(|| {
                format!(
                    "a {} has no {} element",
                    kind.label(),
                    element.placement.label()
                )
            })?;
            resolved.push(ResolvedElement {
                element: *element,
                rect,
                face,
            });
        }
        Ok(Self {
            scale,
            card: grid.card,
            content: grid.content,
            fader_canvas: grid.fader_canvas,
            elements: resolved,
            stack_tabs: grid
                .tab_glyphs
                .into_iter()
                .zip(grid.tab_faces)
                .collect::<Vec<_>>()
                .try_into()
                .expect("four tab rows"),
        })
    }

    /// Rect of an element by placement.
    pub fn rect(&self, placement: ElementPlacement) -> Option<Rect> {
        self.elements
            .iter()
            .find(|element| element.element.placement == placement)
            .map(|element| element.rect)
    }

    /// Face of an element by placement, for elements that reach the right edge.
    pub fn face(&self, placement: ElementPlacement) -> Option<Rect> {
        self.elements
            .iter()
            .find(|element| element.element.placement == placement)
            .and_then(|element| element.face)
    }

    /// The element of a given kind.
    pub fn element(&self, kind: ElementKind) -> Option<&ResolvedElement> {
        self.elements
            .iter()
            .find(|element| element.element.kind == kind)
    }

    /// Content centre line in this layout's coordinate space.
    pub fn centre_x(&self) -> f32 {
        self.content.centre_x()
    }
}

/// Candidate rectangles for every placement, at one zoom.
struct Grid {
    card: Rect,
    content: Rect,
    name: Rect,
    pan: Rect,
    buttons: [Rect; 2],
    readouts: [Rect; 2],
    vu: Rect,
    fader_canvas: Rect,
    fader: Rect,
    fx: Option<Rect>,
    tab_glyphs: [Rect; 4],
    tab_faces: [Rect; 4],
    db_readout: Rect,
    collapse_glyph: Rect,
    collapse_face: Rect,
    submix: Option<Rect>,
}

impl Grid {
    fn resolve(kind: ChannelKind, has_fx: bool, m: &LayoutMetrics, scale: f32) -> Self {
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
        let fx = has_fx.then(|| {
            r(
                m.content_x + m.tab_x + (m.tab_w - m.fx_w) / 2.0,
                middle_y,
                m.fx_w,
                m.fx_h,
            )
        });
        let tab_y = middle_y + if has_fx { m.tab_top_with_fx } else { 0.0 };
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

    fn placement(&self, placement: ElementPlacement) -> Option<(Rect, Option<Rect>)> {
        match placement {
            ElementPlacement::Name => Some((self.name, None)),
            ElementPlacement::Pan => Some((self.pan, None)),
            ElementPlacement::ButtonLeft => Some((self.buttons[0], None)),
            ElementPlacement::ButtonRight => Some((self.buttons[1], None)),
            ElementPlacement::ReadoutLeft => Some((self.readouts[0], None)),
            ElementPlacement::ReadoutRight => Some((self.readouts[1], None)),
            ElementPlacement::Meter => Some((self.vu, None)),
            ElementPlacement::Fader => Some((self.fader, None)),
            ElementPlacement::Fx => self.fx.map(|rect| (rect, None)),
            ElementPlacement::Stack(panel) => {
                let index = panel.stack_index();
                Some((self.tab_glyphs[index], Some(self.tab_faces[index])))
            }
            ElementPlacement::Level => Some((self.db_readout, None)),
            ElementPlacement::Minimize => Some((self.collapse_glyph, Some(self.collapse_face))),
            ElementPlacement::Submix => self.submix.map(|rect| (rect, None)),
        }
    }
}

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

    fn rect(layout: &StripLayout, placement: ElementPlacement) -> Rect {
        layout
            .rect(placement)
            .unwrap_or_else(|| panic!("{} is missing", placement.label()))
    }

    fn tab(layout: &StripLayout, panel: Panel) -> Rect {
        rect(layout, ElementPlacement::Stack(panel))
    }

    fn standard(id: TemplateId) -> StripLayout {
        let spec = TEMPLATES
            .iter()
            .copied()
            .find(|spec| spec.id == id)
            .expect("a standard template");
        StripLayout::for_template(&ChannelTemplate::of(spec), &LayoutMetrics::default(), 1.0)
            .expect("a standard template resolves")
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 0.01, "expected {b}, got {a}");
    }

    fn every_rect(layout: &StripLayout) -> Vec<(&'static str, Rect)> {
        let mut rects: Vec<(&'static str, Rect)> = layout
            .elements
            .iter()
            .map(|element| (element.element.label(), element.rect))
            .collect();
        rects.extend(
            layout
                .elements
                .iter()
                .filter_map(|element| element.face.map(|face| (element.element.label(), face))),
        );
        rects.push(("fader canvas", layout.fader_canvas));
        rects
    }

    #[test]
    fn rows_sum_to_the_standard_height() {
        let l = layout(ChannelKind::Input);
        approx(l.card.height, STRIP_H);
        approx(rect(&l, ElementPlacement::Name).y, CONTENT_Y);
        approx(rect(&l, ElementPlacement::Pan).y, 18.0);
        approx(rect(&l, ElementPlacement::ButtonLeft).y, 73.0);
        approx(rect(&l, ElementPlacement::ReadoutLeft).y, 92.0);
        approx(rect(&l, ElementPlacement::Meter).y, 111.0);
        approx(rect(&l, ElementPlacement::Fader).y, 111.0);
        approx(rect(&l, ElementPlacement::Level).y, 317.0);
        let submix = rect(&l, ElementPlacement::Submix);
        approx(submix.y, 332.0);
        approx(submix.bottom(), STRIP_H);
    }

    #[test]
    fn vu_and_fader_share_top_bottom_height_and_centre() {
        let l = layout(ChannelKind::Input);
        let vu = rect(&l, ElementPlacement::Meter);
        let fader = rect(&l, ElementPlacement::Fader);
        approx(vu.y, fader.y);
        approx(vu.bottom(), fader.bottom());
        approx(vu.width, VU_W);
        approx(fader.width, TRACK_W);
        approx(l.fader_canvas.x, CONTENT_X);
        approx(l.fader_canvas.width, FADER_CANVAS_W);
        assert!(l.fader_canvas.contains(vu));
        assert!(l.fader_canvas.contains(fader));
        approx(fader.x, CONTENT_X + TRACK_X);
        assert!(vu.right() < fader.x);
        approx(fader.centre_x(), l.centre_x());
    }

    #[test]
    fn top_rows_and_readout_share_the_centre_line() {
        let l = layout(ChannelKind::Input);
        approx(rect(&l, ElementPlacement::Name).centre_x(), l.centre_x());
        approx(rect(&l, ElementPlacement::Pan).centre_x(), l.centre_x());
        let left = rect(&l, ElementPlacement::ButtonLeft);
        let right = rect(&l, ElementPlacement::ButtonRight);
        approx((left.right() + right.x) / 2.0, l.centre_x());
        let readout_left = rect(&l, ElementPlacement::ReadoutLeft);
        let readout_right = rect(&l, ElementPlacement::ReadoutRight);
        approx((readout_left.right() + readout_right.x) / 2.0, l.centre_x());
        approx(rect(&l, ElementPlacement::Level).centre_x(), l.centre_x());
    }

    #[test]
    fn tab_faces_reach_the_right_edge_and_keep_the_glyph_centred() {
        let l = layout(ChannelKind::Input);
        for panel in Panel::ALL {
            let glyph = tab(&l, panel);
            let face = l
                .face(ElementPlacement::Stack(panel))
                .unwrap_or_else(|| panic!("{} has no face", panel.label()));
            approx(face.right(), STRIP_W);
            approx(glyph.centre_x(), face.x + TAB_W / 2.0);
            assert!(face.contains(glyph));
        }
        let minimize = rect(&l, ElementPlacement::Minimize);
        let minimize_face = l.face(ElementPlacement::Minimize).expect("minimize face");
        approx(minimize_face.right(), STRIP_W);
        approx(minimize.centre_x(), minimize_face.x + COLLAPSE_W / 2.0);
    }

    #[test]
    fn every_tab_row_reaches_the_right_edge_without_an_element() {
        // A playback has no Trim tab; the reserved row still has geometry.
        let l = standard(TemplateId::PlaybackStereo);
        assert!(
            l.rect(ElementPlacement::Stack(Panel::Trim)).is_none(),
            "the playback template has no Trim tab"
        );
        for (glyph, face) in l.stack_tabs {
            approx(face.right(), STRIP_W);
            assert!(face.contains(glyph));
        }
    }

    #[test]
    fn the_submix_bar_is_full_width_and_flush_with_the_bottom() {
        for kind in [ChannelKind::Input, ChannelKind::Playback] {
            let l = layout(kind);
            let s = rect(&l, ElementPlacement::Submix);
            approx(s.x, 0.0);
            approx(s.width, STRIP_W);
            approx(s.bottom(), STRIP_H);
        }
        assert!(layout(ChannelKind::Output)
            .rect(ElementPlacement::Submix)
            .is_none());
    }

    #[test]
    fn visible_elements_do_not_overlap() {
        let l = layout(ChannelKind::Input);
        let rects: Vec<Rect> = l.elements.iter().map(|element| element.rect).collect();
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                assert!(
                    !rects[i].overlaps(rects[j]),
                    "{} and {} overlap: {:?} / {:?}",
                    l.elements[i].element.label(),
                    l.elements[j].element.label(),
                    rects[i],
                    rects[j]
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
            for (label, rect) in every_rect(&l) {
                assert!(l.card.contains(rect), "{label} leaves the card: {rect:?}");
            }
        }
    }

    #[test]
    fn zoom_scales_every_rectangle() {
        let l = StripLayout::for_kind(ChannelKind::Input, 1.5, Features::default());
        approx(l.card.width, STRIP_W * 1.5);
        approx(l.card.height, STRIP_H * 1.5);
        approx(rect(&l, ElementPlacement::Meter).height, MIDDLE_H * 1.5);
        approx(rect(&l, ElementPlacement::Fader).centre_x(), l.centre_x());
        for panel in Panel::ALL {
            let glyph = tab(&l, panel);
            let face = l.face(ElementPlacement::Stack(panel)).expect("tab face");
            approx(face.right(), STRIP_W * 1.5);
            approx(glyph.centre_x(), face.x + TAB_W * 1.5 / 2.0);
        }
    }

    #[test]
    fn template_tokens_round_trip() {
        for id in TemplateId::ALL {
            assert_eq!(TemplateId::parse(id.token()), Some(id));
            assert_eq!(TemplateId::parse(id.label()), Some(id));
        }
        assert_eq!(TemplateId::parse("Nope"), None);
    }

    #[test]
    fn an_output_has_no_submix_and_no_trim_tab() {
        let l = standard(TemplateId::ControlRoom);
        assert!(l.rect(ElementPlacement::Fx).is_some());
        assert!(l.rect(ElementPlacement::Submix).is_none());
        assert!(l.rect(ElementPlacement::Stack(Panel::Trim)).is_none());
        approx(
            l.stack_tabs[Panel::Trim.stack_index()].0.y,
            rect(&l, ElementPlacement::Meter).y + FX_H + 8.0,
        );
    }

    #[test]
    fn a_template_declares_each_element_and_placement_once() {
        for spec in TEMPLATES {
            let template = ChannelTemplate::of(*spec);
            let mut placements: Vec<ElementPlacement> = template
                .elements
                .iter()
                .map(|element| element.placement)
                .collect();
            let before = placements.len();
            placements.sort_by_key(|placement| placement.label());
            placements.dedup();
            assert_eq!(
                before,
                placements.len(),
                "{:?} repeats a placement",
                spec.id
            );

            let mut kinds: Vec<ElementKind> = template
                .elements
                .iter()
                .map(|element| element.kind)
                .collect();
            let before = kinds.len();
            kinds.sort_by_key(|kind| kind.label());
            kinds.dedup();
            assert_eq!(before, kinds.len(), "{:?} repeats an element", spec.id);
        }
    }

    #[test]
    fn every_standard_template_resolves_to_the_same_grid() {
        for spec in TEMPLATES {
            let template = ChannelTemplate::of(*spec);
            let layouts = StripLayout::for_template(&template, &LayoutMetrics::default(), 1.0)
                .expect("a standard template resolves");
            let direct = StripLayout::for_kind(spec.kind, 1.0, spec.features);
            assert_eq!(layouts.elements, direct.elements, "{:?}", spec.id);
        }
    }

    #[test]
    fn an_element_outside_the_channel_kind_is_reported() {
        let spec = TEMPLATES
            .iter()
            .copied()
            .find(|spec| spec.kind == ChannelKind::Output)
            .expect("an output template");
        let mut template = ChannelTemplate::of(spec);
        template.elements.push(ChannelElement::new(
            ElementKind::Submix,
            ElementPlacement::Submix,
        ));
        let error = StripLayout::for_template(&template, &LayoutMetrics::default(), 1.0)
            .expect_err("an output has no submix");
        assert!(error.contains("no B2"), "{error}");
    }

    #[test]
    fn the_standard_elements_bind_to_the_expected_state() {
        let template = ChannelTemplate::of(TEMPLATES[0]);
        let bindings: Vec<Binding> = template
            .elements
            .iter()
            .map(|element| element.binding())
            .collect();
        assert!(bindings.contains(&Binding::Name));
        assert!(bindings.contains(&Binding::Meter));
        assert!(bindings.contains(&Binding::Level));
        assert!(bindings.contains(&Binding::Toggle(Action::Mute)));
        assert!(bindings.contains(&Binding::Toggle(Action::Solo)));
        assert!(bindings.contains(&Binding::Panel(Panel::Eq)));
    }
}

impl Default for VisualTokens {
    fn default() -> Self {
        VISUAL
    }
}

/// Schema version of the shared GUI layout configuration.
pub const CONFIG_SCHEMA: u32 = 2;

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

fn default_templates() -> Vec<LayoutTemplate> {
    TEMPLATES
        .iter()
        .map(|spec| LayoutTemplate::from_spec(*spec))
        .collect()
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
    pub templates: Vec<LayoutTemplate>,
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

    /// The user's layout document:
    /// `$XDG_CONFIG_HOME/totalcontrol/gui-layout.json`, normally
    /// `~/.config/totalcontrol/gui-layout.json`.
    pub fn default_path() -> Option<std::path::PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
            })?;
        Some(base.join("totalcontrol").join("gui-layout.json"))
    }

    /// Load a document, or fall back to the built-in layout.
    ///
    /// A missing file is not an error.  An unreadable or invalid document is
    /// returned as the message so the caller can report it.
    pub fn load_or_default(path: &std::path::Path) -> (Self, Option<String>) {
        if !path.exists() {
            return (Self::default(), None);
        }
        match Self::load(path) {
            Ok(config) => (config, None),
            Err(error) => (Self::default(), Some(error)),
        }
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
            if template.elements.is_empty() {
                return Err(format!(
                    "template {:?} must contain at least one element",
                    template.id
                ));
            }
            let mut placements = std::collections::HashSet::new();
            for element in &template.elements {
                if !element.is_coherent() {
                    return Err(format!(
                        "template {:?}: {} is not a {} element",
                        template.id,
                        element.placement.label(),
                        element.kind.label()
                    ));
                }
                if !placements.insert(element.placement) {
                    return Err(format!(
                        "template {:?}: {} is placed twice",
                        template.id,
                        element.placement.label()
                    ));
                }
            }
        }
        for template in &self.templates {
            let layout = self
                .layout(template, 1.0)
                .map_err(|error| format!("template {:?}: {error}", template.id))?;
            for (label, rect) in layout
                .elements
                .iter()
                .flat_map(|element| {
                    std::iter::once((element.element.label(), element.rect))
                        .chain(element.face.map(|face| (element.element.label(), face)))
                })
                .chain(std::iter::once(("fader canvas", layout.fader_canvas)))
            {
                if !layout.card.contains(rect) {
                    return Err(format!(
                        "template {:?}: the {label} element leaves the {} x {} card",
                        template.id, m.strip_w, m.strip_h
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
            let rect_at = |placement: ElementPlacement| layout.rect(placement);
            for (name, placement, hint) in [
                (
                    "geometry.pan_x and pan_w",
                    ElementPlacement::Pan,
                    "pan_x = (content_w - pan_w) / 2",
                ),
                (
                    "geometry.db_x and db_w",
                    ElementPlacement::Level,
                    "db_x = (content_w - db_w) / 2",
                ),
                (
                    "geometry.track_x and track_w",
                    ElementPlacement::Fader,
                    "track_x = (content_w - track_w) / 2",
                ),
            ] {
                if let Some(rect) = rect_at(placement) {
                    centred(name, rect, hint)?;
                }
            }
            if let (Some(left), Some(right)) = (
                rect_at(ElementPlacement::ButtonLeft),
                rect_at(ElementPlacement::ButtonRight),
            ) {
                let pair = Rect::new(left.x, left.y, right.right() - left.x, left.height);
                centred(
                    "geometry.button_w and button_gap",
                    pair,
                    "content_w = 2 * button_w + button_gap",
                )?;
            }
        }
        Ok(())
    }

    pub fn layout(&self, template: &LayoutTemplate, scale: f32) -> Result<StripLayout, String> {
        StripLayout::for_template(&template.template(), &self.geometry, scale)
    }

    pub fn template(&self, id: TemplateId) -> Option<&LayoutTemplate> {
        self.templates.iter().find(|template| template.id == id)
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
        assert_eq!(config.templates, default_templates());
        let ids: Vec<TemplateId> = config
            .templates
            .iter()
            .map(|template| template.id)
            .collect();
        assert_eq!(
            ids,
            TEMPLATES.iter().map(|spec| spec.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_geometry_override_reaches_the_resolved_layout() {
        let mut config = GuiLayoutConfig::default();
        config.geometry.strip_w = 100.0;
        config.geometry.strip_h = 360.0;
        let layout = config
            .layout(&config.templates[0], 1.0)
            .expect("standard template resolves");
        assert_eq!(layout.card.width, 100.0);
        assert_eq!(layout.card.height, 360.0);
    }

    #[test]
    fn an_element_list_replaces_the_compiled_composition() {
        let mut config = GuiLayoutConfig::default();
        config.templates[0]
            .elements
            .retain(|element| element.kind != ElementKind::FxFader);
        assert!(!config.templates[0].features().fx);
        config.validate().expect("a template without FX is valid");
        let layout = config.layout(&config.templates[0], 1.0).expect("resolves");
        assert!(layout.rect(ElementPlacement::Fx).is_none());
        assert!(layout.rect(ElementPlacement::Meter).is_some());
    }

    #[test]
    fn an_element_in_the_wrong_place_is_rejected() {
        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "stereo": false,
            "elements": [{"kind": "name", "at": "TL3"}]}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("mismatched pair");
        assert!(error.contains("TL3 is not a name element"), "{error}");

        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "stereo": false,
            "elements": [{"kind": "solo", "at": "TR3"}, {"kind": "cue", "at": "TR3"}]}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("duplicate placement");
        assert!(error.contains("TR3 is placed twice"), "{error}");
    }

    #[test]
    fn unknown_element_tokens_are_rejected() {
        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "stereo": false,
            "elements": [{"kind": "vu", "at": "ML"}]}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("unknown kind");
        assert!(error.contains("unknown element kind"), "{error}");

        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "stereo": false,
            "elements": [{"kind": "meter", "at": "M9"}]}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("unknown placement");
        assert!(error.contains("unknown placement"), "{error}");
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
    fn a_missing_document_falls_back_without_a_message() {
        let path = std::env::temp_dir().join(format!(
            "totalcontrol-layout-{}-missing.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let (config, error) = GuiLayoutConfig::load_or_default(&path);
        assert!(error.is_none());
        assert_eq!(config, GuiLayoutConfig::default());
    }

    #[test]
    fn an_invalid_document_reports_and_falls_back() {
        let path = std::env::temp_dir().join(format!(
            "totalcontrol-layout-{}-invalid.json",
            std::process::id()
        ));
        std::fs::write(&path, r#"{"schema": 1}"#).expect("the fixture writes");
        let (config, error) = GuiLayoutConfig::load_or_default(&path);
        let message = error.expect("an invalid document reports");
        assert!(message.contains("unsupported layout schema"), "{message}");
        assert_eq!(config, GuiLayoutConfig::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_empty_document_uses_the_built_in_layout() {
        let config = GuiLayoutConfig::from_json("{}").expect("empty document loads");
        assert_eq!(config.geometry, LayoutMetrics::default());
        assert_eq!(config.visual, VISUAL);
        assert_eq!(config.colors, ColorScheme::default());
        assert_eq!(config.templates, default_templates());
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

        let text = r#"{"templates": [{"id": "InputMono", "kind": "Input", "stereo": false,
            "elements": [{"kind": "meter", "at": "ML", "extra": 1}]}]}"#;
        let error = GuiLayoutConfig::from_json(text).expect_err("unknown element field");
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
        let elements = property(&template_properties, "elements");
        let element_item = value_property(elements, "items");
        let element_properties = object_property(element_item, "properties");
        let mut documented: Vec<&str> = element_properties.keys().map(String::as_str).collect();
        documented.sort_unstable();
        let mut required = string_list(value_property(element_item, "required"));
        required.sort_unstable();
        assert_eq!(
            documented, required,
            "element fields and required list differ"
        );

        for (property_name, compiled) in [
            (
                "kind",
                ElementKind::ALL
                    .iter()
                    .map(|kind| kind.label().to_string())
                    .collect::<Vec<_>>(),
            ),
            (
                "at",
                ElementPlacement::ALL
                    .iter()
                    .map(|placement| placement.label().to_string())
                    .collect::<Vec<_>>(),
            ),
        ] {
            let mut documented = string_list(property(&element_properties, property_name));
            documented.sort_unstable();
            let mut compiled = compiled;
            compiled.sort_unstable();
            assert_eq!(documented, compiled, "{property_name} tokens differ");
        }

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
