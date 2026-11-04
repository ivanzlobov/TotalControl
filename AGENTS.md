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

## Git workflow

`main` is the protected integration branch. Every change lands through a pull
request; do not commit or push directly to `main`.

- Start from current `main`: `git switch main && git pull --ff-only`, then
  create a short-lived branch.
- Use one branch and one PR per logical change. Branch prefixes are `fix/`,
  `feat/`, `perf/`, `docs/`, `test/` and `chore/`.
- Keep commits atomic. Explain why the change is needed in the commit message
  and in the PR description, using `.github/pull_request_template.md`.
- Push the branch, open a draft PR early, and wait for CI. Do not merge with a
  failing or missing required check.
- Use squash merge by default so `main` stays linear and each PR has one
  revert point. Use rebase-and-merge only for a deliberately clean commit
  series; do not add merge commits.
- After merging, delete the branch, fast-forward local `main`, and run
  `git fetch --prune`.
- Rebase a feature branch on `origin/main` when needed and push it with
  `--force-with-lease`. Never force-push `main`.
- Do not mix unrelated driver, UI, documentation and tooling changes in one
  PR unless they are required for the same tested behaviour.
- Hardware and driver PRs record the device model, transport, firmware,
  quantum, test evidence, and confirmation that the user scene and
  `~/.config/totalcontrol/gui.json` were preserved.

## Working rules

- Use the NAS mailbox in `/media/nas_data/John/Electronica/rme/coordination/`
  for Windows/oracle questions and acknowledgements. Linux owns
  `LINUX-STATUS.md`, `LINUX-REQUESTS.md`, `DECISIONS.md`, `analysis/*` and
  `coordination/inbox/windows/*`.
- Push only to our own repository, `jspx-projects/TotalControl`.
- The UI is part of this repository. Do not add an external TotalControl development
  remote; the imported upstream history is on `archive/tuxmix-upstream`.
- Preserve the user scene and `~/.config/totalcontrol/gui.json`. Leave the device on
  the user's scene after hardware tests.
- One Fireface at a time.
- Verify with `cargo test -p totalcontrol-core --lib --features alsa` and
  `cargo test -p totalcontrol-gui`.
- Do not replace device-backed behaviour with host-side approximations. If the
  unit supports something and the register is unknown, add it to
  `docs/work/` and request a capture.
