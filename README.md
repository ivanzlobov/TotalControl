# TotalControl and the Fireface driver for Linux

Native Linux support for the RME Fireface UC and UCX in normal RME vendor
mode: a kernel transport module (`snd-fireface-uc`) and the TotalControl mixer.

This tree is based on [TuxMix](https://github.com/ismail-bahloul/TuxMix) by
Ismaïl Bahloul — his Babyface Pro FS mixer — and extends it into a full
control surface for the Fireface UC family. His commit history is preserved on
the `archive/tuxmix-upstream` branch.

![The TotalControl mixer and Control Room driving a Fireface UCX, with the EQ and dynamics panels, sends, and the reverb and echo panels open](ui/totalcontrol/docs/screenshots/mixer-ucx.png)

## Functionality

- **Fireface UC and Fireface UCX support.**  The `snd-fireface-uc` kernel
  module provides the vendor-mode USB transport, the ALSA PCM device, a
  `snd-hwdep` control ABI and per-unit identification by serial.
- **Seamless sample-rate switching.**  The rate follows the session without
  a reconnect, and the transport stays stable at the high rates — 192 kHz
  included — down to buffer sizes of 96 samples.
- **MIDI in and out over USB.**  A standard rawmidi port on the unit's
  second USB interface; works with any Linux sequencer or DAW.
- **Full OSC support.**  The complete RME OSC dialect: per-channel
  parameters, streamed meter peaks and remote control of the whole mixer
  and control room from any OSC client.
- **A much improved GUI.**  Near-complete mixer and Control Room
  functionality, with channel strips that slide their panels in and out —
  EQ, dynamics, sends and settings live next to their own strip.

## Architecture

The kernel module is a transport core:

- USB enumeration and per-device descriptors;
- alternate-setting and sample-rate selection;
- interrupt and isochronous URB engines;
- the ALSA PCM device;
- one `snd-hwdep` control node per card.

TotalControl owns the device model: matrix routing, faders, preamps, EQ/dynamics,
meters, clock/options and scenes.  The model and connection are deliberately
separate: a backend reports its capabilities, and the GUI only shows controls
that connection can actually drive.

- `kernel/` — the kernel transport module.
- `ui/totalcontrol/` — TotalControl core, GUI and TUI.
- `docs/spec/` — authoritative, present-tense specifications.

## Devices

| Device | USB ID | Audio transport |
|---|---|---|
| Fireface UC PC | `0424:3fc5` | interrupt IF0 |
| Fireface UC AP | `0424:3fc6`, `2a39:3fc6` | isochronous IF0 |
| Fireface UCX | `0424:3fc9`, `2a39:3fc9` | isochronous IF0 (default) or interrupt IF5 |

The UCX uses the isochronous interface by default (`ucx_if5=0`); set
`ucx_if5=1` for the interrupt interface.  The UC uses the personality selected
on the unit (PC or AP).

Normal RME vendor mode uses RME's proprietary protocol over USB.

One module instance handles several units.  TotalControl identifies each unit by
its serial number, remembers the selected unit across restarts, and opens that
exact unit again after a reconnect.  Scene files are kept per unit, including
two units of the same model.

## Quick start

```sh
./scripts/build-kernel.sh
./scripts/reload-kernel.sh

cd ui/totalcontrol
cargo run -p totalcontrol-core --example fireface-scan --features alsa
cargo run -p totalcontrol-gui
```

`aplay -l` lists the cards and `ls -l /dev/snd/hwC*D*` lists the control nodes.
The kernel exposes no ALSA mixer controls; TotalControl uses the hwdep node.

The WirePlumber rule in `ui/totalcontrol/tools/alsa/wireplumber-uc.conf` selects the
18-channel Pro Audio profile and keeps ALSA scheduling polled.  Install it as
`~/.config/wireplumber/wireplumber.conf.d/60-fireface-uc.conf` and restart
WirePlumber.

## Kernel updates

Install the module through DKMS once:

```sh
./scripts/install-dkms.sh
```

The installer registers the source in `/usr/src/snd-fireface-uc-0.1`, builds
it for the running kernel, and installs and enables
`snd-fireface-uc-load.service`.  On Fedora, the kernel package's
`40-dkms.install` hook rebuilds the module for each new kernel.  No dnf plugin
or path watcher is needed.  Keep `kernel-devel` installed; the boot unit loads
the module after DKMS at the next boot.

Check the result with:

```sh
dkms status
modinfo -k "$(uname -r)" snd-fireface-uc
```

If a new kernel was installed without matching headers:

```sh
sudo dnf install kernel-devel-"$(uname -r)"
sudo dkms autoinstall -k "$(uname -r)"
sudo modprobe snd-fireface-uc
```

`scripts/install-kernel.sh` is a compatibility wrapper for the DKMS installer.

Before a sample-rate, channel-count or transport change, run the 48 kHz gate:

```sh
cd ui/totalcontrol
cargo build -p totalcontrol-core --example fireface-ctl --features alsa
cd ../..
scripts/ffuc-48k-gate.sh all
```

## Documentation

- [Architecture](docs/TOTALCONTROL-ARCHITECTURE.md) — runtime layers, device
  service, control surfaces, driver and latency model.
- [GUI specification](docs/spec/TOTALCONTROL-GUI-SPEC.md) — layout, controls,
  panels, gestures and motion.
- [UC protocol specification](docs/spec/FIREFFACE-UC-PROTOCOL-SPEC.md).
- [UCX protocol specification](docs/spec/FIREFFACE-UCX-PROTOCOL-SPEC.md).
- [Babyface Pro protocol specification](docs/spec/BABYFACE-PRO-PROTOCOL-SPEC.md).
- [OSC specification](docs/spec/TOTALCONTROL-OSC-SPEC.md).
- [User manual](docs/TOTALCONTROL-Manual.md).
- [Development tools](docs/DEVELOPMENT-TOOLS.md) — non-user-facing backends,
  capture and probe helpers.
- [Documentation index](docs/README.md).

## Current scope

- UC PC and AP transports.
- UCX isochronous and interrupt transports.
- Matrix, masters, mutes, pan, loopback, preamps, EQ/dynamics, clock and
  meters.
- Several cards, including identical models, selected by serial.
- Babyface Pro FS through the inherited ALSA and direct-USB backends.
- TotalControl GUI and TUI over the hwdep backend.

The UCX backend hides capabilities whose register or write path is not
implemented: per-cell mono matrix editing, output balance, phantom power, PAD,
pitch, width, MS processing and AN 1>2.  These are absent by design rather
than controls that silently do nothing.

## License

The repository contains separate works under different licenses:

- `kernel/` — the Fireface UC/UCX kernel module, **GPL-2.0-only**, matching the
  kernel and the GPL-2.0 sources it builds on.
- Tools, scripts and documentation — **GPL-3.0-or-later**.  See
  [LICENSE](LICENSE).
- `ui/totalcontrol/totalcontrol-core` — **MIT**.
- `ui/totalcontrol/totalcontrol-gui` and `ui/totalcontrol/totalcontrol-tui` — **GPL-3.0-only**.
- `ui/totalcontrol/totalcontrol-usb` — **GPL-3.0-or-later**;
  `ui/totalcontrol/totalcontrol-sys` — **GPL-3.0**.

The UI notices are in [ui/totalcontrol/LICENSE](ui/totalcontrol/LICENSE) and
[ui/totalcontrol/COPYING](ui/totalcontrol/COPYING).

The module and the mixer meet at the kernel's hwdep ioctl interface; they are
not combined into a single work.

## Credits and attribution

This project is built on prior work, and we are grateful to its authors.

- [TuxMix](https://github.com/ismail-bahloul/TuxMix) by Ismaïl Bahloul
  (MIT core, GPL-3.0-only GUI/TUI) is the base of the mixer application: its
  device model, the Babyface Pro FS support, the TUI and the scene system. His
  commit history is archived on the `archive/tuxmix-upstream` branch, and the
  license and copyright notice stay in [`ui/totalcontrol/LICENSE`](ui/totalcontrol/LICENSE)
  and [`ui/totalcontrol/COPYING`](ui/totalcontrol/COPYING).
- [snd-fireface-uc](https://github.com/berndie/snd-fireface-uc) by Bernd Accou
  (MIT) is the earlier Fireface UC kernel driver; our UC transport follows its
  device modes, packet layout and status path.
- [rme-fireface-ucx-linux-driver](https://github.com/OroboroModular/rme-fireface-ucx-linux-driver)
  (GPL-2.0-or-later) is the Fireface UCX driver; our UCX transport, register
  model and meter decoding build on it.

The Fireface UC/UCX kernel module, the OSC layer and the GUI's mixer and
Control Room extensions were developed in this tree.  All Fireface protocol
knowledge comes from our own USB captures; no RME documentation or code is
included.  Full notices are in [`THIRD-PARTY.md`](THIRD-PARTY.md).

