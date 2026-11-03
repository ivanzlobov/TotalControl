## Problem

What is broken, missing, or being changed? Include the observed symptoms and
the evidence that established them.

## Root cause

Why does the current code behave this way? Link the relevant specification,
working note, capture, or issue.

## Change

What does this PR change? Keep this focused on the problem above. Note any
user-visible behaviour and the device writes involved.

## Tests

- [ ] `cargo test -p tuxmix-core --lib --features alsa`
- [ ] `cargo test -p tuxmix-gui`
- [ ] Kernel build, when `kernel/` changed
- [ ] `scripts/audio-health.sh`, when the driver or audio path changed
- [ ] Hardware test, when the device transport or DSP changed

Describe the commands, counters, and results. For hardware tests, state the
device model, transport, firmware, quantum, and whether the user scene and
`~/.config/tuxmix/gui.json` were preserved.

## Risk and rollback

What could regress? How can this be reverted safely? Call out any temporary
workaround, remaining test gap, or follow-up needed in `docs/work/`.

## Documentation

- [ ] Specifications updated when the implemented model changed
- [ ] `docs/TUXMIX_Manual.md` updated for user-facing behaviour
- [ ] Working notes and open questions kept in `docs/work/`
