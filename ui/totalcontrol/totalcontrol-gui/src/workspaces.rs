//! Named TotalControl workspaces.
//!
//! A workspace is the portable host-side configuration: the current mixer
//! scene, the eight snapshots, channel labels, groups, layouts, view state
//! and Control Room preferences.  It is deliberately separate from the
//! mixer's auto-saved scene and from the unit's six device-side setups.  A
//! workspace checks the device model but does not store the serial number.
//!
//! The file is JSON with the `.tcws` extension.  Writes are atomic so a
//! failed save cannot destroy the previous workspace.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use totalcontrol_core::{ChannelId, Scene};

use crate::app::{RowMode, View, OUTPUT_DESTS};
use crate::prefs::ControlRoomSettings;
use crate::sidebar::Group;

pub const EXTENSION: &str = "tcws";
pub const SCHEMA: u32 = 1;

fn default_schema() -> u32 {
    SCHEMA
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceView {
    pub view: View,
    pub row_mode: RowMode,
    pub sidebar_open: bool,
    pub sidebar_width: f32,
    pub panels_open: [bool; 4],
    pub fx_panel: bool,
    pub ui_scale: f32,
    pub sel_out: usize,
    pub sel_out_side: usize,
    pub quick_channel: ChannelId,
    pub show_original_names: bool,
    pub matrix_mono: bool,
    pub active_snapshot: Option<usize>,
    pub active_layout: Option<usize>,
    pub collapsed: HashSet<ChannelId>,
    /// Legacy field kept for schema compatibility. Active Talkback is a
    /// momentary performance state and is never restored from a workspace.
    pub talkback_on: bool,
    /// External Input is latched on in this workspace.
    pub external_input_on: bool,
    /// Input channels listening through PFL when the workspace was saved.
    pub pfl_inputs: Vec<ChannelId>,
}

impl Default for WorkspaceView {
    fn default() -> Self {
        Self {
            view: View::Quick,
            row_mode: RowMode::Three,
            sidebar_open: true,
            sidebar_width: crate::sidebar::DEFAULT_WIDTH,
            panels_open: [true; 4],
            fx_panel: true,
            ui_scale: default_scale(),
            sel_out: 0,
            sel_out_side: 0,
            quick_channel: ChannelId::Input(0),
            show_original_names: false,
            matrix_mono: false,
            active_snapshot: None,
            active_layout: None,
            collapsed: HashSet::new(),
            talkback_on: false,
            external_input_on: false,
            pfl_inputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceSnapshot {
    /// User-visible snapshot name.  Defaults to `Mix n` when empty.
    pub name: String,
    pub scene: Option<Scene>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    #[serde(default = "default_schema")]
    pub schema: u32,
    pub name: String,
    /// The current mixer scene.
    pub scene: Scene,
    /// The eight Mix slots.
    pub snapshots: [WorkspaceSnapshot; 8],
    pub groups: [Group; 4],
    /// The global fader group (the Shift+clicked strips), stored so a
    /// workspace restores the whole working set, not only the four
    /// permanent groups.
    pub temporary_group: Vec<ChannelId>,
    /// Whether that group was ganged (`F` lit) when saved. Missing in old
    /// workspaces, which always stored a ganged group.
    #[serde(default = "fader_ganged_default")]
    pub fader_ganged: bool,
    /// The six Mixer Layout slots.
    pub layouts: [Option<HashSet<ChannelId>>; 6],
    pub labels: Vec<(ChannelId, String)>,
    pub view: WorkspaceView,
    pub control_room: ControlRoomSettings,
    pub assign: [Option<usize>; 7],
    pub surround_51: bool,
    pub surround_71: bool,
    pub window_size: Option<(f32, f32)>,
}

fn fader_ganged_default() -> bool {
    true
}

impl Default for Workspace {
    fn default() -> Self {
        let mut scene = Scene::new("Untitled");
        scene.model = String::new();
        Self {
            schema: SCHEMA,
            name: "Untitled".into(),
            scene,
            snapshots: Default::default(),
            groups: Default::default(),
            temporary_group: Vec::new(),
            fader_ganged: false,
            layouts: Default::default(),
            labels: Vec::new(),
            view: WorkspaceView::default(),
            control_room: ControlRoomSettings::default(),
            assign: [None; 7],
            surround_51: false,
            surround_71: false,
            window_size: None,
        }
    }
}

/// A predefined surround workspace.  The direct layouts map the virtual
/// sink channels one-to-one to hardware outputs; the downmix layouts fold
/// 5.1 or 7.1 into four full-range channels (front and rear pairs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurroundLayout {
    Direct51,
    Direct71,
    Downmix51To40,
    Downmix71To40,
}

impl SurroundLayout {
    pub const ALL: [Self; 4] = [
        Self::Direct51,
        Self::Direct71,
        Self::Downmix51To40,
        Self::Downmix71To40,
    ];

    pub fn menu_label(self) -> &'static str {
        match self {
            Self::Direct51 => "5.1 Direct",
            Self::Direct71 => "7.1 Direct",
            Self::Downmix51To40 => "5.1 → 4.0",
            Self::Downmix71To40 => "7.1 → 4.0",
        }
    }

    pub fn name(self) -> &'static str {
        self.menu_label()
    }

    fn sources(self) -> &'static [SurroundSourceSpec] {
        match self {
            Self::Direct51 | Self::Downmix51To40 => SOURCES_51,
            Self::Direct71 | Self::Downmix71To40 => SOURCES_71,
        }
    }

    fn outputs(self) -> &'static [SurroundOutputSpec] {
        match self {
            Self::Direct51 => OUTPUTS_51,
            Self::Direct71 => OUTPUTS_71,
            Self::Downmix51To40 | Self::Downmix71To40 => OUTPUTS_40,
        }
    }

    fn routes(self) -> Vec<SurroundRoute> {
        let mut routes = vec![
            route(0, 0, 0, 1.0),
            route(1, 0, 1, 1.0),
        ];
        match self {
            Self::Direct51 => {
                routes.extend([
                    route(2, 1, 0, 1.0),
                    route(3, 1, 1, 1.0),
                    route(4, 2, 0, 1.0),
                    route(5, 2, 1, 1.0),
                ]);
            }
            Self::Direct71 => {
                routes.extend([
                    route(2, 1, 0, 1.0),
                    route(3, 1, 1, 1.0),
                    route(4, 2, 0, 1.0),
                    route(5, 2, 1, 1.0),
                    route(6, 3, 0, 1.0),
                    route(7, 3, 1, 1.0),
                ]);
            }
            Self::Downmix51To40 => {
                routes.extend([
                    route(2, 0, 0, 0.707),
                    route(2, 0, 1, 0.707),
                    route(3, 0, 0, 0.707),
                    route(3, 0, 1, 0.707),
                    route(4, 1, 0, 1.0),
                    route(5, 1, 1, 1.0),
                ]);
            }
            Self::Downmix71To40 => {
                routes.extend([
                    route(2, 0, 0, 0.707),
                    route(2, 0, 1, 0.707),
                    route(3, 0, 0, 0.707),
                    route(3, 0, 1, 0.707),
                    route(4, 1, 0, 1.0),
                    route(5, 1, 1, 1.0),
                    route(6, 1, 0, 1.0),
                    route(7, 1, 1, 1.0),
                ]);
            }
        }
        routes
    }

    /// Software-playback pairs that are split into two mono strips.
    fn split_pairs(self) -> &'static [usize] {
        match self {
            Self::Direct51 => &[1],
            Self::Direct71 => &[1],
            Self::Downmix51To40 | Self::Downmix71To40 => &[1],
        }
    }

    fn seven_one(self) -> bool {
        matches!(self, Self::Direct71 | Self::Downmix71To40)
    }
}

