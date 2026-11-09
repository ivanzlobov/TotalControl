# First-generation Babyface: open questions

Working notes for [BABYFACE-PROTOCOL-SPEC.md](../spec/BABYFACE-PROTOCOL-SPEC.md).
Not part of the specification.

## Mic gain scale

The implemented mapping is `gain dB = 6 + 3 · wValue` (`0x1a`, 0..18), the
range the vendor driver declares. One hardware measurement read +12 dB
between `wValue` 0 and 2 instead of +6 dB; that measurement was taken while
the input had a wiring fault. Repeat it with a clean source and a known
level before changing the scale.

## Not implemented

| Feature | Register | Status |
|---|---|---|
| Input and output EQ, Low Cut | bulk OUT `0x0a`, 64-byte block per channel | layout known from the vendor application; needs a driver ioctl and a hardware check |
| Hi-Z on input 2 | `0x17` bit 4 | from the vendor application; not tested |
| Analog output level AN1/AN2/PH3/PH4 | `0x1a` `wIndex` 2..5 | scale declared as −140..+6 dB in 0.5 dB steps; writing 0 to `wIndex` 4/5 had no audible effect |
| Native init word | `0x10` `0x4241`/`0x46cf` | sent by the vendor driver; the module uses the UCX init and the unit works without it |

## Requests to identify

- `0x16` (36 writes at init in the vendor driver), `0x20`.
- `0x19` status read: sync and lock flags.
- Interfaces 3 (`0x09`/`0x88`) and 4 (`0x8b`).

## Captures wanted

- A USB capture of the vendor application at start-up, to confirm the
  `0x17` global bits and the EQ block upload.
