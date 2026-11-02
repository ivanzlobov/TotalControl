# Agent guidance

Read this file before starting work in this repository.

## Documentation rules

`docs/spec/` contains authoritative specifications. Write them as reference
documents for the current implementation, not as a work log.

- State **what something is** and how it behaves. Do not describe what it is
  not, what it used to be, or what went wrong before.
- No historical notes, retractions, changelogs, capture diaries, or "oracle
  said" narration in a specification.
- No personal identifying information. Do not put serial numbers, USB
  vendor/product IDs, hostnames, usernames, file paths with a home directory,
  or device addresses in a specification.
- Keep specifications minimal. Tables and short normative paragraphs are
  preferred over prose.
- Put open questions, TODO lists, capture requests and working notes in
  `docs/work/`. They are not part of the specification.
- The specification describes the shipped model and its device writes. When
  evidence is incomplete, state the implemented mapping positively and keep
  the open question in `docs/work/`.
- Base UI figures on current TotalMix FX screenshots; the vendor manual can
  lag the shipping application.
- The end-user documentation is `docs/TUXMIX_Manual.md`. The specifications
  are for developers; user-facing behaviour described in them must have a
  counterpart in the manual, and how-to material belongs in the manual, not
  in a specification.

Authoritative files:

- `docs/spec/UCX-CONTROL-SPEC.md` — UCX transport, registers, controls,
  metering.
- `docs/spec/OSC-CONTROL-SPEC.md` — TotalMix FX OSC transport, pages, scales,
  link rules and UCX register mapping.
- `docs/spec/TUXMIX-SPEC.md` — visible UI, panels, state and persistence.
- `docs/spec/UC-CONTROL-SPEC.md` — UC-specific protocol and controls.
- `docs/spec/BABYFACE-PRO-CONTROL-SPEC.md` — Babyface Pro FS channel model,
  ALSA surface, vendor USB protocol, DSP EQ blocks and metering.

## UI rules

All UI layout and interaction rules live in `docs/spec/TUXMIX-SPEC.md`.
Do not duplicate them here; when a rule is missing, add it to the spec.

## Working rules

- Use the NAS mailbox in `/media/nas_data/John/Electronica/rme/coordination/`
  for Windows/oracle questions and acknowledgements. Linux owns
  `LINUX-STATUS.md`, `LINUX-REQUESTS.md`, `DECISIONS.md`, `analysis/*` and
  `coordination/inbox/windows/*`.
- Push only to our own repository, `jspx-projects/tuxmix-fireface`.
- The UI is part of this repository. Do not add an external TuxMix development
  remote; the imported upstream history is on `archive/tuxmix-upstream`.
- Preserve the user scene and `~/.config/tuxmix/gui.json`. Leave the device on
  the user's scene after hardware tests.
- One Fireface at a time.
- Verify with `cargo test -p tuxmix-core --lib --features alsa` and
  `cargo test -p tuxmix-gui`.
- Do not replace device-backed behaviour with host-side approximations. If the
  unit supports something and the register is unknown, add it to
  `docs/work/` and request a capture.
