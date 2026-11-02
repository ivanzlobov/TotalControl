//! Device service — the first stage of the device/UI split.
//!
//! The device currently lives on the UI thread.  This module introduces the
//! boundary that will own it in the next stage:
//!
//! - one thread owns the `DeviceHandle`;
//! - the UI will send [`Command`]s instead of calling the device;
//! - the UI will read a [`Snapshot`] of the channel model;
//! - meter canvases will read [`MeterSnapshot`] without a view rebuild.
//!
//! Stage 1 keeps the service off the running path (the UI still uses the
//! device directly) so the tree stays green while the interface is wired in
//! stage 2.  The mock tests exercise the whole boundary.
//!
//! Temporary switch: `TUXMIX_DEVICE_SERVICE=1` is reserved for the stage-2
//! wiring; it is deliberately not read yet.

#![allow(dead_code)]

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tuxmix_core::{ChannelId, RmeDevice, Scene};

use crate::app::DeviceHandle;

/// A command sent from the UI to the device service.
#[derive(Debug)]
pub enum Command {
    SetVolume(ChannelId, usize, f32),
    SetMute(ChannelId, bool),
    ApplyScene(Box<Scene>),
    Stop,
}

/// Static properties of the device, captured once when the service starts.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub model_name: String,
    pub canonical_model: String,
    pub capabilities: tuxmix_core::DeviceCapabilities,
    pub protocol_features: bool,
    pub output_pair_count: usize,
    pub output_display_order: Vec<usize>,
    pub is_uc: bool,
    pub is_ucx: bool,
    pub is_uc_family: bool,
    pub is_mock: bool,
}

/// The channel model as the UI should draw it.  Replaced wholesale when the
/// service sees a change; cheap enough to clone at the UI's housekeeping
/// rate.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub scene: Scene,
    pub info: DeviceInfo,
    pub connected: bool,
    pub at: Instant,
}

/// The UI-side copy of the model.
///
/// Getters return references into the local copy, so the view never locks or
/// blocks on the device thread.  Setters enqueue a command; the next
/// [`Snapshot`] replaces the local copy.  The service updates it from
/// `DeviceService::take_updates`.
pub struct Mirror {
    scene: Scene,
    info: DeviceInfo,
    commands: Sender<Command>,
}

impl Mirror {
    pub fn new(initial: Snapshot, commands: Sender<Command>) -> Self {
        Self {
            scene: initial.scene,
            info: initial.info,
            commands,
        }
    }

    /// Apply a snapshot published by the service.  Returns false once the
    /// service has stopped.
    pub fn update(&mut self, snapshot: Snapshot) -> bool {
        self.scene = snapshot.scene;
        self.info = snapshot.info;
        true
    }

    pub fn inputs(&self) -> &[tuxmix_core::InputChannel] {
        &self.scene.inputs
    }

    pub fn playbacks(&self) -> &[tuxmix_core::PlaybackChannel] {
        &self.scene.playbacks
    }

    pub fn outputs(&self) -> &[tuxmix_core::OutputChannel] {
        &self.scene.outputs
    }

    pub fn settings(&self) -> &tuxmix_core::DeviceSettings {
        &self.scene.settings
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    pub fn set_volume(&self, channel: ChannelId, output: usize, value: f32) -> bool {
        self.commands
            .send(Command::SetVolume(channel, output, value))
            .is_ok()
    }

    pub fn set_mute(&self, channel: ChannelId, mute: bool) -> bool {
        self.commands.send(Command::SetMute(channel, mute)).is_ok()
    }
}

/// Meter values written by the service and read by the fader canvases.
///
/// `None` for `at` means no meter packet has arrived yet.  The canvases use
/// `at` to interpolate between snapshots without a UI message.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeterSnapshot {
    pub inputs: [f32; 18],
    pub playbacks: [f32; 18],
    pub outputs: [f32; 18],
    pub fx: [f32; 2],
    pub at: Option<Instant>,
}