#[derive(Debug, Clone, Copy)]
struct SurroundSourceSpec {
    channel: usize,
    label: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct SurroundOutputSpec {
    /// Display position of the hardware-output pair.
    pair: usize,
    linked: bool,
    left: &'static str,
    right: &'static str,
    left_member: bool,
    right_member: bool,
}

#[derive(Debug, Clone, Copy)]
struct SurroundRoute {
    source: usize,
    pair: usize,
    side: usize,
    gain: f32,
}

fn route(source: usize, pair: usize, side: usize, gain: f32) -> SurroundRoute {
    SurroundRoute {
        source,
        pair,
        side,
        gain,
    }
}

const SOURCES_51: &[SurroundSourceSpec] = &[
    SurroundSourceSpec { channel: 0, label: "Front L" },
    SurroundSourceSpec { channel: 1, label: "Front R" },
    SurroundSourceSpec { channel: 2, label: "Center" },
    SurroundSourceSpec { channel: 3, label: "LFE" },
    SurroundSourceSpec { channel: 4, label: "Side L" },
    SurroundSourceSpec { channel: 5, label: "Side R" },
];

const SOURCES_71: &[SurroundSourceSpec] = &[
    SurroundSourceSpec { channel: 0, label: "Front L" },
    SurroundSourceSpec { channel: 1, label: "Front R" },
    SurroundSourceSpec { channel: 2, label: "Center" },
    SurroundSourceSpec { channel: 3, label: "LFE" },
    SurroundSourceSpec { channel: 4, label: "Rear L" },
    SurroundSourceSpec { channel: 5, label: "Rear R" },
    SurroundSourceSpec { channel: 6, label: "Side L" },
    SurroundSourceSpec { channel: 7, label: "Side R" },
];

const OUTPUTS_51: &[SurroundOutputSpec] = &[
    SurroundOutputSpec {
        pair: 0,
        linked: true,
        left: "Front L",
        right: "Front R",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 1,
        linked: false,
        left: "Center",
        right: "LFE",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 2,
        linked: true,
        left: "Side L",
        right: "Side R",
        left_member: true,
        right_member: true,
    },
];

const OUTPUTS_71: &[SurroundOutputSpec] = &[
    SurroundOutputSpec {
        pair: 0,
        linked: true,
        left: "Front L",
        right: "Front R",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 1,
        linked: false,
        left: "Center",
        right: "LFE",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 2,
        linked: true,
        left: "Rear L",
        right: "Rear R",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 3,
        linked: true,
        left: "Side L",
        right: "Side R",
        left_member: true,
        right_member: true,
    },
];

const OUTPUTS_40: &[SurroundOutputSpec] = &[
    SurroundOutputSpec {
        pair: 0,
        linked: true,
        left: "Front L",
        right: "Front R",
        left_member: true,
        right_member: true,
    },
    SurroundOutputSpec {
        pair: 1,
        linked: true,
        left: "Rear L",
        right: "Rear R",
        left_member: true,
        right_member: true,
    },
];

/// How the current device numbers output pairs and their per-pair send
/// arrays.  `output_order` maps a display position to the internal pair
/// index; `matrix_uses_display_order` is true for the UC, whose channel
/// arrays are already in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurroundTopology {
    pub output_order: Vec<usize>,
    pub matrix_uses_display_order: bool,
}

