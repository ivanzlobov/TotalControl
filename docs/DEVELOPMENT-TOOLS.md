# Development tools

This note describes the tools that are not part of the supported user
installation. The supported path is the kernel driver plus hwdep; everything
else here is for development, reverse engineering, comparison and hardware
testing.

## Contents

- [Support status](#support-status)
- [Direct USB backend](#direct-usb-backend)
- [Legacy ALSA backends](#legacy-alsa-backends)
- [ALSA PCM plugin](#alsa-pcm-plugin)
- [Mock modes](#mock-modes)
- [Environment variables](#environment-variables)
- [Capture and probe tools](#capture-and-probe-tools)
- [Build and test helpers](#build-and-test-helpers)
- [Licensing](#licensing)

## Support status

| Path | Status |
|---|---|
| Kernel driver + hwdep | supported user path |
| `totalcontrol-usb` direct libusb | development only |
| `BabyfacePro` ALSA | development fallback |
| ALSA PCM plugin | development only |
| Probe and capture programs | development only |
| Mock backends | development and tests |

The manual does not describe these as installation options. They are kept
because they are useful for protocol work, capture sessions and comparisons.

## Direct USB backend

`totalcontrol-usb` speaks the vendor protocol over libusb. It bypasses ALSA and
the kernel driver and claims the USB interface directly. It cannot coexist
with the kernel driver or a running PipeWire graph on the same device.

Use it for:

- protocol probes and captures;
- the Babyface proprietary-mode work;
- comparing a direct transport with the kernel path.

Start it explicitly:

```sh
totalcontrol --backend usb
```

Do not use this mode for normal monitoring. The kernel driver path is the
supported one.

## Babyface ALSA backend

`BabyfacePro` reads and writes the ALSA mixer controls exposed by the Babyface
kernel driver. It is used when the direct hwdep path is not available for that
unit.

It is selected automatically when its card is present, or explicitly:

```sh
totalcontrol --backend alsa
```

## ALSA PCM plugin

`ui/totalcontrol/tools/alsa/` contains the Babyface Pro FS proprietary-mode PCM
bridge:

```text
ALSA application
  -> libasound_module_pcm_totalcontrol.so
  -> libtotalcontrol_sys.so
  -> totalcontrol-usb
```

It exposes playback and capture channels to ALSA and PipeWire while
`totalcontrol-usb` owns the device. It is part of the direct-USB development
path, not the kernel-driver installation.

Build and test:

```sh
cd ui/totalcontrol
cargo build -p totalcontrol-sys --release
sudo make -C tools/alsa install
```

The plugin and its PipeWire drop-in are documented in
`ui/totalcontrol/tools/alsa/README.md`.

## Mock modes

The mock backends run the complete UI without hardware:

```sh
totalcontrol --mock       # Babyface Pro FS
totalcontrol --mock-ucx   # Fireface UCX
totalcontrol --mock-uc    # Fireface UC
```

The mocks are used by the unit tests and by headless UI checks.

## Environment variables

All development switches use the `TOTALCONTROL_` prefix:

| Variable | Effect |
|---|---|
| `TOTALCONTROL_DEVICE_SERVICE=0` | Keep the device on the UI thread |
| `TOTALCONTROL_MOCK_MODEL=ucx\|uc` | Select the mock model |
| `TOTALCONTROL_MOCK_CC=1` | Simulate class-compliant mode |
| `TOTALCONTROL_MOCK_DISCONNECT=1` | Simulate an unplugged unit |
| `TOTALCONTROL_NO_CAPTURE=1` | Do not open the capture PCM |
| `TOTALCONTROL_DEFER_METER_TAP=1` | Defer the input meter tap |
| `TOTALCONTROL_METER_DEBUG=1` | Print meter values once per second |
| `TOTALCONTROL_CAPTURE_DUMP=<path>` | Dump captured samples |
| `TOTALCONTROL_MULTIPLE_INSTANCES=1` | Bypass the single-instance guard |
| `TOTALCONTROL_PROBE_STREAM=1` | Enable the stream probe |
| `TOTALCONTROL_PROBE_PLAYER=1` | Enable the probe player |
| `TOTALCONTROL_PROBE_SCENE_ONLY=1` | Apply the probe scene only |

## Capture and probe tools

| Tool | Purpose |
|---|---|
| `scripts/capture-usbmon.sh` | Capture USB traffic |
| `tools/parse_capture.py` | Parse a captured stream |
| `tools/decode-totalmix-capture.py` | Decode TotalMix FX captures |
| `tools/rme-probe.c` | Direct vendor-request probe |
| `scripts/ffuc-counters.c` | Read driver transport counters |
| `tools/uc-loopback-phase-probe.sh` | UC loopback phase probe |
| `tools/uc-rate-probe.sh` | UC rate probe |
| `scripts/uc-latency-stress.py` | UC latency stress run |
| `scripts/ffuc-48k-gate.sh` | 48 kHz transport gate |
| `scripts/mixer-demo.sh` | Legacy mixer demonstration |
| `ucx-state-watch` example | Print changed UCX state words |
| `ucx-dump-diff` example | Request the full dump, change one control, print the changed words |
| `ucx-dump-compare` example | Request the full dump and compare it with a saved scene |

Protocol captures and measurements are evidence, not specifications. Record
the device model, transport, firmware and rate with every capture.

## Build and test helpers

From `ui/totalcontrol`:

```sh
cargo test -p totalcontrol-core --lib --features alsa
cargo test -p totalcontrol-gui
cargo test -p totalcontrol-layout
cargo build --workspace
```

## Channel layout preview

`totalcontrol-layout` owns the shared channel template schema and produces
the preview used to review channel layout changes without launching the GUI:

```sh
cd ui/totalcontrol
cargo run -p totalcontrol-layout --bin channel-preview -- /tmp/channel-preview.svg 2
```

The second argument is the zoom.  Open the SVG directly or convert it for a
quick raster review:

```sh
magick /tmp/channel-preview.svg /tmp/channel-preview.png
```

Render a candidate configuration without rebuilding:

```sh
cargo run -p totalcontrol-layout --bin channel-preview -- \
    --config ~/.config/totalcontrol/gui-layout.json \
    /tmp/my-layout.svg 2
```

Write the current built-in defaults as a starting example:

```sh
cargo run -p totalcontrol-layout --bin channel-preview -- \
    --example /tmp/gui-layout.json
```

The command writes `/tmp/gui-layout.json` and the matching
`gui-layout.schema.json` beside it; the document references that sibling file
through `$schema` for editor validation and completion.  The checked-in files
are [`examples/gui-layout.json`](examples/gui-layout.json) and
[`schema/gui-layout.schema.json`](schema/gui-layout.schema.json).  See
[`spec/TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md`](spec/TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md)
for the field reference, the validation rules and the change workflow.

`scripts/audio-health.sh` and `scripts/reload-kernel.sh` are transport and
driver helpers. `scripts/install-dkms.sh` is the supported driver installer;
`scripts/install-kernel.sh` is its compatibility wrapper.

`scripts/run-totalcontrol.sh` builds the current GUI sources, stops a running
debug instance and starts the freshly built binary. A local development desktop
entry can point at it so a menu launch never starts a stale build.

## Licensing

The application license split is described in `ui/totalcontrol/LICENSE`:

- `totalcontrol-core`: MIT
- `totalcontrol-gui`, `totalcontrol-layout` and `totalcontrol-tui`: GPL-3.0-only
- `totalcontrol-usb`: GPL-3.0-or-later
- `totalcontrol-sys`: GPL-3.0

The kernel module is GPL-2.0-only. Tools and scripts follow the repository
license unless their own notice says otherwise.