/// A running device service.
pub struct DeviceService {
    commands: Sender<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
    meters: Arc<Mutex<MeterSnapshot>>,
    updates: Option<Receiver<Snapshot>>,
    join: Option<JoinHandle<()>>,
}

/// Temporary stage-2 switch.  The running UI still uses the device directly
/// until the command path and snapshot reads are wired; when that lands,
/// `TUXMIX_DEVICE_SERVICE=1` will select the service for a live comparison.
pub fn requested() -> bool {
    std::env::var_os("TUXMIX_DEVICE_SERVICE").is_some()
}

impl DeviceService {
    /// Take ownership of `device` and start the service thread.
    pub fn spawn(mut device: DeviceHandle) -> Self {
        let initial = device.capture_scene();
        let info = capture_info(&device);
        let snapshot = Arc::new(Mutex::new(Snapshot {
            scene: initial,
            info,
            connected: device.is_connected(),
            at: Instant::now(),
        }));
        let meters = Arc::new(Mutex::new(MeterSnapshot::default()));
        let (commands, rx) = mpsc::channel();
        let (updates_tx, updates_rx) = mpsc::channel();

        let thread_snapshot = Arc::clone(&snapshot);
        let thread_meters = Arc::clone(&meters);
        let join = thread::Builder::new()
            .name("tuxmix-device-service".into())
            .spawn(move || {
                run(
                    &mut device,
                    rx,
                    thread_snapshot,
                    thread_meters,
                    updates_tx,
                );
            })
            .expect("starting the device service");

        Self {
            commands,
            snapshot,
            meters,
            updates: Some(updates_rx),
            join: Some(join),
        }
    }

