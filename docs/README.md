# Documentation

TotalControl has one architecture document, a GUI specification, one protocol
specification per device, an OSC specification and a user manual.

## Core documents

| Document | Contents |
|---|---|
| [`TOTALCONTROL-ARCHITECTURE.md`](TOTALCONTROL-ARCHITECTURE.md) | runtime layers, device service, data flow, state ownership, driver and latency model |
| [`spec/TOTALCONTROL-GUI-SPEC.md`](spec/TOTALCONTROL-GUI-SPEC.md) | layout, controls, panels, gestures and motion |
| [`spec/TOTALCONTROL-OSC-SPEC.md`](spec/TOTALCONTROL-OSC-SPEC.md) | TotalMix FX OSC transport, pages, scales and feedback |
| [`TOTALCONTROL-Manual.md`](TOTALCONTROL-Manual.md) | installation, setup, use and troubleshooting |
| [`DEVELOPMENT-TOOLS.md`](DEVELOPMENT-TOOLS.md) | development backends, capture and probe tools |

## Device protocol specifications

| Document | Device |
|---|---|
| [`spec/FIREFFACE-UCX-PROTOCOL-SPEC.md`](spec/FIREFFACE-UCX-PROTOCOL-SPEC.md) | Fireface UCX |
| [`spec/FIREFFACE-UC-PROTOCOL-SPEC.md`](spec/FIREFFACE-UC-PROTOCOL-SPEC.md) | Fireface UC |
| [`spec/BABYFACE-PRO-PROTOCOL-SPEC.md`](spec/BABYFACE-PRO-PROTOCOL-SPEC.md) | Babyface Pro FS |

Open questions, capture requests and working notes live in [`work/`](work/).
Raw captures live in [`captures/`](captures/).

The specifications are present-tense and describe the current implementation.

RME device manuals are proprietary and are not part of the source
distribution.
