# TotalControl architecture

This is the central architecture document for TotalControl: the runtime
layers, the device model, the control-surface boundary and the driver path.
The other specifications extend it with device-specific or user-facing
detail.

## Contents

1. [Document map](#1-document-map)
2. [Layers](#2-layers)
3. [Device model](#3-device-model)
4. [Device backends](#4-device-backends)
5. [Device service and mirror](#5-device-service-and-mirror)
6. [Hardware to display](#6-hardware-to-display)
7. [Display to hardware](#7-display-to-hardware)
8. [Control surfaces](#8-control-surfaces)
9. [State, scenes and persistence](#9-state-scenes-and-persistence)
10. [Driver and latency architecture](#10-driver-and-latency-architecture)
11. [Development tools](#11-development-tools)
12. [Extension points](#12-extension-points)
13. [Code map](#13-code-map)

## 1. Document map

| Document | Owns |
|---|---|
| `TOTALCONTROL-ARCHITECTURE.md` | runtime structure, threads, data flow, state ownership, control-surface boundary, driver and latency model |
| `spec/TOTALCONTROL-GUI-SPEC.md` | layout, controls, panels, gestures, motion and visible state |
| `spec/FIREFFACE-UCX-PROTOCOL-SPEC.md` | Fireface UCX transport, registers, writes and readback |
| `spec/FIREFFACE-UC-PROTOCOL-SPEC.md` | Fireface UC transport, registers, writes and readback |
| `spec/BABYFACE-PRO-PROTOCOL-SPEC.md` | Babyface Pro FS transport, ALSA surface, USB protocol and DSP blocks |
| `spec/TOTALCONTROL-OSC-SPEC.md` | TotalMix FX OSC address space, pages, scales and feedback |
| `TOTALCONTROL-Manual.md` | installation, setup, use and troubleshooting |
| `DEVELOPMENT-TOOLS.md` | development-only backends, probes and capture helpers |

The architecture document does not contain protocol encodings, pixel geometry
or open questions. Unresolved work belongs in `work/`.

## 2. Layers

```text
 control surfaces                    device backends
 GUI, keyboard, tray, OSC, ...       hwdep, ALSA, vendor USB
        |                                    ^
        v                                    |
   Message + update()  ->  mixer model  ->  RmeDevice
                              |
                              v
                        view / canvases
```

| Layer | Responsibility | Main code |
|---|---|---|
| Mixer model | channels, routing, settings, scenes | `totalcontrol-core` |
| Device backend | read/write the model over one transport | `totalcontrol-core`, `DeviceHandle` |
| Device service | own the real handle on its own thread | `totalcontrol-gui/src/device_service.rs` |
| Application state | selection, view, undo, prefs, host scene | `totalcontrol-gui/src/app.rs` |
| Control adapters | translate external input into `Message` | `app.rs`, `osc.rs`, `osc_rme.rs` |
| View | build the iced element tree from state | `app.rs`, `widgets/` |

The mixer model is the centre. Control surfaces do not talk to the device,
and device backends do not know about control surfaces.

## 3. Device model

`totalcontrol-core` defines the model and the `RmeDevice` trait. The trait
covers matrix volumes and pans, mute/solo, preamps, EQ, dynamics, FX, clock,
scenes, event polling and meters.

The serialisable model is a `Scene`:

- `inputs`: `InputChannel`
- `playbacks`: `PlaybackChannel`
- `outputs`: `OutputChannel`
- `settings`: `DeviceSettings`
- `matrix_overrides`: per-cell values the pair model cannot express

The device model owns routing, faders, preamps, EQ/dynamics, FX, meters, clock
options and scenes. The backend reports capabilities and the GUI exposes only
the controls that connection can drive.

## 4. Device backends

`DeviceHandle` is the runtime union of the available backends:

| Variant | Transport | Support |
|---|---|---|
| `Fireface` | Fireface UC/UCX over the kernel hwdep node | production |
| `BabyfacePro` | Babyface ALSA controls | development fallback |
| `BabyfaceProUsb` | direct libusb vendor protocol | development only |
| `Mirror` | UI-side copy owned by the device service | production |
| `Mock` | simulation | development and tests |

Every variant implements `RmeDevice`. The production installation is the
kernel driver plus hwdep path; the other real-device backends are documented
in `DEVELOPMENT-TOOLS.md`.

## 5. Device service and mirror

In the running application the device normally lives on the
`totalcontrol-device-service` thread. The UI holds a `Mirror` instead of the
real handle.

The service loop:

1. drains `Command`s from the UI;
2. calls `device.poll_events()`;
3. reads input, playback and output meters;
4. captures the model every 50 ms and publishes a `Snapshot` when it changed;
5. publishes meters separately through a shared `MeterSnapshot`.

The UI sends commands such as `SetVolume`, `SetMute`, `ApplyScene`, or a
`Run(closure)` for controls without a dedicated command. Setters on the mirror
queue the command and return. The local model changes when the confirming
snapshot arrives, not optimistically.

`TOTALCONTROL_DEVICE_SERVICE=0` keeps the device on the UI thread. Unit tests
use the direct path unless they attach the service explicitly.

Meters are not part of the model snapshot. The fader canvases read the shared
meter bus directly. `MeterWake` is only an edge that starts canvas animation
after silence; it does not rebuild the view for each meter packet.

## 6. Hardware to display

```text
front panel / device state
  -> kernel hwdep event queue or ALSA control / USB status poll
  -> RmeDevice::poll_events()
  -> model fields
  -> Snapshot (device service)
  -> Message::DeviceUpdated
  -> Mirror::update()
  -> view()
```

Backend-specific behaviour:

- **Fireface UCX over hwdep**: `Fireface::poll_events()` drains `EV_STATUS`,
  `EV_STATE` and meter events. The UCX heartbeat keeps the state stream
  flowing. Front-panel gain, output volume, DIM and Mono updates reach
  `apply_ucx_state_word()` and the corresponding model fields.
- **Legacy ALSA backends**: `poll_events()` refreshes output volume and preamp
  controls at a 100 ms interval.
- **Babyface USB**: `panel_tick()` reads the `0x17` panel status and runs the
  host-side panel state machine.
- **Meters**: the service copies device meter pages into `MeterSnapshot` at the
  polling rate. Peak hold lives there, so a transient shorter than a canvas
  frame is still visible.

The view reads the model through `Mirror` getters, which never lock or block on
the device thread.

## 7. Display to hardware

```text
widget event
  -> Message (for example VolumeChanged, Mute, EqBandGain)
  -> update()
  -> model mutation helpers
  -> state.device.set_*()
  -> RmeDevice backend
  -> registers, vendor requests or ALSA controls
```

Examples:

- **Fireface UCX over hwdep**: `Fireface::set_volume()` computes the matrix
  coefficient or fader record and queues IF1 register words. Live writes and
  scene applies use the paced batch writer; the UI thread does not sleep
  through the pacing.
- **Fireface UC**: the same model operation becomes an EP0 vendor request.
- **ALSA backends**: the operation becomes an ALSA control write.
- **USB backend**: the operation becomes a proprietary vendor request.

All control adapters enter through `Message`, so they share the same clamps,
undo handling, host-scene merge and autosave. The update loop captures the
scene before an undoable change and merges the confirmed device state after the
command, either immediately on the direct path or after the next snapshot with
the service.

## 8. Control surfaces

A control surface is an adapter at the `Message` boundary.

### GUI, keyboard and tray

Widgets and global subscriptions emit `Message` values. They are the reference
path for the other adapters.

### OSC

OSC is optional. Without `--osc`, no worker or feedback sender is installed.

Incoming:

```text
UDP -> osc::worker -> osc_rme::parse()
    -> RmeState (bank, selected channel, submix)
    -> Vec<Message>
    -> update()
```

Outgoing:

```text
notify_osc() / notify_osc_raw()
    -> OscOutbound
    -> worker
    -> RmeFeedback::translate()
    -> TotalMix paths
    -> UDP
```

`RmeFeedback` keeps a mirror of the last known values and the page state. It
resends the current page after `/setBankStart`, `/setOffsetInBank` or
`/setSubmix`, and suppresses sends outside the visible bank or selected submix.

OSC uses the TotalMix FX dialect because its bank/page/submix model maps onto
the mixer's channel rows and send matrix. The address space is defined in
`spec/TOTALCONTROL-OSC-SPEC.md`.

## 9. State, scenes and persistence

The device model is the runtime state. It is captured as a `Scene` — the
internal name for one mixer state — for:

- the auto-saved scene;
- snapshot slots;
- undo and redo;
- host-scene merge after a control change;
- scene import and export.

The GUI also persists interface state: window layout, sidebar state, channel
names, Control Room preferences and device identity. The TUI and GUI share the
mixer scene, so the last writer wins.

A named **workspace** bundles the current scene, the eight snapshots and their
names, channel labels, the four groups, the six layouts, view state, Control
Room state, virtual-surround settings and window size in one `.tcws` JSON
file. It is the portable host-side configuration; `gui.json` remains the
per-installation preference file, `auto.json` the crash-recovery scene, and
the unit's six setups remain device state.  A workspace includes the sample
rate, pitch/varispeed and active Dim and Speaker B states; the unit's six
device setups and the device serial number remain outside it.

Factory surround Workspaces are `Workspace` values built in memory and
applied through the same `apply_workspace` scene path as a `.tcws` file.  They
start from a clean mixer (no sends, FX returns, Reverb/Echo, EQ, dynamics,
mutes, solos, labels, groups, layouts or snapshots) while retaining the
device topology and device-owned clock, rate, references and preamps.  They
have no file path, so the next Save opens Save As.  The layout code only
writes the mixer scene, labels and group; it does not add a second
persistence format or a device setup.

The direct 5.1/7.1 layouts use pair sends.  The 5.1/7.1 → 4.0 layouts also
use mono matrix overrides for Center and LFE into both front channels.  The
UC and UCX both apply these cells: the UCX writes `0x4000 +
dsp_output*0x40 + source`, while the UC uses its existing matrix path.  A
selected split output channel is stored with its pair and side, so the UI can
address and display the mono bus rather than both sides of the pair.

Active Talkback, Listenback, External Input, PFL and cue selections are
workspace state.  A snapshot stores the mixer scene; the workspace also
stores the active monitoring controls.

Startup sync compares the saved scene with the device where the backend can
read state back.  Groups without readback are written from the saved scene.

For the UCX the window is built from the saved scene model without writing the
device.  The pre-connect dump is requested asynchronously, collected while the
window opens, and decoded before any scene write.  This keeps dump words and
scene-write echoes from mixing on the same state stream.  If the decoded state
matches, the scene is written and startup continues; otherwise the mismatch
dialog is shown.  The direct device handle stays in the UI until that
comparison is resolved, then moves to the device service.

A connect restore leaves output registers that match the pre-connect dump
unchanged.  A reapply after a runtime rate change or stream restart writes the
full scene instead: the unit may have reloaded its own state, so the startup
dump no longer describes the registers, and skipping matching values can leave
effects or routing from the old rate active.

## 10. Driver and latency architecture

The production transport is the kernel module `snd-fireface-uc`. It owns USB
enumeration, alternate settings, the isochronous and interrupt URB engines,
the ALSA PCM device and one hwdep control node per card. TotalControl owns the
device model and writes through hwdep; PipeWire and the DAW use the ALSA PCM.

The audio path has four parts:

```text
host ring -> USB URB queue -> device lead -> converters
```

The host ring is the ALSA buffer managed by PipeWire or the application. The
USB queue is the driver's feedback delay and queued URBs. The device lead is a
fixed vendor-path delay. The converters add a fixed small delay.

The GUI latency line reports the application/ALSA budget, not a measured
acoustic round trip. It derives the device side from the kernel parameters and
adds the host ring. Measured cable and loopback evidence is kept outside the
specifications.

The vendor path and the class-compliant path are mutually exclusive device
personalities. In class-compliant mode the stock `snd-usb-audio` driver drives
audio and the vendor mixer is unavailable.

## 11. Development tools

Everything except the kernel driver plus hwdep is a development tool:
the Babyface ALSA backend, the direct libusb backend, the ALSA PCM plugin,
probe programs, capture decoders and mock modes.

They are not user-facing installation options. `DEVELOPMENT-TOOLS.md`
describes what each tool does and how to run it.

## 12. Extension points

### Another OSC dialect

Add a module that parses its addresses into `Message`, keeps its own navigation
state, translates `OscOutbound` into its feedback paths, and runs as a
subscription on its own port.

### MIDI SysEx, MCU or another transport

The inbound side is generic: any transport that produces `Message` values can
drive the mixer. A MIDI adapter needs a transport worker, SysEx framing and
7-bit packing, a decoder and a feedback encoder.

The feedback side still carries the names of its first user: `OscOutbound`
contains internal address strings and the application stores one `osc_tx`.
Several control surfaces would be cleaner with a control-surface-neutral
`ControlEvent`, typed variants and a list or broadcast of feedback sinks.

### Another device backend

A new device model or transport implements `RmeDevice` and adds a
`DeviceHandle` variant. Control surfaces do not change.

## 13. Code map

| Area | Location |
|---|---|
| Model and `RmeDevice` | `ui/totalcontrol/totalcontrol-core/src/device.rs` |
| Scene and persistence | `totalcontrol-core/src/scene.rs` |
| Fireface UC/UCX hwdep backend | `totalcontrol-core/src/fireface/` |
| USB resource layer | `kernel/` |
| Device service | `totalcontrol-gui/src/device_service.rs` |
| Application state and update loop | `totalcontrol-gui/src/app.rs` |
| OSC bridge | `totalcontrol-gui/src/osc.rs`, `osc_rme.rs` |
| GUI widgets | `totalcontrol-gui/src/widgets/` |
| Developer tools | `docs/DEVELOPMENT-TOOLS.md` |