impl SurroundTopology {
    fn internal_pair(&self, display_pair: usize) -> Option<usize> {
        self.output_order.get(display_pair).copied()
    }

    fn volume_pair(&self, display_pair: usize) -> Option<usize> {
        if self.matrix_uses_display_order {
            Some(display_pair)
        } else {
            self.internal_pair(display_pair)
        }
    }
}

fn set_label(workspace: &mut Workspace, channel: ChannelId, label: &str) {
    workspace.labels.retain(|(id, _)| *id != channel);
    workspace.labels.push((channel, label.to_string()));
}

/// Reset the mixer and monitoring state to factory defaults while keeping
/// the device topology and device-owned integration settings (clock, rate,
/// references, preamps).
fn factory_scene(base: &Scene) -> Scene {
    let mut scene = base.clone();
    for input in &mut scene.inputs {
        input.volumes.fill(0.0);
        input.pans.fill(0);
        input.fx_send_db = -65.0;
        input.eq = input.eq.map(|_| Default::default());
        input.dynamics = input.dynamics.map(|_| Default::default());
        input.mute = false;
        input.solo = false;
        input.trim = 0.0;
        input.width = 1.0;
        input.ms_proc = false;
        input.phase = false;
        input.phase_r = false;
    }
    for playback in &mut scene.playbacks {
        playback.volumes.fill(0.0);
        playback.pans.fill(0);
        playback.fx_send_db = -65.0;
        playback.split = false;
        playback.trim = 0.0;
        playback.width = 1.0;
        playback.ms_proc = false;
        playback.mute = false;
        playback.solo = false;
    }
    for output in &mut scene.outputs {
        output.volume = 1.0;
        output.eq = output.eq.map(|_| Default::default());
        output.dynamics = output.dynamics.map(|_| Default::default());
        output.loopback = false;
        output.balance = 0;
        output.fx_return_db = -65.0;
        output.phase_l = false;
        output.phase_r = false;
        output.no_trim = false;
        output.talkback = false;
        output.mute = false;
        output.solo = false;
        output.cue = false;
    }
    scene.matrix_overrides.clear();
    let settings = &mut scene.settings;
    settings.pitch_percent = 0.0;
    settings.ms_proc = false;
    settings.an12 = false;
    settings.eq_for_record = false;
    settings.dim = false;
    settings.dim_amount_db = -20.0;
    settings.dim_saved.clear();
    settings.speaker_b = false;
    settings.main_out = None;
    settings.main_b_out = None;
    settings.mono = false;
    settings.mute_fx = false;
    settings.external_input = false;
    settings.fx_send_db = None;
    settings.reverb = settings.reverb.map(|_| Default::default());
    settings.echo = settings.echo.map(|_| Default::default());
    settings.width = 0.0;
    settings.input_link = false;
    settings.input_pair_link = vec![false; scene.inputs.len() / 2];
    settings.output_link = vec![true; scene.outputs.len() / 2];
    scene
}

/// Build a completely initialized factory surround workspace.  Nothing
/// from the current mixer, routing, FX, labels, groups, layouts or
/// snapshots is carried over.
pub fn factory_workspace(
    base: &Workspace,
    layout: SurroundLayout,
    topology: &SurroundTopology,
) -> Result<Workspace, String> {
    let mut workspace = Workspace::default();
    workspace.scene = factory_scene(&base.scene);
    workspace.view.view = View::Mixer;
    workspace.view.show_original_names = false;
    workspace.view.ui_scale = base.view.ui_scale;
    workspace.view.matrix_mono = true;
    workspace.view.fx_panel = false;
    workspace.window_size = None;
    configure_layout(&mut workspace, layout, topology)?;
    Ok(workspace)
}