    /// The current model snapshot.
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().unwrap().clone()
    }

    /// Take the update receiver; the UI drains it and applies the snapshots
    /// to its `Mirror`.
    pub fn take_updates(&mut self) -> Option<Receiver<Snapshot>> {
        self.updates.take()
    }

    /// A clone of the command sender, for a `Mirror`.
    pub fn command_sender(&self) -> Sender<Command> {
        self.commands.clone()
    }

    /// The current meter snapshot.
    pub fn meters(&self) -> MeterSnapshot {
        *self.meters.lock().unwrap()
    }

    /// Queue a command.  Returns false once the service has stopped.
    pub fn send(&self, command: Command) -> bool {
        self.commands.send(command).is_ok()
    }

    /// Ask the thread to stop and wait for it.
    pub fn stop(mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for DeviceService {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn capture_info(device: &DeviceHandle) -> DeviceInfo {
    DeviceInfo {
        model_name: device.model_name().to_string(),
        canonical_model: device.canonical_model().to_string(),
        capabilities: device.capabilities(),
        protocol_features: device.protocol_features(),
        output_pair_count: device.output_pair_count(),
        output_display_order: device.output_display_order(),
        is_uc: device.is_uc_model(),
        is_ucx: device.is_ucx_model(),
        is_uc_family: device.is_uc_family(),
        is_mock: device.is_mock(),
    }
}

fn run(
    device: &mut DeviceHandle,
    rx: Receiver<Command>,
    snapshot: Arc<Mutex<Snapshot>>,
    meters: Arc<Mutex<MeterSnapshot>>,
    updates: Sender<Snapshot>,
) {
    let mut last_scene = Instant::now();
    let mut last_connected = device.is_connected();

    loop {
        // The timeout paces the polling loop at about 120 Hz while still
        // waking immediately for a command.
        match rx.recv_timeout(Duration::from_millis(8)) {
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(command) => apply(device, command),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        while let Ok(command) = rx.try_recv() {
            if matches!(command, Command::Stop) {
                return;
            }
            apply(device, command);
        }

        // State/status events feed the model and the front panel.
        let _ = device.poll_events();

        // Meters are published independently of the model snapshot and of
        // the UI's view tick.
        {
            let mut m = meters.lock().unwrap();
            let inputs = device.input_meters();
            let playbacks = device.device_playback_meters().unwrap_or_default();
            let outputs = device.device_output_meters().unwrap_or_default();
            let fx = device.fx_meters().unwrap_or([0.0, 0.0]);
            for (slot, value) in m.inputs.iter_mut().zip(inputs) {
                *slot = value;
            }
            for (slot, value) in m.playbacks.iter_mut().zip(playbacks) {
                *slot = value;
            }
            for (slot, value) in m.outputs.iter_mut().zip(outputs) {
                *slot = value;
            }
            m.fx = fx;
            m.at = Some(Instant::now());
        }

        // The model changes rarely (a control command, a front-panel move,
        // a gain readback).  A 20 Hz capture is plenty for the UI.
        if last_scene.elapsed() >= Duration::from_millis(50) {
            last_scene = Instant::now();
            let connected = device.is_connected();
            let update = {
                let mut s = snapshot.lock().unwrap();
                if connected != last_connected || s.at.elapsed() >= Duration::from_millis(50) {
                    s.scene = device.capture_scene();
                    s.connected = connected;
                    s.at = Instant::now();
                    last_connected = connected;
                    Some(s.clone())
                } else {
                    None
                }
            };
            if let Some(update) = update {
                let _ = updates.send(update);
            }
        }
    }
}

fn apply(device: &mut DeviceHandle, command: Command) {
    match command {
        Command::SetVolume(channel, output, value) => {
            let _ = device.set_volume(channel, output, value);
        }
        Command::SetMute(channel, mute) => {
            let _ = device.set_mute(channel, mute);
        }
        Command::ApplyScene(scene) => {
            let _ = device.apply_scene(&scene);
        }
        Command::Stop => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, DeviceService};
    use crate::app::DeviceHandle;
    use std::thread;
    use std::time::{Duration, Instant};
    use tuxmix_core::{ChannelId, MockBabyfacePro};

    fn mock() -> DeviceHandle {
        DeviceHandle::Mock(MockBabyfacePro::open_ucx().expect("mock opens"))
    }

    fn wait_for(mut check: impl FnMut() -> bool) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if check() {
                return true;
            }
            thread::sleep(Duration::from_millis(5));
        }
        false
    }

    #[test]
    fn the_service_publishes_a_snapshot_and_applies_commands() {
        let service = DeviceService::spawn(mock());

        assert!(
            wait_for(|| !service.snapshot().scene.outputs.is_empty()),
            "the initial snapshot arrives"
        );
        assert!(service.snapshot().connected);

        assert!(service.send(Command::SetVolume(ChannelId::Output(0), 0, 0.25)));
        assert!(
            wait_for(|| (service.snapshot().scene.outputs[0].volume - 0.25).abs() < 0.001),
            "the model snapshot follows the volume command"
        );

        assert!(service.send(Command::SetMute(ChannelId::Input(0), true)));
        assert!(
            wait_for(|| service.snapshot().scene.inputs[0].mute),
            "the model snapshot follows the mute command"
        );

        service.stop();
    }

    #[test]
    fn meters_are_published_without_the_model_changing() {
        let service = DeviceService::spawn(mock());
        assert!(
            wait_for(|| service.meters().at.is_some()),
            "the meter snapshot is written"
        );
        assert!(service.meters().inputs.iter().any(|v| *v >= 0.0));
        service.stop();
    }

    #[test]
    fn a_mirror_applies_updates_and_sends_commands() {
        let mut service = DeviceService::spawn(mock());
        let updates = service.take_updates().expect("the update receiver");
        let mut mirror = super::Mirror::new(service.snapshot(), service.command_sender());
        assert!(!mirror.outputs().is_empty());
        assert_eq!(mirror.info().output_pair_count, mirror.outputs().len() / 2);

        assert!(mirror.set_volume(ChannelId::Output(0), 0, 0.25));
        assert!(
            wait_for(|| {
                while let Ok(snapshot) = updates.try_recv() {
                    mirror.update(snapshot);
                }
                (mirror.outputs()[0].volume - 0.25).abs() < 0.001
            }),
            "the mirror follows the service snapshot"
        );
        service.stop();
    }
}
