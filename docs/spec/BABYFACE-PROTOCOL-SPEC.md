# Babyface (first generation) protocol

The first-generation Babyface in its native (vendor) USB mode: transport,
channel model, mixer writes and metering, as implemented by
`snd-fireface-uc` and the `BabyfaceGen1` backend. The Babyface Pro FS is a
different device; see [BABYFACE-PRO-PROTOCOL-SPEC.md](BABYFACE-PRO-PROTOCOL-SPEC.md).

## Contents

- [1. Transport](#1-transport)
- [2. Channel model](#2-channel-model)
- [3. Mixer](#3-mixer)
- [4. Preamps and settings](#4-preamps-and-settings)
- [5. Metering](#5-metering)
- [6. State ownership](#6-state-ownership)

## 1. Transport

| Interface | Endpoints | Type | Use |
|---|---|---|---|
| 0 | `0x03` OUT, `0x84` IN | isochronous | audio, UCX packet format |
| 1 | `0x85` IN | bulk | DSP meter packets (§5) |
| 1 | `0x0a` OUT | bulk | DSP EQ blocks; not used |
| 2 | `0x07` OUT, `0x86` IN | bulk | USB-MIDI |

Audio words per frame:

| Rate | Capture | Playback |
|---|---|---|
| 1× (32–48 kHz) | 12 | 14 |
| 2× (64–96 kHz) | 8 | 10 |
| 4× (128–192 kHz) | 6 | 8 |

Sample rate and stream start/stop use the UCX EP0 requests (`0x1b`,
`0x10`). The playback mute requests `0x13` (mute) and `0x14` (unmute) take
an 18-bit channel mask packed like a `0x12` level: channel *n* is bit *n*,
`wValue = mask >> 2`, `wIndex = (mask & 3) << 14`. The driver sends all
18 bits (`wValue 0xffff`, `wIndex 0xc000`) at stream stop and start. The
driver reports the model as `FFUC_MODEL_BABYFACE` in the hwdep INFO block.

All mixer and settings writes are EP0 vendor OUT requests
(`bmRequestType 0x40`, no data stage) sent through the hwdep vendor ioctl.

## 2. Channel model

| Index | Inputs | Outputs |
|---|---|---|
| 0, 1 | AN1, AN2 (mic/line) | AN1, AN2 (XLR) |
| 2, 3 | AS1, AS2 (optical ADAT 1/2 or S/PDIF) | PH3, PH4 (phones) |
| 4, 5 | ADAT3, ADAT4 | AS1, AS2 |
| 6..9 | ADAT5..ADAT8 | ADAT3..ADAT6 |
| 10, 11 | — | ADAT7, ADAT8 |
| 12, 13 | — | FX send bus L/R |

Playback channels 1..12 feed the matrix. Playback 13/14 are the return
of the host-side reverb; they do not enter the DSP, and the crosspoints
`src` 22/23 have no effect. Capture channels 0..9 carry the inputs;
capture 10/11 carry the FX send bus.

The profile groups the 10 inputs and 12 outputs into the stereo pairs
AN1/2, PH3/4, AS1/2, ADAT3/4, ADAT5/6, ADAT7/8.

## 3. Mixer

Request `0x12` writes one matrix crosspoint or one output master.

| Target | Address |
|---|---|
| crosspoint `src` → output `out` | `24 · out + src` |
| output master `out` | `0x3e0 + out` |

| `src` | Source |
|---|---|
| 0..9 | hardware inputs (§2) |
| 10..21 | playback 1..12 |

The level is an 18-bit linear gain, `level = gain · 0x8000` (`0x8000` =
0 dB, `0x10000` = +6 dB, 0 = off):

```text
wValue = level >> 2
wIndex = ((level & 3) << 14) | address
```

The unit powers up with no playback routed. The backend writes nothing
when it opens the device. The first full write covers the whole matrix,
all output masters, both mic gains and both +48 V bits. It happens when
the saved scene is applied, or when metering starts if there is no saved
scene. The saved scene includes +48 V, so a scene saved with +48 V on
switches it on at connect. Without a saved scene, the model is:

- playback pair *n* → output pair *n* at 0 dB, centred;
- every input crosspoint at off;
- every output master at 0 dB;
- mic gain 6 dB, +48 V off.

Mic inputs are mono sources panned into both sides of a pair. ADAT
inputs and playback channels feed only their own side; their pan acts as
a balance. Mute, solo and the output masters are applied host-side by
writing 0 or the scaled level.

Request `0x15` selects loopback: `wValue` 1 = on, 0 = off, `wIndex` =
capture channel 0..9. With loopback on, the capture channel records the
output of the same index (post-master). The backend keeps loopback off.

## 4. Preamps and settings

| Control | Request | `wValue` | `wIndex` |
|---|---|---|---|
| Mic gain AN1/AN2 | `0x1a` | 0..18 | 0, 1 |
| +48 V | `0x17` | bit mask to set | bit mask to change |

- Mic gain: `gain dB = 6 + 3 · wValue`, 6..60 dB. The unit has no gain
  readback. The backend snaps the model to the 3 dB grid and writes the
  gain at the first full write and on every change.
- +48 V: bit 0 = AN1, bit 1 = AN2. The register is write-only. The
  backend writes one bit at a time with the matching mask, so the other
  bits are unchanged, at the first full write and on every change.
- Sample rate: set by the host stream. The backend writes no rate request
  and shows the rate from the hwdep INFO block.
- Controls the backend does not implement (Dim, Mono, Speaker B, Cue,
  clock source, EQ and others) return an error.

## 5. Metering

The unit streams meter packets on bulk IN `0x85` without a request. The
driver delivers each packet as an `FFUC_EV_METER` hwdep event. A packet is
456 bytes:

| Bytes | Contents |
|---|---|
| 0..303 | 38 little-endian `u64` mean-square values |
| 304..455 | 38 little-endian `u32` peak values |

| Slot | Channel |
|---|---|
| 0..9 | inputs (§2) |
| 10..23 | playback 1..14 (13/14 = host reverb return) |
| 24..37 | outputs 0..13 (§2) |

```text
peak  dBFS = 20 · log10(peak  / 2^27)
power dBFS = 10 · log10(power / 2^55)
```

A value of 31 (`0x1f`) marks a channel below range. Slots of channels the
unit does not carry read 0.

The backend shows the peak value. It holds the highest peak per slot until
the UI reads it. Input strips use slots 0..9, playback strips slots
10..21 and output strips slots 24..35. The meters are read from the DSP,
so they do not claim the capture PCM.

## 6. State ownership

The unit has no mixer readback. The host owns the mix: the scene model
holds every fader, pan, mute, solo and master, and the backend sends only
the `0x12` addresses whose level changed since the last write.