fn configure_layout(
    workspace: &mut Workspace,
    layout: SurroundLayout,
    topology: &SurroundTopology,
) -> Result<(), String> {
    let sources = layout.sources();
    let outputs = layout.outputs();
    let routes = layout.routes();

    for source in sources {
        if workspace.scene.playbacks.get(source.channel).is_none() {
            return Err(format!(
                "This device has no software playback channel {}",
                source.channel + 1
            ));
        }
    }

    let mut internal_pairs = Vec::with_capacity(outputs.len());
    let mut volume_pairs = Vec::with_capacity(outputs.len());
    let mut chosen_blocks = std::collections::HashSet::new();
    for output in outputs {
        let internal = topology.internal_pair(output.pair).ok_or_else(|| {
            format!(
                "This device has no hardware output pair {}",
                output.pair + 1
            )
        })?;
        let volume = topology.volume_pair(output.pair).ok_or_else(|| {
            format!(
                "This device has no hardware output pair {}",
                output.pair + 1
            )
        })?;
        if workspace.scene.outputs.len() < internal * 2 + 2 {
            return Err(format!(
                "This device has no hardware output pair {}",
                output.pair + 1
            ));
        }
        internal_pairs.push(internal);
        volume_pairs.push(volume);
        chosen_blocks.insert(output.pair * 2);
        chosen_blocks.insert(output.pair * 2 + 1);
    }

    // Clear the playback row so no old send leaks into the surround
    // outputs, then clear input sends and mono-matrix cells that address
    // the output pairs chosen above.  Input routing to other outputs is
    // left alone.
    for playback in &mut workspace.scene.playbacks {
        playback.volumes.fill(0.0);
        playback.pans.fill(0);
        playback.mute = false;
        playback.solo = false;
        playback.trim = 0.0;
        playback.width = 1.0;
        playback.ms_proc = false;
        playback.split = false;
    }
    for input in &mut workspace.scene.inputs {
        for &pair in &volume_pairs {
            if let Some(volume) = input.volumes.get_mut(pair) {
                *volume = 0.0;
            }
        }
    }
    workspace
        .scene
        .matrix_overrides
        .retain(|cell| cell.source < 18 && !chosen_blocks.contains(&cell.block));

    for output in outputs {
        let internal = topology.internal_pair(output.pair).ok_or_else(|| {
            format!(
                "This device has no hardware output pair {}",
                output.pair + 1
            )
        })?;
        let link = &mut workspace.scene.settings.output_link;
        if link.len() <= internal {
            link.resize(internal + 1, true);
        }
        link[internal] = output.linked;
        for (side, (label, member)) in [
            (output.left, output.left_member),
            (output.right, output.right_member),
        ]
        .into_iter()
        .enumerate()
        {
            let channel = internal * 2 + side;
            let out = &mut workspace.scene.outputs[channel];
            out.volume = if member { 1.0 } else { 0.0 };
            out.mute = false;
            out.solo = false;
            out.balance = 0;
            out.phase_l = false;
            out.phase_r = false;
            out.no_trim = false;
            out.loopback = false;
            out.cue = false;
            out.talkback = false;
            set_label(workspace, ChannelId::Output(channel), label);
        }
    }

    for &pair in layout.split_pairs() {
        for side in 0..2 {
            let channel = pair * 2 + side;
            let Some(playback) = workspace.scene.playbacks.get_mut(channel) else {
                return Err(format!(
                    "This device has no software playback channel {}",
                    channel + 1
                ));
            };
            playback.split = true;
        }
    }

    // Route by source/pair, putting the natural side in the pair model and
    // any cross-side gain in a mono matrix override.
    let mut pair_gains: std::collections::BTreeMap<(usize, usize), [Option<f32>; 2]> =
        std::collections::BTreeMap::new();
    for route in routes {
        if route.side > 1 {
            return Err("Surround route has an invalid side".into());
        }
        pair_gains
            .entry((route.source, route.pair))
            .or_insert([None, None])[route.side] = Some(route.gain);
    }
    for ((source, pair), gains) in pair_gains {
        let volume_pair = topology.volume_pair(pair).ok_or_else(|| {
            format!("This device has no hardware output pair {}", pair + 1)
        })?;
        let Some(playback) = workspace.scene.playbacks.get_mut(source) else {
            return Err(format!(
                "This device has no software playback channel {}",
                source + 1
            ));
        };
        let natural = source % 2;
        let natural_gain = gains[natural].unwrap_or(0.0);
        if let Some(volume) = playback.volumes.get_mut(volume_pair) {
            *volume = natural_gain;
        }
        for side in 0..2 {
            if side == natural {
                continue;
            }
            let Some(gain) = gains[side] else {
                continue;
            };
            if gain > 0.0 {
                workspace
                    .scene
                    .matrix_overrides
                    .push(totalcontrol_core::scene::MatrixOverride {
                        source: 18 + source,
                        block: pair * 2 + side,
                        gain,
                    });
            }
        }
    }

    for source in sources {
        set_label(workspace, ChannelId::Playback(source.channel), source.label);
    }

    let members: Vec<ChannelId> = outputs
        .iter()
        .flat_map(|output| {
            let internal = topology.internal_pair(output.pair).unwrap_or(0);
            [
                (internal * 2, output.left_member),
                (internal * 2 + 1, output.right_member),
            ]
            .into_iter()
            .filter_map(|(channel, member)| member.then_some(ChannelId::Output(channel)))
        })
        .collect();
    workspace.groups[0] = Group {
        members,
        mute_linked: true,
        solo_linked: true,
        fader_linked: true,
    };

    for slot in workspace.assign[..OUTPUT_DESTS].iter_mut() {
        if slot.is_some_and(|pair| internal_pairs.contains(&pair)) {
            *slot = None;
        }
    }
    workspace.scene.settings.main_out = None;
    workspace.scene.settings.main_b_out = None;
    workspace.scene.settings.dim = false;
    workspace.scene.settings.speaker_b = false;
    workspace.scene.settings.mono = false;
    workspace.scene.settings.mute_fx = false;
    workspace.scene.settings.external_input = false;
    workspace.view.talkback_on = false;
    workspace.view.external_input_on = false;
    workspace.view.pfl_inputs.clear();
    workspace.view.active_snapshot = None;
    workspace.view.active_layout = None;
    workspace.view.collapsed.clear();
    workspace.view.sel_out = internal_pairs[0];
    workspace.view.sel_out_side = 0;
    workspace.view.show_original_names = false;
    workspace.view.matrix_mono = true;

    workspace.surround_51 = !layout.seven_one();
    workspace.surround_71 = layout.seven_one();
    workspace.name = layout.name().to_string();
    workspace.scene.name = workspace.name.clone();
    Ok(())
}

