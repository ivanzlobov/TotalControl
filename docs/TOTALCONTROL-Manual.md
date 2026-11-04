# TotalControl Manual

TotalControl is a control surface for RME Fireface interfaces on Linux. It
replaces the vendor's mix application on Linux: routing, gains, EQ, dynamics,
FX and snapshots, driven through the kernel driver.

This manual is written for the end user. It describes installation, setup
and use. The developer specifications are in [`spec/`](spec/README.md); the
runtime architecture is in
[`TOTALCONTROL-ARCHITECTURE.md`](TOTALCONTROL-ARCHITECTURE.md).

Only the kernel driver plus hwdep path is supported for users. The legacy
ALSA backends, direct USB and capture/probe tools are development tools; see
[`DEVELOPMENT-TOOLS.md`](DEVELOPMENT-TOOLS.md).

Panels and controls appear only where the connected unit has them. Examples
in this manual assume a Fireface UCX, which has the full set: EQ, Dynamics,
FX and the Control Room. On a Fireface UC the same layout applies with the
reduced control set.

## Contents

- [Setup](#setup)
- [1. The mixer window](#1-the-mixer-window)
- [2. Views](#2-views)
- [3. Channel strips](#3-channel-strips)
- [4. How the mixer works: submixes](#4-how-the-mixer-works-submixes)
- [5. Working in stereo](#5-working-in-stereo)
- [6. The slideout panels](#6-the-slideout-panels)
- [7. Equalizer](#7-equalizer)
- [8. Dynamics](#8-dynamics)
- [9. FX: sends, reverb and echo](#9-fx-sends-reverb-and-echo)
- [10. Control Room](#10-control-room)
- [11. Gestures](#11-gestures)
- [12. Reset Mix](#12-reset-mix)
- [13. Snapshots, layouts, undo and redo](#13-snapshots-layouts-undo-and-redo)
- [14. Latency](#14-latency)
- [15. Where TotalControl keeps its data](#15-where-totalcontrol-keeps-its-data)
- [16. Running in the background](#16-running-in-the-background)
- [17. Device settings at startup](#17-device-settings-at-startup)
- [Appendix A — Device modes](#appendix-a--device-modes)
- [Appendix B — PipeWire](#appendix-b--pipewire)
- [Appendix C — OSC setup](#appendix-c--osc-setup)
- [Appendix D — Command-line options](#appendix-d--command-line-options)

## Setup

### Requirements

- A Linux system with ALSA and PipeWire.
- A supported Fireface in normal vendor mode.
- A kernel with matching headers and DKMS for the driver.

The supported devices are the Fireface UC and Fireface UCX. The Babyface Pro
FS backend is present but is not part of the supported installation path yet.

### Install the kernel driver

From the repository root:

```sh
./scripts/install-dkms.sh
```

The installer registers `snd-fireface-uc` with DKMS, builds it for the running
kernel and enables the load service. Keep the kernel headers installed so the
module is rebuilt after a kernel update.

Check the installation:

```sh
dkms status
modinfo -k "$(uname -r)" snd-fireface-uc
```

If the module is missing after a kernel update:

```sh
sudo dnf install kernel-devel-"$(uname -r)"
sudo dkms autoinstall -k "$(uname -r)"
sudo modprobe snd-fireface-uc
```

### Start TotalControl

Start the application from the desktop menu or from a terminal:

```sh
totalcontrol
```

The window opens on the Quick view. The Mixer and Matrix views are in the top
bar.

### Check the device

1. Open the device card in the sidebar.
2. Confirm the model, sample rate and buffer.
3. Confirm that the input meters move when a source is connected.

From a terminal, `aplay -l` lists the card and
`ls -l /dev/snd/hwC*D*` lists the hwdep nodes.

If no device appears, check that the unit is in normal vendor mode, that the
kernel module is loaded, and that another application is not holding the
device.

## 1. The mixer window

The mixer shows three rows of channel strips:

1. **Hardware Inputs** — the physical inputs of your interface.
2. **Software Playback** — the channels your applications play into.
3. **Hardware Outputs** — the physical outputs.

Every row carries an uppercase header. A **Control Room** section closes the
Hardware Outputs row; it holds your monitoring destinations (see
[section 10](#10-control-room)).

The right sidebar holds the device card (rate, buffer), Undo/Redo, global
mute and solo, the FX knob, group and snapshot controls and layout. It
stays visible as the window narrows and can be collapsed.

When the channels do not all fit, one slider at the lower right scrolls all
three rows together, so the routing picture stays aligned.

The device card in the sidebar reads `Internal, <sample rate>, <buffer>` and
opens the device sheet, which carries clock source, sample rate, the digital
input formats and firmware and serial information.

## 2. Views

TotalControl offers three views:

- **Quick**: one source into one destination — the fastest way to build a
  basic mix.
- **Mixer**: all strips on three rows, TotalMix-style.
- **Matrix**: the complete routing grid; double-click a cell to toggle it
  between off and unity.

These are also the tabs in the top bar, next to the `RESET MIX` button.

## 3. Channel strips

A channel strip shows, left to right:

- the **channel name** (click to select the channel, double-click to rename);
- a **submix label box** on inputs and playback channels; it selects the bus
  being written (see [section 4](#4-how-the-mixer-works-submixes));
- **mute** and **solo** (outputs use mute and **cue**);
- a **level fader** with a meter beside it;
- a column of small buttons: `T` (trim), the settings gear, `EQ` and `D`
  (dynamics), when the unit provides them;
- a **thin FX fader** at the top of that column, when the unit has an FX bus.

Every strip has the same height. A collapse button in the readout row under
the fader folds a strip so only its channel name and meter remain, and the
meter grows to fill the space.

The fader value is shown in a dB readout under the fader, without a unit
suffix. Double-clicking that readout opens a numeric entry field.

Output strips also carry a **balance** knob that pans the output pair
between its two channels; it appears only while the pair is linked.

Meters are peak meters with a held peak marker. They show the level as it
arrives at the interface. If the host (PipeWire) volume is below 0 dB, a
full-scale recording reaches the interface that much lower and the playback
meter reads correspondingly low. The device sheet shows the host volume and
has a `0 dB` button; set it to unity and use the interface's own faders for
monitoring. A linked output pair shows two meter bars, one per channel.

The meters are read from the interface stream independently of the rest of the
window. Moving a fader, opening a panel or scrolling does not interrupt or
freeze them. A short transient lights the peak marker even when it falls
between two screen frames. When every meter has been quiet for a moment they
stop redrawing, so an idle mixer stays still; the next signal wakes them
immediately and the held peak still catches a transient shorter than a frame.

## 4. How the mixer works: submixes

The mixer is a set of buses, one per hardware output. Selecting an output
strip makes it the *current submix*. After that, every input and playback
fader edits the **send of that channel into the selected output**, and the
output's own fader is the master level of that bus.

This is the same model as TotalMix FX:

- The fader of an input or playback strip never changes what the channel
  sends to your recording or other outputs; it changes how loud that
  channel plays into the currently selected output.
- To send a channel to another output, select that output's strip and set
  the fader again. Every output remembers its own mix independently.
- The meter on an input or playback strip shows the channel after its trim
  and before the sends; the meter on an output shows the finished bus.

## 5. Working in stereo

Input and playback pairs can be linked. When linked:

- the fader shows the louder channel of the pair and moving it shifts both
  channels together, preserving their level difference;
- the pair's balance is visible and adjustable from one knob;
- mute, solo, FX send, EQ, low cut and dynamics settings are shared from the
  left channel;
- panel buttons split into a **linked pair**: a phase pair shows `Ø L` and
  `R Ø`, for example.

Splitting a pair sets the two channels hard left and right for panning into
a submix and gives each channel its own fader. Re-linking restores the pair
level and balance you had before the split.

Output pairs behave the same way: the balance knob exists only while
linked, and splitting converts the pair level and balance into two
independent channel volumes. A pair assigned to the Control Room is always
stereo and never splits.

## 6. The slideout panels

Pressing a symbol button on a strip opens that channel's panel beside it.
The panel opens to the right of the strip and closes when you press the
same button again. There is no extra close button; the tabs never move.

### 6.1 Settings

The gear opens the Settings panel. Its rows follow the channel type, so
every channel of one type shows the same panel shape:

- Inputs 1/2: `stereo`, `48V`, `autoset`, `gain`, `width`, `send FX`,
  monitor processing (`MS proc`) and phase.
- Inputs 3/4: like above, with `inst` and the gain in the instrument/line
  range on the UCX, plus the level reference dropdown for these inputs.
- Inputs 5–8: `stereo`, the reference dropdown, `width`, `send FX`,
  `MS proc` and phase.
- Inputs from channel 9 on: `stereo`, `width`, `send FX`, `MS proc` and
  phase.
- Software playback: like the top inputs but without microphone features.
- Hardware outputs: `stereo`, the reference dropdown, `return FX`, phase,
  `no trim`, `talkback` receive and `loopback`.

Notes:

- **Gain** is the preamp gain of the four front inputs. It follows the
  stereo link and follows the unit's physical encoder when one is turned
  on the hardware.
- **Width** narrows or widens the pair: `1.00` full stereo, `0.00` mono,
  `-1.00` swapped channels.
- **MS proc** encodes or decodes the pair as mid/side.
- **Phase** is the `Ø` button; engaged phase shows green.
- The **send FX** knob and the strip's thin FX fader are the same control;
  the **return FX** knob on outputs is the return level of the FX bus.
- The **reference dropdown** sets the input or output reference level. For
  inputs 5–8 the four channels move together; inputs 3/4 can be linked.
- `loopback` makes this output's signal available on its input channel, so
  recording software can capture what the output plays. `no trim` and
  `talkback` receive are host-side settings.

On/off controls sit at the left of their row. Engaged switches light orange
(`48V`, `stereo`, `autoset`, `inst`, and others) or green (phase).

### 6.2 EQ and Dynamics

The `EQ` and `D` buttons open the equalizer and the dynamics panel. See
[sections 7](#7-equalizer) and [8](#8-dynamics).

### 6.3 Trim

`T` on an input or playback strip is the pre-fader trim of the channel: it
attenuates the channel before all of its sends, whatever output it plays
into. The tab lights while trim is active, and its tooltip shows the value.

## 7. Equalizer

EQ is available on every hardware input and hardware output. The panel shows
the response plot with three coloured segments — red low, green mid, blue
high — built from the three band frequencies.

Three bands, each with gain, frequency and width:

| Parameter | Range |
|---|---|
| Gain | ±20 dB |
| Frequency | 20 Hz – 20 kHz |
| Width (Q) | 0.7–5.0 |

- Band 1 and band 3 can be a bell or a shelf; band 2 is always a bell; band
  3 can instead be a low pass (hi cut).
- The gain knob of each band carries the colour of its segment.
- Every band keeps its frequency even when the band is off, so the plot
  never jumps.

The low-cut row holds the low-cut on/off button, its frequency knob and the
slope buttons (6/12/18/24 dB per octave). Low cut is independent of the main
EQ; you can edit it while it is off, and it is only heard when enabled.

Double-clicking an EQ handle on the plot resets that handle.

## 8. Dynamics

`D` opens the dynamics panel, on hardware inputs and hardware outputs. The
graph is an input-to-output level curve: green where the expander works,
blue in the neutral range, red where the compressor limit applies.

The panel has:

- an on/off switch at the top;
- shared **gain**, **attack** (0–200 ms) and **release** (100–999 ms) used
  by both compressor and expander;
- a **compressor** column: threshold −60…0 dB, ratio 1:1…10:1 (red);
- an **expander** column: threshold −99…−30 dB, ratio 1:1…10:1 (green);
- an **auto level** row: on/off, maximum gain 0–18 dB, headroom 3–12 dB and
  rise time 0.1–9.9 s.

A change to any knob updates the whole dynamics block of that channel.

## 9. FX: sends, reverb and echo

The `FX` knob in the right sidebar opens the FX column, when the unit has an
FX bus. It lights while the column is out. Every input and playback strip can
send to it:

- the thin fader at the top of the strip (or `send FX` in the Settings
  panel) is the send level; its rail fills orange when raised;
- an output strip's thin fader (or `return FX`) is how loud the FX return
  plays on that output; its rail fills green when raised.

The column holds **Reverb** and **Echo**, each with an on/off switch and a
type menu. Both carry the controls TotalMix FX provides: predelay, room
scale, cutoffs, width and volume for reverb; delay time with a BPM readout,
feedback, high cut and width for echo. The delay and BPM readouts are two
views of one value. The echo block also shows the FX bus level meters.

## 10. Control Room

The Control Room section sits at the right of the Hardware Outputs row. It
holds your monitoring destinations — `Main`, `Speaker B` and `Phones 1–4`.
Every hardware output is either assigned in the Control Room or unassigned
in the Hardware Outputs row; there are no empty slots.

### Who hears what

The Control Room is not another mixer bus. It gives names and monitoring
behaviour to hardware outputs you already have:

- **Main** is normally the engineer’s monitor output: the speakers in the
  control room.
- **Phones 1–4** are normally the performers’ headphone outputs. They are
  not the engineer’s headphones unless you assign one of them that way.
- **Speaker B** is an alternative monitor output, for a second pair of
  speakers or a second listening position.
- **Cue/PFL to** chooses where the engineer’s cue or PFL signal is heard.

So the usual studio picture is: the engineer listens on Main, the performers
listen on Phones, and the Control Room decides which physical outputs play
those roles. Assigning a pair moves it out of the ordinary Hardware Outputs
row and gives it the fixed Control Room name.

### Talkback and Listenback

Talkback is the engineer’s communication path to the performers. While
Talkback is active, the selected talkback microphone is sent to the Phones
outputs that receive it, and the other signals on those Phones outputs are
attenuated so the performer can hear the engineer clearly without removing
the cue mix permanently.

Listenback is the reverse direction: a microphone in the recording room is
sent to Main while Talkback is active, with its own attenuation for the Main
signal. RME’s Control Room uses both directions; TotalControl currently exposes the
Talkback button and its receive switches.

Talkback attenuates the active sends in the monitored mix; silent routes are
not rewritten. The talkback source is added only to outputs whose Talkback
receive is enabled. Talkback also silences the FX returns on the monitored
outputs; it does not switch the Reverb or Echo processors off. Releasing
Talkback restores the previous sends and FX-return levels.

These are temporary monitoring states. Releasing Talkback returns the
monitor mix to its previous state; talkback is not stored in a scene.

The button stack, on one channel, has:

- **dim** and **recall** (snapshot recall) and **speaker B**;
- **mono**, **talkback**, **ext. input** and **mute FX**;
- **settings** and **assign** at the bottom.

| Button | Effect |
|---|---|
| `dim` | lowers the monitor level according to the Control Room Dim setting; releasing it restores the level |
| `recall` | sets the monitor level to the configured listening volume; momentary, not a toggle |
| `speaker B` | switches monitoring from Main to the Speaker B output |
| `mono` | sums the Main monitor signal to mono |
| `talkback` | sends the talkback source to the performers’ Phones outputs and dims the other Phones signals |
| `ext. input` | replaces the Main mix with the selected stereo external input |
| `mute FX` | removes Reverb and Echo from Main |
| `settings` | opens the Control Room Settings dialog |
| `assign` | assigns Main, Speaker B and Phones outputs and selects the Cue/PFL destination |

Talkback and External Input are temporary listening states. Recall changes the
underlying monitor level; it is not a second volume control that is restored
on release.

### Control Room Settings

The Talkback source and Dim, Listenback source and Dim, Recall level, Main Dim,
External Input and Gain, and the Solo/PFL modes are set in the Control Room
Settings dialog. Press `settings`, immediately above `assign`.

The dialog has four sections:

- **Talkback**: source and Dim.
- **Listenback**: source and Dim.
- **Main Out**: Recall, Dim, External Input and Gain.
- **Solo / PFL Mode**: `Live Mode, PFL replaces Solo` and
  `Exclusive Solo/PFL Mode`.

The source lists start with `None` and then use the hardware input names. The
five level controls cover `-65.0` to `0.0 dB` in `0.5 dB` steps. The settings
are saved per device in TotalControl's own preferences, not in the mixer scene.

`OK` applies and saves the values; `Cancel` and `Escape` discard the edits.
The settings in the dialog are not written while it is open.

`assign` opens the destination menu. The first level lists the destinations
— the same names as the Control Room channels — plus `Cue/PFL to`, which
chooses which output carries your cue/PFL signal and never
reassigns hardware. The second level lists the hardware channels as their
predetermined names (`None`, then every hardware output, current choice in
bold). Choosing `None` clears a destination.

Control Room channels are always stereo: an assigned pair is linked and
cannot be unlinked, so the `STEREO` switch in its Settings panel stays
engaged and does nothing. A split pair is linked when it is assigned.

Control Room channel names are fixed: they are always `Main`, `Speaker B`
and `Phones 1`–`Phones 4`, regardless of the View → Names setting, and they
cannot be renamed. The hardware channel feeding each destination is listed
by its own hardware name in the second menu level.

Clicking a Control Room strip — including its name, fader or switches —
selects it as the current submix, like any other output. Control Room
channels are ordinary outputs for routing, so a hardware output stays a
submix even while it is your monitoring destination.

The Control Room buttons (dim, speaker B, mono, and so on) behave like the
front-panel equivalent controls of the unit.

## 11. Gestures

- **Drag** a fader, knob, matrix cell or EQ handle to change it. The drag
  keeps working even when the pointer leaves the control or the window,
  until you release it.
- **Hold Shift** while dragging for fine adjustment. This works on faders,
  knobs, matrix cells and EQ handles.
- **Scroll** over a fader or knob nudges it.
- **Double-click** a fader, the FX fader, a knob or an EQ handle to reset
  that control; double-click a matrix cell to toggle it between off and
  unity; double-click a channel name to rename it; double-click the dB
  readout to type a value; double-click a strip's background to collapse
  it.
- Ordinary buttons act once per click.

## 12. Reset Mix

The `RESET MIX` button in the top bar lists the standard mixes:

| Entry | Effect |
|---|---|
| Straight playback with all to Main Out | playbacks 1:1 to the outputs, each also mixed to Main |
| Straight playback | playbacks 1:1 to the outputs, nothing else routed |
| Clear all submixes | every input and playback send set to nothing |
| Clear channel effects | EQ, low cut, dynamics, auto level, reverb, echo and width off |
| Set output volumes | outputs 0 dB; Main and Speaker B −10 dB |
| Reset channel names | user names cleared |
| Set all channels mono/stereo | the stereo or mono mode of every pair |
| Set inputs mono, outputs stereo | the ADM layout |
| Total reset | straight playback to Main, effects off, output volumes set |

Choosing one rewrites the whole mixer in one go.

## 13. Snapshots, layouts, undo and redo

TotalControl keeps your work in three layers:

- **Snapshots** (eight slots) store a complete mix: routing, faders, names,
  trims and the device settings TotalControl manages. They are mixer-plus files —
  broader than the device's own snapshot. Recall applies them to the device
  at once. The current mix is saved every few seconds and again when
  TotalControl closes.
- **Layouts** (six slots) store only which strips are collapsed.
- **GUI state** (window, sidebar, view, zoom, names, open panels, selected
  output and link groupings) is remembered automatically between sessions.

Undo restores the scene as it was before your last mix-changing step; every
continuous drag of one control counts as a single step. Redo reapplies what
you undid. Device settings and window layout are not part of undo.

Groups (multi-select sets) exist only while TotalControl runs; they are not saved.
There is no RME-style Workspace file — TotalControl keeps the same information in
separate files instead.

## 14. Latency

The sidebar shows the round-trip latency estimate for the current sample
rate and buffer size, and the **Quantum** picker sets the PipeWire graph
period in samples. The driver and the unit add a fixed amount of
device-side latency on top of the buffer; the readout lists the estimate
and its parts (USB queue, device lead, converters and graph period) so you
can budget monitoring and recording latency.

The **Errors** section lists driver transport faults since the device sheet
opened or the last **reset**, one row per type. `usb`, `gaps` and `short`
show input and output as `in / out`; `urb`, `recover` and `events` are single
values. A non-zero row means the USB transport is dropping or mis-timing
packets; reset only clears the display baseline.

## 15. Where TotalControl keeps its data

- Snapshots: `Mix <n>.json` in the scene directory.
- The continuously saved current mix: `auto.json` in the scene directory.
- Layouts: `Layout <n>.json` in the layout directory.
- Window and GUI state: `gui.json` in the `~/.config/totalcontrol/` directory.

These are plain JSON files you can back up or copy between machines.

## 16. Running in the background

With a system tray, closing the window keeps TotalControl running, as TotalMix
does: the front-panel DIM button and the restore of your mix after a replug
are TotalControl's job while it runs. The tray icon toggles the window; its menu
has dim, recall mix 1–8, settings and quit. `totalcontrol --minimized` starts
hidden, and Settings can start TotalControl with your session. Without a tray,
closing the window quits; one TotalControl runs per session, and a second launch
tops up the first.

See [`ui/totalcontrol/docs/guide.md`](../ui/totalcontrol/docs/guide.md) for the Linux
audio stack, building from source and project status.

## 17. Device settings at startup

At connect TotalControl compares the readable device state with the saved scene
and lets you choose, with a dialog:

- **Use TotalControl settings** — write the saved scene to the device.
- **Use device settings** — keep what the device currently reports where
  TotalControl can read it (clock, gains, volumes, and similar).
- **Always use TotalControl settings** — as above and stop asking.

Note that on the UCX the hardware-held switches on the front panel (options,
reference levels, `48V` and similar) are not read back by TotalControl, so the
panel shows the last value set from TotalControl; the dialog never changes them.

## Appendix A — Device modes

TotalControl needs the normal vendor mode of the interface. In that mode the
kernel driver exposes the PCM device and the hwdep mixer node.

In class-compliant mode the stock `snd-usb-audio` driver carries audio, but the
vendor mixer is not available. TotalControl cannot control the mixer in that
mode. Select the normal mode on the unit before starting TotalControl.

Only one application may own the mixer at a time. Close other mixer
applications before starting TotalControl.

## Appendix B — PipeWire

TotalControl uses the ALSA PCM exposed by the kernel driver. PipeWire sees it
as a normal card. Use the Pro Audio profile when the device should expose all
channels to a DAW.

The buffer-size picker in TotalControl sets the graph quantum. The driver adds
its own USB queue and device lead; the latency display explains the result.

The WirePlumber rule in
[`ui/totalcontrol/tools/alsa/wireplumber-uc.conf`](../ui/totalcontrol/tools/alsa/wireplumber-uc.conf)
selects polled scheduling and the 18-channel Pro Audio profile.

## Appendix C — OSC setup

OSC is opt-in and binds to loopback by default:

```sh
totalcontrol --osc
```

| Option | Default | Meaning |
|---|---:|---|
| `--osc-port` | 9000 | UDP port TotalControl listens on |
| `--osc-send-port` | 9001 | UDP port for feedback |
| `--osc-host` | 127.0.0.1 | Feedback destination |

Configure the controller to send to the `--osc-port` and receive feedback on
`--osc-send-port`. A TotalMix FX-compatible controller can use its normal
address space. The address space is defined in
[`spec/TOTALCONTROL-OSC-SPEC.md`](spec/TOTALCONTROL-OSC-SPEC.md).

For a controller on another host, set `--osc-host` to the controller's address
and allow the feedback port through the firewall. Do not run two controllers
on the same feedback port.

## Appendix D — Command-line options

| Option | Effect |
|---|---|
| `--osc` | Enable the OSC control surface |
| `--view quick|mixer|matrix` | Start on a view |
| `--fullscreen` | Start fullscreen |
| `--minimized` | Start hidden in the tray |
| `--scale <n>` | Initial UI scale |
| `--select-out <n>` | Initial submix |
| `--mock` | Simulate a Babyface Pro FS |
| `--mock-ucx` | Simulate a Fireface UCX |
| `--mock-uc` | Simulate a Fireface UC |
| `--control-only` | Do not open the capture PCM |

The mock modes are for demonstration and development. Device backend switches
and capture tools are documented in `DEVELOPMENT-TOOLS.md` and are not
supported installation options.
