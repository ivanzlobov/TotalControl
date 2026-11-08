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

use crate::app::{RowMode, View};
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
    pub quick_channel: ChannelId,
    pub show_original_names: bool,
    pub matrix_mono: bool,
    pub active_snapshot: Option<usize>,
    pub active_layout: Option<usize>,
    pub collapsed: HashSet<ChannelId>,
    /// The Talkback button is latched on in this workspace.
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

    #[test]
    fn workspace_round_trips_through_json() {
        let mut workspace = Workspace::default();
        workspace.name = "5.1".into();
        workspace.labels.push((ChannelId::Playback(0), "FL".into()));
        workspace.groups[0].members.push(ChannelId::Output(0));
        workspace.groups[0].fader_linked = true;
        workspace.view.view = View::Mixer;
        workspace.snapshots[0].name = "Surround".into();

        let json = serde_json::to_string(&workspace).unwrap();
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "5.1");
        assert_eq!(back.labels[0].1, "FL");
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
}