/// Default folder for the file dialogs.  `Documents` when it exists,
/// otherwise the user's home directory.
pub fn default_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let documents = home.join("Documents");
    if documents.is_dir() {
        documents
    } else {
        home
    }
}

/// Read and validate a workspace file.
pub fn load(path: &Path) -> Result<Workspace, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    let workspace: Workspace = serde_json::from_str(&text)
        .map_err(|e| format!("Could not parse {}: {e}", path.display()))?;
    if workspace.schema > SCHEMA {
        return Err(format!(
            "{} was written by a newer TotalControl (workspace schema {}).",
            path.display(),
            workspace.schema
        ));
    }
    Ok(workspace)
}

/// Write a workspace atomically.
pub fn save(path: &Path, workspace: &Workspace) -> Result<(), String> {
    let json = serde_json::to_string_pretty(workspace)
        .map_err(|e| format!("Could not serialise workspace: {e}"))?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
    let tmp = path.with_extension(format!("{EXTENSION}.tmp"));
    std::fs::write(&tmp, json)
        .map_err(|e| format!("Could not write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Could not replace {}: {e}", path.display())
    })
}

/// Ensure a save dialog name ends in the workspace extension.
pub fn with_extension(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|s| s.to_str()) == Some(EXTENSION) {
        path
    } else {
        path.with_extension(EXTENSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use totalcontrol_core::scene::MatrixOverride;
    use totalcontrol_core::{
        ChannelType, InputChannel, OutputChannel, PlaybackChannel, Reverb,
    };

    fn channel_label(workspace: &Workspace, channel: ChannelId) -> Option<&str> {
        workspace
            .labels
            .iter()
            .find(|(id, _)| *id == channel)
            .map(|(_, label)| label.as_str())
    }

    fn base_workspace(playbacks: usize, outputs: usize) -> Workspace {
        let mut workspace = Workspace::default();
        for i in 0..playbacks {
            workspace.scene.playbacks.push(PlaybackChannel::new(
                i,
                &format!("PB {}", i + 1),
                outputs / 2,
            ));
        }
        for i in 0..outputs {
            let mut output = OutputChannel::new(i, &format!("OUT {}", i + 1));
            output.eq = Some(Default::default());
            output.dynamics = Some(Default::default());
            workspace.scene.outputs.push(output);
        }
        workspace.scene.settings.output_link = vec![true; outputs / 2];
        workspace
    }

    fn ucx_topology() -> SurroundTopology {
        SurroundTopology {
            output_order: vec![1, 2, 3, 0, 4, 5, 6, 7, 8],
            matrix_uses_display_order: false,
        }
    }

    fn uc_topology() -> SurroundTopology {
        SurroundTopology {
            output_order: vec![1, 2, 3, 0, 4, 5, 6, 7, 8],
            matrix_uses_display_order: true,
        }
    }

    fn factory(layout: SurroundLayout, topology: &SurroundTopology) -> Workspace {
        factory_workspace(&base_workspace(18, 18), layout, topology).unwrap()
    }

    #[test]
    fn factory_workspace_starts_from_a_clean_mixer() {
        let mut base = base_workspace(18, 18);
        let mut input = InputChannel::new(0, "IN1", ChannelType::Line, 9);
        input.eq = Some(Default::default());
        input.dynamics = Some(Default::default());
        base.scene.inputs.push(input);
        // Make every mixer area dirty so the factory reset is observable.
        base.scene.inputs[0].volumes[0] = 1.0;
        base.scene.inputs[0].fx_send_db = 0.0;
        base.scene.inputs[0].mute = true;
        base.scene.inputs[0].phase = true;
        base.scene.inputs[0].trim = -6.0;
        if let Some(eq) = base.scene.inputs[0].eq.as_mut() {
            eq.enabled = true;
        }
        if let Some(dynamics) = base.scene.inputs[0].dynamics.as_mut() {
            dynamics.on = true;
        }
        base.scene.playbacks[8].volumes[4] = 1.0;
        base.scene.playbacks[8].fx_send_db = 0.0;
        base.scene.playbacks[8].trim = -6.0;
        base.scene.playbacks[8].width = 0.0;
        base.scene.playbacks[8].ms_proc = true;
        base.scene.playbacks[8].mute = true;
        base.scene.outputs[0].volume = 2.0;
        base.scene.outputs[0].fx_return_db = 0.0;
        base.scene.outputs[0].loopback = true;
        if let Some(eq) = base.scene.outputs[0].eq.as_mut() {
            eq.enabled = true;
        }
        if let Some(dynamics) = base.scene.outputs[0].dynamics.as_mut() {
            dynamics.on = true;
        }
        base.scene.settings.reverb = Some(Reverb {
            on: true,
            ..Default::default()
        });
        base.scene.settings.dim = true;
        base.scene.settings.speaker_b = true;
        base.scene.settings.mono = true;
        base.scene.settings.mute_fx = true;
        base.scene.settings.pitch_percent = 3.0;
        base.scene.settings.main_out = Some(1);
        base.scene.matrix_overrides.push(MatrixOverride {
            source: 18,
            block: 3,
            gain: 1.0,
        });
        base.labels
            .push((ChannelId::Input(0), "Old input".into()));
        base.groups[1].members.push(ChannelId::Output(0));
        base.snapshots[0].name = "Old".into();
        base.snapshots[0].scene = Some(base.scene.clone());
        base.view.show_original_names = true;
        base.view.matrix_mono = false;

        let workspace =
            factory_workspace(&base, SurroundLayout::Direct51, &ucx_topology()).unwrap();

        assert!(workspace.scene.inputs[0].volumes.iter().all(|&v| v == 0.0));
        assert_eq!(workspace.scene.inputs[0].fx_send_db, -65.0);
        assert!(!workspace.scene.inputs[0].mute);
        assert!(!workspace.scene.inputs[0].phase);
        assert_eq!(workspace.scene.inputs[0].trim, 0.0);
        assert!(!workspace.scene.inputs[0].eq.unwrap().enabled);
        assert!(!workspace.scene.inputs[0].dynamics.unwrap().on);

        let unused = &workspace.scene.playbacks[8];
        assert!(unused.volumes.iter().all(|&v| v == 0.0));
        assert_eq!(unused.fx_send_db, -65.0);
        assert_eq!(unused.trim, 0.0);
        assert_eq!(unused.width, 1.0);
        assert!(!unused.ms_proc);
        assert!(!unused.mute);

        let output = &workspace.scene.outputs[0];
        assert_eq!(output.volume, 1.0);
        assert_eq!(output.fx_return_db, -65.0);
        assert!(!output.loopback);
        assert!(!output.eq.unwrap().enabled);
        assert!(!output.dynamics.unwrap().on);

        assert!(!workspace.scene.settings.reverb.unwrap().on);
        assert!(!workspace.scene.settings.dim);
        assert!(!workspace.scene.settings.speaker_b);
        assert!(!workspace.scene.settings.mono);
        assert!(!workspace.scene.settings.mute_fx);
        assert_eq!(workspace.scene.settings.pitch_percent, 0.0);
        assert_eq!(workspace.scene.settings.main_out, None);
        assert!(workspace.labels.iter().all(|(id, _)| *id != ChannelId::Input(0)));
        assert!(workspace.groups[1].members.is_empty());
        assert!(workspace.snapshots.iter().all(|slot| slot.scene.is_none()));
        assert!(!workspace.view.show_original_names);
        assert!(workspace.view.matrix_mono);
        assert!(!workspace.view.fx_panel);
    }

    #[test]
    fn direct_51_keeps_stereo_pairs_and_mono_centre_and_lfe() {
        let workspace = factory(SurroundLayout::Direct51, &ucx_topology());

        assert_eq!(workspace.name, "5.1 Direct");
        assert!(workspace.surround_51);
        assert!(!workspace.surround_71);

        for (source, pair) in [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3)] {
            assert_eq!(workspace.scene.playbacks[source].volumes[pair], 1.0);
        }
        for source in 0..6 {
            for pair in 0..9 {
                if ![(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3)].contains(&(source, pair)) {
                    assert_eq!(
                        workspace.scene.playbacks[source].volumes[pair], 0.0,
                        "unexpected send from playback {source} to pair {pair}"
                    );
                }
            }
        }
        assert!(workspace.scene.playbacks[2].split);
        assert!(workspace.scene.playbacks[3].split);
        assert!(!workspace.scene.playbacks[0].split);
        assert!(!workspace.scene.playbacks[1].split);
        assert!(!workspace.scene.playbacks[4].split);
        assert!(!workspace.scene.playbacks[5].split);

        assert!(workspace.scene.settings.output_link[1]);
        assert!(!workspace.scene.settings.output_link[2]);
        assert!(workspace.scene.settings.output_link[3]);

        assert_eq!(channel_label(&workspace, ChannelId::Playback(0)), Some("Front L"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(1)), Some("Front R"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(2)), Some("Center"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(3)), Some("LFE"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(4)), Some("Side L"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(5)), Some("Side R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(2)), Some("Front L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(3)), Some("Front R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(4)), Some("Center"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(5)), Some("LFE"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(6)), Some("Side L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(7)), Some("Side R"));

        assert_eq!(
            workspace.groups[0].members,
            vec![
                ChannelId::Output(2),
                ChannelId::Output(3),
                ChannelId::Output(4),
                ChannelId::Output(5),
                ChannelId::Output(6),
                ChannelId::Output(7),
            ]
        );
        assert!(workspace.scene.matrix_overrides.is_empty());
        assert_eq!(workspace.view.sel_out, 1);
        assert_eq!(workspace.view.sel_out_side, 0);
    }

    #[test]
    fn direct_51_uses_display_order_volume_indices_on_the_uc() {
        let workspace = factory(SurroundLayout::Direct51, &uc_topology());

        for (source, pair) in [(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2)] {
            assert_eq!(workspace.scene.playbacks[source].volumes[pair], 1.0);
        }
        // Output channels stay in internal Phones-first order.
        assert_eq!(channel_label(&workspace, ChannelId::Output(2)), Some("Front L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(3)), Some("Front R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(7)), Some("Side R"));
        assert_eq!(workspace.view.sel_out, 1);
    }

    #[test]
    fn direct_71_maps_front_centre_lfe_rear_and_side() {
        let workspace = factory(SurroundLayout::Direct71, &ucx_topology());

        assert_eq!(workspace.name, "7.1 Direct");
        assert!(!workspace.surround_51);
        assert!(workspace.surround_71);

        for (source, pair) in [
            (0, 1),
            (1, 1),
            (2, 2),
            (3, 2),
            (4, 3),
            (5, 3),
            (6, 0),
            (7, 0),
        ] {
            assert_eq!(workspace.scene.playbacks[source].volumes[pair], 1.0);
        }
        assert!(workspace.scene.playbacks[2].split);
        assert!(workspace.scene.playbacks[3].split);
        assert!(!workspace.scene.playbacks[0].split);
        assert!(!workspace.scene.playbacks[4].split);
        assert!(!workspace.scene.playbacks[6].split);

        assert!(workspace.scene.settings.output_link[1]);
        assert!(!workspace.scene.settings.output_link[2]);
        assert!(workspace.scene.settings.output_link[3]);
        assert!(workspace.scene.settings.output_link[0]);

        assert_eq!(channel_label(&workspace, ChannelId::Playback(4)), Some("Rear L"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(5)), Some("Rear R"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(6)), Some("Side L"));
        assert_eq!(channel_label(&workspace, ChannelId::Playback(7)), Some("Side R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(6)), Some("Rear L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(7)), Some("Rear R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(0)), Some("Side L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(1)), Some("Side R"));

        assert_eq!(
            workspace.groups[0].members,
            vec![
                ChannelId::Output(2),
                ChannelId::Output(3),
                ChannelId::Output(4),
                ChannelId::Output(5),
                ChannelId::Output(6),
                ChannelId::Output(7),
                ChannelId::Output(0),
                ChannelId::Output(1),
            ]
        );
        assert!(workspace.scene.matrix_overrides.is_empty());
    }

    #[test]
    fn downmix_51_to_40_centres_center_and_lfe_without_a_sub_output() {
        let workspace = factory(SurroundLayout::Downmix51To40, &ucx_topology());

        assert_eq!(workspace.name, "5.1 → 4.0");
        assert!(workspace.surround_51);
        assert!(!workspace.surround_71);

        assert_eq!(workspace.scene.playbacks[0].volumes[1], 1.0);
        assert_eq!(workspace.scene.playbacks[1].volumes[1], 1.0);
        assert_eq!(workspace.scene.playbacks[2].volumes[1], 0.707);
        assert_eq!(workspace.scene.playbacks[3].volumes[1], 0.707);
        assert_eq!(workspace.scene.playbacks[4].volumes[2], 1.0);
        assert_eq!(workspace.scene.playbacks[5].volumes[2], 1.0);

        let overrides = &workspace.scene.matrix_overrides;
        assert_eq!(overrides.len(), 2);
        assert!(overrides.iter().any(|cell| {
            cell.source == 20 && cell.block == 1 && (cell.gain - 0.707).abs() < 1e-6
        }));
        assert!(overrides.iter().any(|cell| {
            cell.source == 21 && cell.block == 0 && (cell.gain - 0.707).abs() < 1e-6
        }));

        assert!(workspace.scene.settings.output_link[1]);
        assert!(workspace.scene.settings.output_link[2]);
        assert_eq!(channel_label(&workspace, ChannelId::Output(2)), Some("Front L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(3)), Some("Front R"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(4)), Some("Rear L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(5)), Some("Rear R"));
        assert_eq!(
            workspace.groups[0].members,
            vec![
                ChannelId::Output(2),
                ChannelId::Output(3),
                ChannelId::Output(4),
                ChannelId::Output(5),
            ]
        );
        // LFE is mixed to both front channels only; there is no sub pair.
        assert_eq!(
            workspace.scene.playbacks[3]
                .volumes
                .iter()
                .filter(|&&v| v > 0.0)
                .count(),
            1
        );
        assert!(!workspace.groups[0].members.contains(&ChannelId::Output(6)));
        assert!(!workspace.groups[0].members.contains(&ChannelId::Output(7)));
        assert!(workspace
            .scene
            .matrix_overrides
            .iter()
            .all(|cell| cell.block < 4));
    }

    #[test]
    fn downmix_71_to_40_adds_rear_and_side_into_the_rear_pair() {
        let workspace = factory(SurroundLayout::Downmix71To40, &ucx_topology());

        assert_eq!(workspace.name, "7.1 → 4.0");
        assert!(!workspace.surround_51);
        assert!(workspace.surround_71);

        assert_eq!(workspace.scene.playbacks[0].volumes[1], 1.0);
        assert_eq!(workspace.scene.playbacks[1].volumes[1], 1.0);
        assert_eq!(workspace.scene.playbacks[2].volumes[1], 0.707);
        assert_eq!(workspace.scene.playbacks[3].volumes[1], 0.707);
        for source in 4..=7 {
            assert_eq!(
                workspace.scene.playbacks[source].volumes[2], 1.0,
                "rear and side sources must sum into the rear pair at unity"
            );
        }
        assert_eq!(workspace.scene.matrix_overrides.len(), 2);
        assert_eq!(
            workspace.groups[0].members,
            vec![
                ChannelId::Output(2),
                ChannelId::Output(3),
                ChannelId::Output(4),
                ChannelId::Output(5),
            ]
        );
        assert_eq!(channel_label(&workspace, ChannelId::Output(4)), Some("Rear L"));
        assert_eq!(channel_label(&workspace, ChannelId::Output(5)), Some("Rear R"));
    }

    #[test]
    fn downmix_51_on_the_uc_uses_display_order_volume_indices() {
        let workspace = factory(SurroundLayout::Downmix51To40, &uc_topology());

        assert_eq!(workspace.scene.playbacks[2].volumes[0], 0.707);
        assert_eq!(workspace.scene.playbacks[3].volumes[0], 0.707);
        assert_eq!(workspace.scene.playbacks[4].volumes[1], 1.0);
        assert_eq!(workspace.scene.playbacks[5].volumes[1], 1.0);
        assert_eq!(workspace.scene.matrix_overrides.len(), 2);
    }

    #[test]
    fn factory_surround_workspaces_round_trip_through_tcws() {
        let dir = std::env::temp_dir().join(format!(
            "totalcontrol-surround-workspace-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("seven-one-downmix.tcws");

        let workspace = factory(SurroundLayout::Downmix71To40, &ucx_topology());
        save(&path, &workspace).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.name, "7.1 → 4.0");
        assert!(loaded.surround_71);
        assert_eq!(loaded.scene.playbacks[4].volumes[2], 1.0);
        assert_eq!(loaded.scene.matrix_overrides.len(), 2);
        assert_eq!(channel_label(&loaded, ChannelId::Playback(4)), Some("Rear L"));
        assert_eq!(loaded.groups[0].members, workspace.groups[0].members);
        assert!(loaded.view.matrix_mono);
        assert_eq!(loaded.view.sel_out_side, 0);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn direct_surround_does_not_add_bass_management() {
        for layout in SurroundLayout::ALL {
            let workspace = factory(layout, &ucx_topology());
            assert!(workspace.scene.outputs.len() >= 8);
            assert_eq!(
                workspace.scene.playbacks[3]
                    .volumes
                    .iter()
                    .filter(|&&v| v > 0.0)
                    .count(),
                1,
                "LFE must have exactly one pair route, never a dedicated sub path"
            );
            assert!(workspace.scene.matrix_overrides.len() <= 2);
        }
    }

    #[test]
    fn workspace_round_trips_through_json() {
        let mut workspace = Workspace::default();
        workspace.name = "5.1".into();
        workspace.labels.push((ChannelId::Playback(0), "Front L".into()));
        workspace.groups[0].members.push(ChannelId::Output(0));
        workspace.groups[0].fader_linked = true;
        workspace.view.view = View::Mixer;
        workspace.snapshots[0].name = "Surround".into();

        let json = serde_json::to_string(&workspace).unwrap();
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "5.1");
        assert_eq!(back.labels[0].1, "Front L");
        assert!(back.groups[0].fader_linked);
        assert_eq!(back.snapshots[0].name, "Surround");
        assert_eq!(back.view.view, View::Mixer);
    }

    #[test]
    fn save_and_load_round_trips_atomically() {
        let dir = std::env::temp_dir().join(format!(
            "totalcontrol-workspace-{}-{}",
            std::process::id(),
            SCHEMA
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.tcws");
        let mut workspace = Workspace::default();
        workspace.name = "Test".into();
        save(&path, &workspace).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.name, "Test");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn newer_schema_is_rejected() {
        let mut workspace = Workspace::default();
        workspace.schema = SCHEMA + 1;
        let json = serde_json::to_string(&workspace).unwrap();
        let parse: Workspace = serde_json::from_str(&json).unwrap();
        assert!(parse.schema > SCHEMA);
    }

    #[test]
    fn extension_is_added_once() {
        assert_eq!(with_extension(PathBuf::from("a")), PathBuf::from("a.tcws"));
        assert_eq!(
            with_extension(PathBuf::from("a.tcws")),
            PathBuf::from("a.tcws")
        );
    }

    #[test]
    fn an_old_workspace_restores_its_group_ganged() {
        let mut workspace = Workspace::default();
        workspace.temporary_group = vec![ChannelId::Input(0), ChannelId::Input(4)];
        let mut json = serde_json::to_value(&workspace).unwrap();
        json.as_object_mut().unwrap().remove("fader_ganged");
        let back: Workspace = serde_json::from_value(json).unwrap();
        assert!(back.fader_ganged, "old files always stored a ganged group");
    }
}
