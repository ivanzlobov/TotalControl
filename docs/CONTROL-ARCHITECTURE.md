# Mixer and control-surface architecture

Orientation note for the runtime path between the mixer UI, the device model
and the hardware. The protocol and UI specifications remain authoritative:

- [`spec/TUXMIX-SPEC.md`](spec/TUXMIX-SPEC.md) — visible UI, panels, state and
  persistence
- [`spec/OSC-CONTROL-SPEC.md`](spec/OSC-CONTROL-SPEC.md) — TotalMix FX OSC
  address space
- [`spec/UCX-CONTROL-SPEC.md`](spec/UCX-CONTROL-SPEC.md) and
  [`spec/UC-CONTROL-SPEC.md`](spec/UC-CONTROL-SPEC.md) — device protocol

## 1. Layers

```text
 control surfaces                    device backends
 GUI, keyboard, tray, OSC, ...       ALSA, hwdep, vendor USB
        |                                    ^
        v                                    |
   Message + update()  ->  mixer model  ->  RmeDevice
                              |
                              v
                        view / canvases
```

The mixer model is the centre. Control surfaces do not talk to the device, and
device backends do not know about control surfaces.

| Layer | Responsibility | Main code |
|---|---|---|
| Mixer model | Channels, routing, settings, scenes | `tuxmix-core` |
| Device backend | Read/write the model over one transport | `tuxmix-core`, `DeviceHandle` |
| Device service | Own the real handle on its own thread | `tuxmix-gui/src/device_service.rs` |
| Application state | Selection, view, undo, prefs, host scene | `tuxmix-gui/src/app.rs` |
| Control adapters | Translate external input into `Message` | `app.rs`, `osc.rs`, `osc_rme.rs` |
| View | Build the iced element tree from state | `app.rs`, `widgets/` |

## 2. Device model and backends

`tuxmix-core` defines the model and the `RmeDevice` trait. The trait covers
matrix volumes and pans, mute/solo, preamps, EQ, dynamics, FX, clock, scenes,
event polling and meters.

`DeviceHandle` in `tuxmix-gui/src/app.rs` is the runtime union of the available
backends:

| Variant | Transport |
|---|---|
| `Fireface` | UC/UCX kernel hwdep transport |
| `UcxAlsa`, `Uc` | Legacy ALSA control surface |
| `BabyfacePro` | Babyface ALSA controls |
| `BabyfaceProUsb` | Proprietary USB backend |
| `Mirror` | UI-side copy owned by the device service |
| `Mock` | Simulation |

Every variant implements `RmeDevice`, so the application does not branch on the
model for ordinary control work. Capability queries decide which controls the
view exposes.

## 3. Device service and mirror

In the running application the device normally lives on the
`tuxmix-device-service` thread. The UI holds a `Mirror` instead of the real
handle.

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

`TUXMIX_DEVICE_SERVICE=0` keeps the device on the UI thread. Unit tests use the
direct path unless they attach the service explicitly.

Meters are not part of the model snapshot. The fader canvases read the shared
meter bus directly. The `MeterWake` message is only an edge that starts canvas
animation after silence; it does not rebuild the view for each meter packet.

## 4. Hardware to display

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

Backend-specific examples:

- **UCX hwdep**: `Fireface::poll_events()` drains `EV_STATUS`, `EV_STATE` and
  meter events. The UCX heartbeat keeps the state stream flowing. Front-panel
  gain, output volume, DIM and Mono updates reach `apply_ucx_state_word()` and
  the corresponding model fields.
- **Legacy UCX ALSA**: `poll_events()` refreshes output volume and preamp
  controls at a 100 ms interval.
- **Babyface USB**: `panel_tick()` reads the `0x17` panel status and runs the
  host-side panel state machine.
- **Meters**: the service copies device meter pages into `MeterSnapshot` at the
  polling rate. Peak hold lives there, so a transient shorter than a canvas
  frame is still visible.

The view reads the model through `Mirror` getters, which never lock or block on
the device thread.

## 5. Display to hardware

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

- **UCX hwdep**: `Fireface::set_volume()` computes the matrix coefficient or
  fader record and queues IF1 register words. Live writes and scene applies use
  the paced batch writer; the UI thread does not sleep through the pacing.
- **UC devices**: the same model operation becomes an EP0 vendor request.
- **ALSA backends**: the operation becomes an ALSA control write.
- **USB backend**: the operation becomes a proprietary vendor request.

All control adapters enter through `Message`, so they share the same clamps,
undo handling, host-scene merge and autosave. The update loop captures the
scene before an undoable change and merges the confirmed device state after the
command, either immediately on the direct path or after the next snapshot with
the service.

## 6. Control surfaces

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

`RmeFeedback` keeps a mirror of the last known values and the page state. That
is what allows it to resend the current page after `/setBankStart`,
`/setOffsetInBank` or `/setSubmix`, and to suppress sends outside the visible
bank or selected submix.

OSC uses the TotalMix FX dialect because its bank/page/submix model maps
directly onto the mixer's channel rows and send matrix. It is a translation
layer, not an internal representation.

## 7. Extending the control side

### Another OSC dialect

Add a module that:

1. parses its addresses into `Message`;
2. owns its own navigation state;
3. translates `OscOutbound` into its feedback paths;
4. runs as a subscription on its own port.

The mixer, undo, scene and device paths are unchanged.

### MIDI SysEx, MCU or another transport

The inbound side is already generic: any transport that produces `Message`
values can drive the mixer. A MIDI adapter needs:

- a MIDI transport worker;
- SysEx framing and 7-bit packing/unpacking;
- a decoder into `Message`;
- a feedback encoder.

The feedback side is the part that still needs generalising. `OscOutbound`
currently carries the internal event stream, but it is named for OSC and its
`Raw` variant contains internal address strings. For several control surfaces,
the natural refactor is:

- a control-surface-neutral `ControlEvent` enum;
- typed variants instead of internal path strings where practical;
- a list or broadcast of feedback sinks instead of the single `osc_tx`;
- one navigation/feedback state per surface.

### Another device backend

A new RME model or USB transport is a separate axis: implement `RmeDevice`,
add a `DeviceHandle` variant, and expose the capabilities the view should
enable. Control surfaces do not change.

## 8. Further reading

- [`DRIVER-ARCHITECTURE.md`](DRIVER-ARCHITECTURE.md) — USB, buffers and
  latency
- [`OSC-SETUP.md`](OSC-SETUP.md) — configuring TotalMix FX OSC
- [`spec/OSC-CONTROL-SPEC.md`](spec/OSC-CONTROL-SPEC.md) — OSC pages, scales
  and feedback table
- [`spec/TUXMIX-SPEC.md`](spec/TUXMIX-SPEC.md) — UI and persistence
