# TotalControl GUI specification

This document defines the visible TotalControl interface for the Fireface UC
and UCX: layout, controls, panels, gestures and motion. Runtime structure,
state ownership and device I/O are in
[`../TOTALCONTROL-ARCHITECTURE.md`](../TOTALCONTROL-ARCHITECTURE.md). The
end-user manual is [`../TOTALCONTROL-Manual.md`](../TOTALCONTROL-Manual.md).

## Contents

1. [Mixer layout](#1-mixer-layout)
2. [Channel strip](#2-channel-strip)
3. [Runtime reference](#3-runtime-reference)
4. [Settings panel](#4-settings-panel)
5. [Equalizer panel](#5-equalizer-panel)
6. [Dynamics panel](#6-dynamics-panel)
7. [FX panel](#7-fx-panel)
8. [Output balance](#8-output-balance)
9. [Pan and balance](#9-pan-and-balance)
10. [Motion](#10-motion)
11. [Stereo link](#11-stereo-link)
12. [Control Room](#12-control-room)
13. [Latency display](#13-latency-display)
14. [State references](#14-state-references)
15. [Control invariants](#15-control-invariants)

## 1. Mixer layout

The Mixer is the default view on startup. TotalControl presents three channel
rows:

1. Hardware Inputs
2. Software Playback
3. Hardware Outputs

Each row is a horizontal row of channel strips. Every section carries an
uppercase header — HARDWARE INPUTS, SOFTWARE PLAYBACK, HARDWARE OUTPUTS,
CONTROL ROOM — and each header spans the width of its own strips, opened by an
accent tick and closed by a matching block; the slack between sections is the
break in the line. When a row is wider than the
mixer area, one horizontal slider scrolls the three rows together. The slider
sits in the main area, immediately left of the right sidebar.

The right sidebar contains the device card, Undo/Redo, global mute/solo, the
FX knob, group and snapshot controls, and layout. It stays in the layout as
the window narrows. Its default width is 140 px; it accepts resizing and has
its own collapse toggle.

The global row is `M`, `S`/`F`, `F`.

The left button is the **Mute master**. Membership is automatic: every channel
mute joins the set, and while the master is lit the channel mutes. The master
is lit from the first frame with an empty set. A press suspends the set: every
member unmutes and the master and each held channel show the pause mark;
pressing it again mutes the set. A member click toggles that channel and
removes it from the set, a linked pair leaving together; muting any other
channel mutes it and adds it, whatever the master's state. The master can also be switched off while the set
is empty. The set is global: it silences the channel on every routing.

The middle button is the **Solo master**, the same machine with the pause mark
after the letter, over the current submix; it is also lit from the first frame
and its membership is automatic. In PFL mode it shows `F` while an input is
listening; a press then cancels the active PFL and does not restore it.

The right `F` is the **global fader group** master, not the FX panel.
Shift+click a strip adds or removes it; the first member turns `F` on. A plain
click moves the focus and leaves the group untouched. While `F` is on, the
members of each row gang level moves and their caps carry the group colour. A
press turns it off: the members stop ganging and lose the colour but keep
their membership; another press restores both. Pan never gangs. The group is
per row: input faders gang with input faders, software-playback faders with
their own, hardware-output faders with theirs, so a group spanning rows moves
one row at a time. It never affects the stored fader groups `1`–`4`; an empty
group leaves `F` off.

The three masters are outline switches in the colour of the channel control
they stand for — mute blue, solo yellow, fader-group colour — and take a muted
version of that hue while off. They use the normal sidebar button box and
centre their label in both axes. Mute and solo are lit from the first frame;
switching one off always shows the pause mark. The pause mark is two short,
thick bars, half the label height: before the `M`, after the `S`.

The `groups` card has four rows of storage places: `M1`–`M4` for the mute
link, `S1`–`S4` for the solo link and `1`–`4` for the fader link, with `edit`
and `clear` below.  `edit` opens a group-assignment session and every storage
place flashes.  Choosing a place keeps only it flashing, seeds it with the
group's current members and turns the button into `end edit`; clicks on
strips then add or remove members.  `end edit` stores the membership and
engages the chosen link, choosing another place stores the current one and
starts the next, and Esc cancels the session.  A linked place keeps its
steady colour: fader groups red, blue, green and violet, mute steel blue,
solo amber.  A fader-linked group tints the caps of its members with its
colour; the global fader group tints them yellow while it is ganged.

The Control Room closes the Hardware Outputs row, flush with the mixer's right
edge.

The device card reads:

```text
Internal, <sample rate>, <buffer>
```

## 2. Channel strip

A full strip contains:

- channel name and type;
- pan or balance knob;
- mute and solo; outputs use mute and cue;
- level fader and a meter with a coloured RMS bar, a matching peak line and
  peak hold;
- a right-hand column in fixed order: Trim, settings gear, EQ, Dynamics;
- the channel's active panel, when open.

Every full strip has the same height. A collapsed strip uses that same
height: the meter absorbs the space that the fader and controls no longer
use. A Settings, EQ, Dynamics, Trim or Route panel opens beside the strip at
that same height. The standard height is 350 px at scale 1.0.

### 1.1 Options menu

The top bar carries `OPTIONS` beside the view tabs. It opens a menu with
`Reset Mix` and `Settings`. `Reset Mix` is a submenu; choosing an entry closes
the menu, rewrites the whole mixer and pushes it in a single scene apply:

| Entry | Effect |
|---|---|
| Straight playback with all to Main Out | playbacks 1:1 to the outputs, each also mixed to Main |
| Straight playback | playbacks 1:1 to the outputs, nothing else routed |
| Clear all submixes | every input and playback send to zero |
| Clear channel effects | EQ, low cut, dynamics, Auto Level, Reverb, Echo and Width off |
| Set output volumes | outputs 0 dB; Main and Speaker B −10 dB |
| Reset channel names | user names cleared |
| Set all channels mono / stereo | every pair's stereo/mono mode |
| Set inputs mono, outputs stereo | the ADM layout |
| Total reset | straight playback to Main plus effects off and output volumes set |

Menu items and dialog body text use the Control Room Settings size: 16 px at
scale 1.0.  The Workspace and Options menus use the same 224 px width at
scale 1.0; long Reset Mix labels wrap inside the item.  Settings use standard
controls: independent switches are checkboxes, mutually exclusive choices are
radio buttons, and buttons are reserved for actions.

Confirmation dialogs never use `Yes`/`No`.  The confirming button names the
action in one word (`Open`, `Load`, `Discard`, `Delete`); the other button is
`Cancel`.

### 1.2 Workspace menu

The top bar has `WORKSPACE` immediately before `OPTIONS`. Its menu contains:

| Item | Action |
|---|---|
| `Open Workspace…` | choose a `.tcws` file and replace the current workspace |
| `Save Workspace` | save to the current file, or open Save As when there is no file |
| `Factory Workspaces  ▸` | open the built-in direct surround layouts |
| `Save Workspace As…` | choose a new `.tcws` file |

The file dialogs remember the last used folder in `gui.json`. With no saved
folder they start in `Documents`, falling back to the home directory. A
workspace written for another device model is refused. Opening or loading a
workspace first shows an in-app confirmation: loading a workspace will
overwrite the current workspace, with `Cancel` and the action word (`Open` or
`Load`).

The Factory Workspaces submenu lists `5.1 Direct`, `7.1 Direct`, `5.1 → 4.0`
and `7.1 → 4.0`. A factory workspace starts from a clean mixer rather than
the current one: routing, FX sends and returns, Reverb/Echo, EQ, dynamics,
mutes, solos, labels, groups, layouts and snapshots are initialised. Device
clock, sample rate, references and device-owned preamps are retained. The 4.0
layouts downmix into the existing 5.1/7.1 sink rather than creating a new
sink.

Direct layouts map one source channel to the first available hardware-output
pairs. 4.0 maps Center and LFE to both front channels at 0.707, and maps the
surround pair to the rear pair. In 7.1 → 4.0 both Rear and Side feed the rear
pair at unity, preserving the sum of the original four surround speakers.

Front L/Front R and Side L/Side R stay linked pairs; Center, LFE and Rear/Back
channels are mono where applicable. Linked user labels drop the L/R suffix
(`Front L` + `Front R` displays `Front`); a split pair displays the two
individual names. The used outputs form group 1. Factory workspaces switch
the control strip to custom names, collapse the FX column and enable Matrix
Mono so the mono routing can be edited. Control Room assignments that use one
of the chosen output pairs are cleared. A factory workspace has no file:
`Save` opens Save As so the user can keep it.

A workspace contains the current mixer, the eight named snapshots, labels,
groups (including the global fader group and its ganged state), six layout slots,
collapsed/expanded strips, view and sidebar state,
Control Room settings and assignments, virtual-surround settings and window
size. It stores the sample rate, pitch/varispeed and active Dim, Speaker B,
Cue, External Input and PFL states, plus each output's Talkback receive flag
in the mixer scene. Active Talkback is momentary and is not stored; a
workspace opens with Talkback off. It excludes undo/redo, open dialogs,
transient selections, app preferences, the unit's six device setups and the
device serial number. Selecting a snapshot slot exposes a name field below
its Store action.

`WORKSPACE` shows `*` while the host state differs from the saved file.
Menus start below the top bar, so clicking the other top-bar menu switches to
it immediately on the first press; submenu items open on the first click.

### 1.3 App settings

`Options > Settings`, the tray Settings entry and the `,` shortcut open the
app-level settings sheet. It is a centered modal dialog with a dimmed scrim,
not a sidebar popover; it holds BACKGROUND, STARTUP SYNC, SURROUND and OSC
sections.  Independent options are checkboxes, one-of-many options are radio
buttons, and the only footer action is `Close`.

SURROUND has the `Virtual 5.1 output` and `Virtual 7.1 output` checkboxes. A
checked output creates a PipeWire sink while TotalControl runs and removes it
when the application exits. The setting is persisted. The sinks follow the
selected Fireface/MR18 Pro Audio device and bridge to the first six or eight
SW playback channels.

OSC has an `Enable OSC control surface` checkbox, `Classic` and `Global`
radio buttons, a receive-port field, a send-port field and `Apply`.  The
`Show OSC debug log` checkbox sits directly below the enable checkbox and is
disabled and greyed out while OSC is off; turning OSC off also hides the
panel.  `Classic` is the banked TotalMix FX 1.96–2.11 address space; `Global`
is the absolute-channel TotalMix FX 2.1+ address space. Enabling it starts
the loopback UDP surface; disabling it stops it. The switch, protocol and
both ports are persisted. The command line `--osc`, `--osc-mode`, `--osc-port`
and `--osc-send-port` remain a development override.

### 2.0 FX fader

Every strip on a unit with an FX bus carries a second, thin fader at the top
of its right-hand control column.  The T / gear / EQ / D column starts
directly below that fader, as high as the strip allows; spare height stays
below the column, not between the fader and the buttons.

The symbols sit on one grid shared by all strips.  Every strip has a main
fader; the column is anchored to it, below the FX fader when the unit has an
FX bus and directly below the fader top otherwise.  The column holds the
symbols the channel can have, in the fixed order `T`, settings gear, `EQ`,
`D`.  The UC has `T` and the gear; the UCX adds `EQ` and `D` on inputs and
outputs.  Software playback has `T` and the gear.  A hardware output without
`T` keeps an empty `T` slot so the gear, `EQ` and `D` stay on the shared
grid.  The gap below the FX fader is `theme::SPACE_LG` plus the column's own
spacing (12 px at scale 1) and the symbols are `theme::SPACE_MD` (6 px)
apart; empty slots use the same heights and gaps.

| Strip | Fader | Register | Fill |
|---|---|---|---|
| Input | FX send | `NN01`, 0.1 dB | orange |
| Software playback | FX send | `0x47A0` family, linear, `+0x41` mirror | orange |
| Hardware output | FX return | `NN03`, 0.1 dB | green |

Both faders slide in a dark fader slot of the same width.  The slot fill
carries the send/return colour as it is raised, so an untouched control shows
no colour at all.  The thin FX fader has no level marks and no ruler: no tick
marks, no 0 dB reference tick; only its coloured slot fill and cap are
drawn.

The main fader's value sits in the strip's bottom row, centred on the fader
track; the collapse button sits at the right edge of that row.

Dragging a fader or knob keeps the ordinary pointer; the grab hand is not
used, since it covers the values it hovers over.

### 2.1 Knob appearance

- Every knob has the same near-black face, a step darker than the strip card,
  so its rim and arc read as colour against the card.
- Every interactive knob draws a faint full-range track and a value arc from
  the range minimum, or from top centre for pan and balance.
- An always-on coloured outline is not used.  The full-range track stays
  neutral; colour appears on an arc only while the control is live.
- The default arc colour is white. Semantic colours are:
  - input gain: orange;
  - EQ gain: red, green and blue for bands 1–3;
  - pan and balance: accent colour;
  - dynamics: solo colour.
- Frequency labels are compact: `20`, `120`, `1k`, `1.5k`, `10k`. There is at
  most one decimal and no unit suffix; the row label supplies the unit.
- Label size is 15 px for 1–4 characters, 11 px for 5–6, and 9.5 px for longer
  frequency labels.

### 2.2 On/off controls

An on/off switch sits at the left of its row when the row has a label. This
applies to Dynamics, Auto Level, Reverb and Echo.

### 2.3 Labels and alignment

- Related controls and values share a row or column. A control under another
  control uses the same column width and alignment.
- A column of knobs shares one vertical centre line. A secondary readout such
  as the Echo BPM sits at the underside or the right of its knob without
  moving the knob off that line.
- Ordinary nouns in slideouts are lowercase: `gain`, `attack`, `compressor`,
  `auto level`.
- The minimum gap between distinct controls is `theme::SPACE_TIGHT`
  (2 px at scale 1).
- A knob and its label use `theme::KNOB_LABEL_GAP`. The `attack` label and
  its knob are the reference distance.

### 2.4 Tab appearance

- The settings gear is a 20×26 tab with a 14 px glyph.
- Trim, EQ and Dynamics use the 20×22 tab with a 13 px glyph.
- A slideout tab shows only its glyph at rest.  Hover gives it a face one
  step lighter than the open face and still darker than the channel surface.
  An open slideout keeps the darker face behind its tab.  When a face is
  drawn it extends from the 20 px column to the complete right edge of the
  channel; the glyph stays centred in the column, not in the extended face.
  The channel minimize button follows the same right-edge rule: its glyph
  stays centred in its B1 column and its face reaches the channel's right
  edge.
- The T, EQ and D tabs keep their glyph only, in orange, while their setting
  is active and the slideout is closed: trim below 0 dB, EQ enabled, or
  dynamics on. While the slideout is open the tab carries no orange; the
  panel's own on/off control is the single indication. That control uses the
  orange highlight while it is active: the EQ button when EQ is enabled and
  the Dynamics ON button while dynamics is on. The gear tab only marks the
  open panel.
- Outlines belong to buttons, not slideout tabs. A slideout tab never draws
  a button outline or a copy of the panel-open marker. Hover brightens the
  glyph and adds the lighter hover face, but never uses orange.
- The T, gear, EQ and D tabs keep the same fixed slot with the slideout open
  or closed. Only the panel grows beside them; the tabs themselves never
  move.
- The glyph is centred on its column before and after opening.

### 2.5 Control sizing

There are two sizing families, one per surface:

| Surface | Standard control height at scale 1 |
|---|---|
| Channel strip (`M`/`S`/`CUE`, tabs, collapse, submix box) | 18 px |
| Settings slideout (buttons and dropdowns) | 24 px |

Standard controls use one width per stack or panel: a single button spans it;
a pair in one row splits it with one gap. Menu rows share the menu width.
Black button faces use a raised near-black, one step lighter than the page
and rail black.  Engaged channel `M`, `S` and `CUE` buttons are outlines, not
filled pills: the face stays dark, and the border and text take mute blue,
solo yellow or cue green respectively.
Slideout tabs (`T`, gear, `EQ`, `D`) and the channel minimize button keep
their fixed glyph column; their faces extend to the channel's right edge
(§2.4). Compact navigation and utility buttons keep their own fixed or
label-sized size. A strip's `M`/`S` pair (hardware inputs and software
playback) or `M`/`CUE` pair (hardware outputs, including Control Room
channels) shares the strip's control width; an output never shows `M` alone.

- The strip's submix box spans the strip width at the strip height, with the
  bus name vertically centred at the left and the arrow at the right.
- The dB readout under the main fader is centred on the fader track and has
  no unit suffix; the numeric scale is printed inside the VU element, and
  the level marks run from its right edge to the track. The collapse button
  sits at the right edge of the readout row.

### 2.6 Pointer gestures

Dragging a fader or knob changes it; the scroll wheel nudges it. Holding Shift
reduces the movement for fine adjustment; this applies to faders, knobs,
matrix cells and EQ handles. A press captures the pointer until release, so a
drag continues when the pointer leaves the control or the window.

A double click on a fader, knob, FX fader or EQ handle resets that control to
its default. A double click on the dB readout opens a numeric entry field. A
double click on a channel name starts a rename, on the strip background
collapses the strip, and on a Matrix cell toggles the cell between off and
unity. Ordinary push buttons act once per click and have no separate double
click.

When a collapse starts on a channel with an open slideout, the slideout
closes first; the width tween starts after the panel has finished sliding
away.

### 2.7 Meters

#### 2.7.1 VU meter element

The VU meter is one element.  Its channel spans the complete element height.
The OVR segment occupies the top of that channel, inside the element bounds;
no part of the element is drawn outside them.  The level scale runs from
-65 dB at the bottom to +6 dB at the top over the complete channel height,
behind the OVR segment.  A mono or split channel shows one centred bar just
wide enough to cover the level text; a linked pair shows two bars with a
small gap, one per channel.  Each bar measures its own channel.  The level
labels are centred in the VU element whether it carries one bar or two.

- an **RMS bar** with a slow time constant, coloured in three bands: green
  below -3 dB, yellow from -3 dB to 0 dB and red above 0 dB;
- a **peak line** with zero attack, held for a fixed two seconds and then
  decaying; its colour follows the same green/yellow/red bands;
- the **OVR segment** at the top, which lights from -0.1 dBFS.

The VU well is one shade darker than the channel card; it is not a separate
black panel.  The numeric level text is centred in the VU channel at 60, 40,
20, 10, 6 and 0.  There is no +3 text label.  The labels use a light grey one
step below the primary text tone.  The bars are opaque; a raised bar may
hide the labels underneath.  The VU element has no level stripes or tick
marks.

Peak hold applies only to the peak line. The RMS bar is never held; it
continues to follow the RMS value. The whole region above 0 dB is red. The
bar is mathematically correct RMS, so a full-scale sine reads about -3 dB;
no RMS +3 dB shift is applied. There is no separate red guide across the
meter bar.

Falling levels use a 20 dB/s release. The peak hold is fixed at two seconds.
The OVR segment lights from -0.1 dBFS; there is no configurable hold time or
OVR sample count. If the device disconnects, the meters fall to silence with
the normal release instead of freezing at their last reading.

Input and playback meters are pre-fader and post-trim: the trim changes the
reading, the send fader and selected submix do not. Hardware output meters
are post-fader and go to zero when the output is muted.

#### 2.7.2 Fader scale

The fader scale uses the same -65 dB to +6 dB range as the VU element, so
each VU label and the matching fader mark share one vertical centre line.
The scale has an unlabelled green tick at -3 dB, the full-scale-sine RMS
target, and a red tick at +3 dB.
Each tick occupies 60% of the span from the VU element's right edge to a
little under the fader cap, anchored at the cap end; its left end stays clear
of the VU.  The green and red ticks are half the length of the other
graduations and share the same right end.  The 0 dB graduation is slightly
whiter and is repeated just right of the fader slot.  The cap covers the line
at its current level. The cap centre line is dark grey, aligns with the tick,
spans the complete cap width and is twice the thickness
of the other cap ridges (2 px at zoom 1).  The two outer ridges are inset from
the cap ends and lighter than the centre line.  The cap has a physical half-height, so its centre
stops at -64.5 dB at the bottom of the track.  The position below that
selects the -65 dB value, shown as inf; the shared scale mapping keeps the
VU labels and fader marks on the same line.

The narrow dark groove under the cap is the **fader slot**.  The channel
fader and the thin FX fader use the same slot width: 4.3 px at zoom 1.  The
slot is slightly lighter than the page black.  The channel fader's fill
below the cap is only slightly lighter than the channel surface, so the level
is visible without a bright white rail; the cap stays light.

### 2.8 Channel layout model

The executable template schema and the layout preview are defined in
[`TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md`](TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md).

#### 2.8.1 Definition and sources

A strip is described at three levels:

| Level | Meaning | Defined by |
|---|---|---|
| Channel kind | input, software playback, hardware output | `ChannelKind` |
| Channel template | a kind plus its linkage and optional elements: input mono, playback stereo, output mono | `standard_elements` in `totalcontrol-layout` |
| Channel | one instance: AN1, SW1/2, OUT3, MAIN | the runtime channel list |

The layout is global.  One set of dimensions, visual tokens and colours
applies to every strip in every view, workspace and scene; no workspace and
no device scene stores its own layout.

Three sources answer three different questions:

| Question | Source |
|---|---|
| What must hold: rules, invariants, element vocabulary | this specification |
| What the values are: dimensions, font sizes, colours | `LayoutMetrics`, `VISUAL` and `ColorScheme` in `ui/totalcontrol/totalcontrol-layout/src/lib.rs` |
| What a user changed | `$XDG_CONFIG_HOME/totalcontrol/gui-layout.json` |

This section states the rules and does not repeat the values, so a tuning
change touches the code and its generated files only.

The schema at `docs/schema/gui-layout.schema.json` mirrors the values for
editors and is kept identical to the code by a test.  The example at
`docs/examples/gui-layout.json` is written from the built-in defaults by
`channel-preview --example`; it is a snapshot and a regression fixture, not a
second source.  The layout preview and the runtime both read the user
document; the runtime falls back to the built-in layout when none loads.  The
document's colour categories are installed into the runtime theme at startup;
the tones derived from them stay in code.

#### 2.8.2 Grid and scaling

A channel is a three-area (Top, Middle, Bottom) by three-column (Left,
Centre, Right) grid.  An element is named `ACr`, where `A` is the area, `C`
the column and `r` the row within that area; `r` is omitted when the area has
one row.  `TC1` is the name, `TL3` the left button of the third Top row and
`MR4` the fourth element of the right Middle stack.

The shipped strip is 90 px wide and 350 px tall at zoom 1.0.  The Top, Middle
and B1 rows use a content box inset on the left, right and top; there is no
bottom inset.  B2 spans the full card width at the bottom edge.  The
slideout-tab and minimize faces are the right-hand exception: their glyphs
stay in the content box, but the face extends through the right padding to
the card edge.  The collapsed strip is a separate narrow layout.

Every dimension, font size and gap is multiplied by the UI zoom
(`0.75..1.75`); the aspect ratio is invariant.

Rows are separated by a gap.  The Centre column is the reference centre line
for the Top rows and the Bottom readout.  Columns are centred on their own
line.  An element that is absent keeps its row, so an element below it does
not move.

Channel elements never overlap, and every part of an element is inside its
own bounds.  The VU bars and the level text are inscriptions rather than
controls: the moving VU wash and the fader cap may cover them.

The VU element and the channel fader share one vertical dB scale and always
have the same top, bottom and height.  The vertical centre of each VU level
label is on the fader mark for the same level; a label at 60 and the fader
mark at 60 are the same line.

When vertical space is short, the VU element and the channel fader are
scaled down together by the same factor; their level scales stay aligned and
no other element changes size.

#### 2.8.3 Vertical rows

| Ref | Element | Column |
|---|---|---|
| T1 | channel name and type tag | C |
| T2 | pan or balance knob | C |
| T3 | `M`/`S` pair, or `M`/`CUE` pair | L / R |
| M1 | channel strength readouts, L and R | L / R |
| M2 | VU, fader and right-hand stack | L / C / R |
| B1 | dB readout and collapse button | C / R |
| B2 | submix selector, input and playback only | C |

The rows plus their gaps and the top padding fill the card height; there is
no bottom padding.  M1 has two readout cells that match the T3 buttons, but
is drawn as text only.  The VU element, fader track and FX fader all start at
the top of M2.  The VU OVR segment is inside the VU element at its top.  B1
sits directly below M2 and the fader track ends above it, so the cap never
covers the readout.

B2 is a full-width bar flush with the card's bottom edge and clipped by the
card radius.  A hardware-output strip has no B2 and leaves that row and its
gap empty, so B1 keeps the same height on every channel kind.

#### 2.8.4 Horizontal columns

| Element | Position |
|---|---|
| VU element | left column, labels centred |
| fader track | centre column, centred on the content centre line |
| fader slot | inside the track |
| fader cap | centred on the slot |
| right-hand control column | glyph column in the content box, faces to the card edge |
| pan or balance knob | centre column box |
| T3 buttons | left and right cells |
| M1 readouts | the same cells as T3, text only |
| B1 collapse glyph | right column, face to the card edge |
| B2 submix selector | full card width |

The lane between the VU element and the fader track carries the level marks;
each mark occupies a fixed fraction of the lane-to-cap span, anchored at the
cap end and clear of the VU.  The level labels are centred in the VU channel;
the moving bars cover them.

A linked pair draws two bars with a gap; a split or mono channel shows one
centred bar just wide enough to cover the level text.  The labels stay
centred in the element in both cases.  The OVR segment occupies the top of
the VU element, inside its bounds; the level scale runs the full element
height behind it.

M1 is drawn as text only, right-aligned inside each cell with a small inset
so the values sit inside the edges of the T3 buttons above.  The left cell is
the left channel and the right cell is the right channel; no `L` or `R`
prefix is printed.  The cell width accommodates `-99.9`.  Values of magnitude
100 and above omit the fractional part (`100`, `120`, `-150`).  Silence, and
anything below the unit's -150.5 dBFS floor, reads `-∞`.  The values refresh
every 0.25 s; the VU bars continue at the meter tick rate.  B2's label is
always centred on the channel width and the arrow sits at the right edge.

#### 2.8.5 Right-hand stack

The right-hand column starts level with the top of the VU element and the
fader track.  It holds:

| Ref | Element | Present on |
|---|---|---|
| MR1 | FX send or FX return fader | every strip with an FX bus |
| MR2 | `T` trim tab | inputs and software playback |
| MR3 | settings gear | every strip |
| MR4 | `EQ` tab | UCX inputs and outputs |
| MR5 | `D` dynamics tab | UCX inputs and outputs |

The glyph column keeps its glyphs in the content box; the `T`/gear/`EQ`/`D`
tab faces and the B1 minimize face reach the card's right edge.

The FX fader has no level marks and no ruler: no tick marks, no 0 dB
reference tick; only its coloured slot fill and cap are drawn.  The FX fader
is followed by a gap, and the tabs below it are evenly spaced.  A missing tab
keeps its row where a later element depends on it: an output gets an empty
MR2, so the gear and the UCX tabs stay on the input grid.

#### 2.8.6 Channel kinds

| Ref | Input | Software playback | Hardware output |
|---|---|---|---|
| T1 | name | name | name |
| T2 | pan | pan | balance, when the backend supports it |
| T3 | `M` `S` | `M` `S` | `M` `CUE` |
| M1 | text-only L/R readouts | text-only L/R readouts | text-only L/R readouts |
| ML | VU element, labels centred | VU element, labels centred | VU element, labels centred |
| MC | fader and cap | fader and cap | fader and cap |
| MR1 | FX send | FX send | FX return |
| MR2 | `T` | `T` | empty |
| MR3 | gear | gear | gear |
| MR4 | `EQ` on the UCX | empty | `EQ` on the UCX |
| MR5 | `D` on the UCX | empty | `D` on the UCX |
| B1 | dB and collapse | dB and collapse | dB and collapse |
| B2 | full-width bottom bar | full-width bottom bar | empty |

A linked pair is one strip and shows two VU bars with a small gap.  A split
pair is two adjacent strips; each has one VU bar and its own fader.  The Control Room
output strips use the hardware-output layout; their stereo link is locked.

#### 2.8.7 Examples

Input, linked pair, UCX:

```text
┌──────────────────────────────────────┐
│                TC1 name              │  T1
│                TC2 pan               │  T2
│        TL3 [ M ]   [ S ] TR3         │  T3
│              -6.0          -6.0       │  M1
│  ML VU+labels │ MC fader │ MR1 FX send │  M2
│               │   + cap  │ MR2 T       │
│               │          │ MR3 gear    │
│               │          │ MR4 EQ      │
│               │          │ MR5 D       │
│            BC1 0.0       BR1 [-]      │  B1
│ BC2 [        Main         ▸ ]         │  B2
└──────────────────────────────────────┘
```

Software playback, linked pair, UCX:

```text
┌──────────────────────────────────────┐
│                TC1 name              │  T1
│                TC2 pan               │  T2
│        TL3 [ M ]   [ S ] TR3         │  T3
│              -6.0          -6.0       │  M1
│  ML VU+labels │ MC fader │ MR1 FX send │  M2
│               │   + cap  │ MR2 T       │
│               │          │ MR3 gear    │
│               │          │ MR4 empty   │
│               │          │ MR5 empty   │
│            BC1 0.0       BR1 [-]      │  B1
│ BC2 [        Main         ▸ ]         │  B2
└──────────────────────────────────────┘
```

Hardware output, linked pair, UCX:

```text
┌──────────────────────────────────────┐
│                TC1 name              │  T1
│              TC2 balance             │  T2
│        TL3 [ M ]   [ CUE ] TR3       │  T3
│              -6.0          -6.0       │  M1
│  ML VU+labels │ MC fader │ MR1 FX return │  M2
│               │   + cap  │ MR2 empty      │
│               │          │ MR3 gear      │
│               │          │ MR4 EQ        │
│               │          │ MR5 D         │
│            BC1 0.0       BR1 [-]      │  B1
│            (no B2)                    │
└──────────────────────────────────────┘
```

#### 2.8.8 Knobs

All strip and panel knobs use the same widget and print their value in the
knob face.  Drag vertically to turn, scroll over the knob to nudge, hold
Shift for fine movement and double-click to reset.  A linked input pair's
gain knob prints the left value above the right value; it remains one
control, so dragging or scrolling changes the left value and moves the right
value by the same delta.

#### 2.8.9 Collapsed strip

The collapsed strip is a separate narrow layout: rotated name, `M`, `S` or
`CUE`, a VU element, a `+` expand button and a dark bottom display face.  It
does not use the T/M/B grid.  The face is short, spans the collapsed card
width, and uses the normal button face shape and colour; it carries no
control yet.  It puts the `+` button on the B1 line.  The VU element keeps
the normal well colour: a linked pair shows two bars, a mono or split
channel shows one centred bar covering the level text, and the labels stay
centred in the element.

#### 2.8.10 Changing the layout

The layout is defined once, in `totalcontrol-layout`.  This table names the
place to change and what follows from it:

| Change | Where | Also update |
|---|---|---|
| row heights, content box, column positions | `LayoutMetrics` in `ui/totalcontrol/totalcontrol-layout/src/lib.rs` | the example document, the invariant tests |
| font sizes, knob, fader slot and cap dimensions | `VISUAL`, same file | the example document |
| colours | `ColorScheme::default()`, same file | the example document |
| which elements a template contains | `standard_elements`, same file | the template table in [`TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md`](TOTALCONTROL-CHANNEL-TEMPLATE-SPEC.md) |
| a new element kind or placement | `ElementKind` and `ElementPlacement`, and `Grid::placement` | the schema enumerations, the template specification, the preview draw arm |
| a visual state such as hover, active or an open panel | the strip widgets, and `draw_card` in the preview | this section |
| the hand-drawn preview annotations | `totalcontrol-layout/src/bin/channel-preview.rs` | — |

After changing values:

```sh
cd ui/totalcontrol
cargo test -p totalcontrol-layout
cargo run -p totalcontrol-layout --bin channel-preview -- /tmp/layout.svg 2
cargo run -p totalcontrol-layout --bin channel-preview -- --example /tmp/gui-layout.json
python3 ../../scripts/check-docs.py
```

The second command renders the sheet for review.  The third writes an example
document plus its schema into a scratch directory; the checked-in
`docs/examples/gui-layout.json` carries the same values and keeps its
`../schema/gui-layout.schema.json` reference, because the schema lives in
`docs/schema`.  A checked-in example that differs from the compiled defaults
fails, as does a layout whose elements overlap, leave the card, leave the
content centre line, or whose tab and minimize faces no longer reach the card
edge.

The user document changes the same values at runtime for the preview.  It
does not change behaviour: the action of an element is bound to its kind in
code, not to the document.

#### 2.8.11 Layout preview

`channel-preview` renders a resolved layout to SVG: the template sheet, or one
strip with `--focus` and an element state.  It shares the geometry, the
element composition, the visual tokens and the semantic colour scheme with the
runtime, so a difference in geometry or composition is a defect.

| Reproduced | Approximated or absent |
|---|---|
| card and element rectangles, zoom scaling | font rasterisation, weights, anti-aliasing |
| element presence, placement and states | text metrics: the SVG baseline differs from the widget's box centring |
| tab and minimize faces, B2 bar | meter animation, peak hold, motion, hover timing |
| the semantic colours | slideout panel bodies: a labelled placeholder |
| each bar's meter well, colour zones and the level labels behind them | sample names, readouts and levels |

The preview reads no runtime state: names, readouts and levels are samples.
Its zoom range is `0.75..3.0`; the runtime uses `0.75..1.75`.  Approve
structure in the preview and pixels in the window.

## 3. Runtime reference

The device-service boundary, command queue, model snapshots and meter bus are
defined in [`TOTALCONTROL-ARCHITECTURE.md`](../TOTALCONTROL-ARCHITECTURE.md) §5
and §6. This specification only describes the visible result: a control shows
its local drag position immediately, and the next confirmed model snapshot
settles it.

## 4. Settings panel

The panel follows the channel type.

The layout is one vertical stack per channel type. Inputs share a nine-row
grid; a control a channel lacks leaves its row empty, so the rows below never
move and `send FX` and the phase pair sit at the same height on every input.
Empty rows keep the height of the control the grid reserves for them.

### 4.1 Input slideout grid

| Row | Inputs 1/2 | Inputs 3/4 | Inputs 5–8 | Inputs 9+ |
|---|---|---|---|---|
| 1 | `stereo` | `stereo` | `stereo` | `stereo` |
| 2 | space | reference label | reference label | space |
| 3 | `+48V` | `instr.` + reference | reference | space |
| 4 | `auto set` | `auto set` | space | space |
| 5 | gain knob | gain knob | space | space |
| 6 | `width` | `width` | `width` | `width` |
| 7 | `send FX` | `send FX` | `send FX` | `send FX` |
| 8 | `MS proc` | `MS proc` | `MS proc` | `MS proc` |
| 9 | `Ø L` `Ø R` | `Ø L` `Ø R` | `Ø L` `Ø R` | `Ø L` `Ø R` |

### 4.2 Playback and output slideout grids

| Row | Software playback | Hardware output |
|---|---|---|
| 1 | `stereo` | `stereo` |
| 2 | `width` | reference label |
| 3 | `send FX` | reference dropdown |
| 4 | `MS proc` | `return FX` |
| 5 | `Ø L` `Ø R` | `Ø L` `Ø R` |
| 6 | — | `no trim` |
| 7 | — | space |
| 8 | — | `talkback` |
| 9 | — | `loopback` |

The row heights are shared across channel types. A missing control keeps its
row so the controls below stay aligned. The Control Room output keeps its own
reference group; `talkback` is omitted on Control Room channels.

| Channel | Contents, top to bottom |
|---|---|
| Input 1/2 | Stereo, space, 48V, AutoSet, Gain, Width, Send FX, MS Proc, Ø L / R Ø |
| Input 3/4 | Stereo, `ref.`, Inst + reference dropdown, Gain, Width, Send FX, MS Proc, Ø L / R Ø |
| Inputs 5–8 | Stereo, `ref.`, reference dropdown, two empty rows, Width, Send FX, MS Proc, Ø L / R Ø |
| Inputs 9+ | Stereo, four empty rows, Width, Send FX, MS Proc, Ø L / R Ø |
| Software playback | Stereo, Width, Send FX, MS Proc, phase slot |
| Hardware output | Stereo, `ref.`, reference dropdown, Return FX, Ø L / R Ø, No Trim, space, Talkback, Loopback |

- 48 V and AutoSet are available on inputs 1/2; Inst and a gain knob on 3/4;
  a reference dropdown on 3–8; none from channel 9 on.
- The reference levels for 5–8 move together: any change to one applies to
  all four. Inputs 3 and 4 keep separate levels unless linked.
- Width is a pair control, -1.00 … 1.00: 1.00 full stereo, 0.00 mono and
  -1.00 swapped channels. It rewrites the pair's 2x2 send matrix in every
  routing that carries a live send; silent routings are untouched. Inputs and
  software playback share it.
- MS Proc is a pair button on inputs and software playback. On the UCX it
  writes `NN05` on both input channels and rewrites the same 2x2 send matrix
  with the M/S encoder (`L' = (L + R) / 2`, `R' = (L - R) / 2`); software
  playback rewrites the matrix only. A mono input pair cannot enable it.
- One control spans the panel content width; two controls share it with one
  gap, so their combined width equals a single control. `stereo`, `48V`,
  `autoset`, `no trim`, `loopback` and a phase pair therefore all end at the
  same right edge.
- Knobs keep one fixed column at the right: gain, width, send FX and return
  FX share a vertical centre line. `send`/`FX` and `return`/`FX` break over
  two lines to keep the label column narrow.
- Button labels and dropdown values are centred. Every button and dropdown
  shares one control height (24 px at scale 1).
- A standalone label such as `ref.` is centred and sits at the bottom of its
  row, directly above the control it names. It is used only where the panel
  has a reference dropdown.
- A reference dropdown reserves the handle's width, so its label clips at
  the sides and the arrow stays visible.
- Input reference values are `-10 dBV`, `+4 dBu`, `Lo Gain`; output
  reference values are `-10 dBV`, `+4 dBu`, `Hi Gain`. Reference levels
  follow register groups: the three line-output pairs share one setting and
  the Phones/Main pair has its own.
- The Talkback row is empty on Control Room channels.
- The Send FX knob mirrors the thin FX fader on the strip; the Return FX knob
  mirrors the thin fader on outputs. One value, two views.
- Every slideout fits the strip height at normal size: everything is visible
  at a glance and the panel does not scroll.
- The UCX has no PAD.

Control styles:

Buttons are only as wide as their label needs; list and menu rows are the
exception.

- `48V` uses the orange phantom outline.
- `STEREO`, `INST`, `AUTOSET`, `FX RETURN` and `LOOPBACK` use an orange
  text-and-outline engaged state.
- The Level button shows `-10dBV`, `+4dBu` or `LoGain` and cycles through the
  three values.
- Phase uses green text and a green outline when engaged.

### 3.1 Phase

Phase is available on every input channel at the bottom of the Settings
panel, and on hardware outputs.  The label is the symbol `Ø`, never the word:
a linked pair shows `Ø L` and `R Ø`, a single channel shows `Ø`.  Engaged
buttons use green text and a green outline.

### 3.2 Gain

Gain is a knob in the Settings panel for the four front inputs. Inputs 1/2
use the microphone range; inputs 3/4 use the instrument/line range, in 0.5 dB
steps on the UCX. The knob writes the per-channel preamp gain and follows the
stereo link. At connect the saved scene gain is written; a state word that
arrives before the scene is applied does not override it. On the UCX the knob
then follows physical encoder moves, which the unit reports in the IF1 state
stream.

A linked input pair always shows both values, left above right, even when
they are equal. This mirrors the device, which always treats inputs 1–4 as
unlinked. An unlinked input shows one value.

### 3.3 Device sheet

The device card at the top of the sidebar (or the gear on the folded rail)
opens the device sheet. It starts with the group the unit itself owns. Where
the unit has no readback, the saved scene value is written at connect and the
sheet shows that value. A value is written when it changes there.

| Row | UC | UCX |
|---|---|---|
| Clock source | yes | yes |
| Sample rate | yes | yes, when the device owns the rate |
| Line In reference | yes | — |
| Line Out reference | yes | — |
| Phones reference | yes | — |
| Optical | yes | yes |
| SPDIF | yes | yes |
| Single speed word clock | yes | yes |
| Word clock in | — | yes |
| EQ for Record | — | yes |
| Firmware | yes | yes |
| Serial | yes | yes |

On/off options are checkboxes after their caption. `Optical` and `SPDIF` are
dropdowns (`ADAT`/`SPDIF`, `Consumer`/`Professional`). Every caption and
control shares one left edge, and every dropdown has the same width.

The options rows belong to the unit's own group. The UC offers five clock
sources, including LTC; the UCX offers four: Internal, SPDIF coax., Optical
and Word. Host-side modes (`MS Proc`, `AN 1>2`, `Input Link`) appear in the
GLOBAL grid, apart from the device-owned rows.

On the UCX the options word, the reference levels and the 48 V/Inst/Level
switches have no readback. The saved scene value is written at connect and the
sheet shows that value. Input gain is also written from the saved scene at
connect; the Gain knob follows later encoder moves.

The UC reference rows are the unit's global options word. The UCX handles
input and output sensitivity per channel instead, in per-channel registers
(`NN08` on inputs 3/4, `0x0407`–`0x0707` for inputs 5–8, `NN08` on the output
pairs, `0x1808/0x1908` for Phones). Where one control drives a group, it
writes each channel's own register. Rows a unit has no register for are absent
rather than shown inert.

Below the device group, the host engine group shows the PipeWire rate,
quantum and latency when the sound server owns the rate. `Quantum` is the
graph quantum, one ALSA period; the `Latency` line shows the resulting ring,
USB queue and device lead. It also shows the Fireface sink's host volume in
percent and dB, with a `0 dB` action. The host volume is an attenuation before
the interface, so it lowers the playback meters by the same amount; the
interface's own faders remain the monitoring level.

The rate control offers `Auto (PipeWire)`, `Follow SW playback 1/2` and the
connected device's fixed rates: 32, 44.1, 48, 64, 88.2, 96, 128, 176.4 and
192 kHz. The engine group always shows `Graph`, the rate PipeWire is currently
running at, in every mode. In Follow mode it also shows `Mode` and `Source`;
`Source` reads `<application> · <rate>` for the stream setting the graph, and
reads `Waiting for playback` with the last graph rate when no stream feeds SW
playback 1/2. The source is always SW playback 1/2, never the focused strip.

Follow uses the stream linked to `playback_AUX0`/`playback_AUX1`, including a
player feeding a TotalControl 5.1/7.1 sink through the sink's input stream, not
the loopback output. A live owner keeps the rate while its node exists; another
client does not take it, a same-client replacement does, and after two seconds
without one the most recent remaining candidate takes over. A new rate is
forced after it has been stable for 300 ms. Unsupported rates stay visible but
are not forced. A followed rate is runtime state: it does not create an undo
step, mark the workspace dirty, or replace the fixed-rate preference.

At connect TotalControl replaces PipeWire's `clock.allowed-rates` with the
connected device's nine rates, keeping the previous value. On disconnect or a
clean exit it restores that value if the list is still the one it wrote. No
persistent PipeWire configuration file is installed.

`Errors` lists the driver transport faults since the sheet opened or the
last `reset`, one row per type: `usb` (URB status errors), `urb` (URB resubmit
errors), `gaps` (the USB queue ran dry), `short` (short packets), `recover`
(engine recoveries) and `events` (dropped driver events). `usb`, `gaps` and
`short` show input/output as `in / out`; `urb`, `recover` and `events` are
single values. A non-zero row is a fault; the section is absent when the
backend does not expose counters.

## 5. Equalizer panel

EQ and low cut are available on every hardware input and every hardware
output. The EQ trigger appears on both rows; playback strips have no EQ.

### 4.1 Response plot

- The response line is drawn in three segments: red low, green mid, blue high.
- The boundaries are `sqrt(f1*f2)` and `sqrt(f2*f3)`, where the three band
  frequencies are sorted.
- The three band frequencies are always present, including when a band is off.
- The three gain knobs carry their band colour on the arc. Frequency and Q
  knobs use the white default.

### 4.2 Bands

| Parameter | Range |
|---|---|
| Gain | ±20 dB |
| Frequency | 20 Hz..20 kHz |
| Q | 0.7..5.0 |

Band 1 and band 3 can be Bell or Shelf. Band 2 is Bell. Band 3 can also be a
Low Pass (Hi Cut).

The UI model uses Hz and Q × 10. That mapping is written to the device.

### 4.3 Low cut

The low-cut row is one centred line:

- the Low Cut on/off button on the left;
- the frequency knob in the middle;
- the 6/12/18/24 dB/oct buttons on the right.

The full word is used when the panel is wide enough; `LC` is the narrow
fallback. The dB/oct unit is in the slope tooltips. Low cut is independent of
the main EQ bypass. Frequency and slope remain editable while it is off, and
the plotted low-cut response is omitted while off.

### 4.4 Panel geometry

Inline slideouts keep at least 8 px between the panel edge and content
(`theme::FLYOUT_PAD`). This applies to Settings, EQ, Dynamics and Trim.

A slideout never shows a scrollbar at normal size. The panels are sized to the
strip height and their contents are compacted to fit. The scrollable wrapper
stays in the code so content remains reachable at extreme zoom.

A slideout has no close button; the strip tab that opened it closes it again.
When vertical space is tight, a knob label may sit over the unused bottom
margin of its knob, with the panel background behind the label.

## 6. Dynamics panel

A **D** tab opens on every hardware input and every hardware output. Playback
strips have no Dynamics.

| Row | Controls |
|---|---|
| Header | On button at the left, then the Dynamics label |
| Shared | Gain −30..+30 dB, Attack 0..200 ms, Release 100..999 ms |
| Compressor | Threshold −60..0 dB, Ratio 1..10 |
| Expander | Threshold −99..−30 dB, Ratio 1..10 |
| Auto Level | On/off, Max. Gain 0..18 dB, Headroom 3..12 dB, Rise 0.1..9.9 s |

Gain, attack and release are shared between compressor and expander.

The panel follows the new TotalMix FX layout: the amplitude graph spans the
top, the shared Gain/Attack/Release knobs sit below it, then the
Compressor/Expander columns and the Auto Level row. Compressor aligns with
Attack and Expander with Release. Threshold and ratio are grouped as one
tightly spaced block to show that they belong together; the ratio row is
labelled `ratio 1:` on one line.

The graph has input level on the horizontal axis and output level on the
vertical axis. The curve has three coloured segments:

- green below the expander threshold;
- blue between the expander and compressor thresholds;
- red above the compressor threshold.

The compressor knobs and labels use red, the expander knobs and labels use
green. The shared Gain/Attack/Release and Auto Level knobs use white.

Each knob writes the complete dynamics block for its channel.

## 7. FX panel

The FX panel is a column between the mixer rows and the right sidebar,
toggled by the `FX` knob in that sidebar and shown only on a backend with
the FX bus.  The knob lights while the column is out.
Reverb sits above Echo in one 180 px column.

The column opens and closes with the same constant-speed width tween as the
strip panels. It first reduces the mixer area until the widest row reaches its
natural width. If that still leaves no room for the column, it slides over
the rightmost part of the rows instead of forcing them to scroll further.
The overlap grows continuously as the window narrows; an open slideout keeps
its position and is covered from the right. Widening the window moves the
column back into the layout; closing FX removes it.

| Band | Height |
|---|---:|
| Header (on switch + effect name) | 36 |
| Type row | 31.5 |
| Reverb controls | 247.5 |
| Reverb type-specific band | 98 |
| Echo controls | 343 |

Knob rows are 78.75 px apart and knobs are 49.5 px across. Reverb's volume
fader occupies the right-hand column across the first two knob rows. Echo's
volume is a knob in the second column; its high-cut picker sits below it. The
Echo panel's right-hand column carries the `FX` `in`/`out` meter pair, fed by
the UCX FX bus power channels.

### 6.1 Reverb

| Control | Range |
|---|---|
| On | on/off |
| Type | Small Room, Medium Room, Large Room, Walls, Shorty, Attack, Swagger, Old School, Echoistic, 8plus9, Grand Wide, Thicker, Envelope, Gated, Space |
| PreDelay | 0..999 ms |
| Room scale | 0.50..3.00 |
| Volume | −30..+6 dB, vertical fader |
| Low cutoff | 20..500 Hz |
| High cutoff | 5..20 kHz |
| Smooth | 0..100 |
| Width | 0..100 |

The lower band holds the type-specific parameters, in the same knobs:

| Type | Controls |
|---|---|
| Envelope (12) | attack 5..400 ms, hold 5..400 ms, release 5..500 ms |
| Gated (13) | hold 5..400 ms, release 5..500 ms |
| Space (14) | reverb time 0.1..9.9 s, high damp 200..20000 Hz |

Other types leave the band empty.

### 6.2 Echo

The four knobs share one vertical centre line. The delay knob shows the
delay time inside it; the BPM readout sits at its underside/right without
moving the knob. `HC` sits left of a compact high-cut box; the box is only
as wide as its value, and the value text is centred. The FX `in`/`out` meters
sit to the right of the knob column, wider than a strip meter, and show an
overload segment at the top when the peak reaches full scale.

| Control | Range |
|---|---|
| On | on/off |
| Type | Stereo Echo, Stereo Cross, Pong Echo |
| Delay time | 10..2000 ms; second readout 30..300 BPM |
| Feedback | 0..100 % |
| High cutoff | Off, 16k, 12k, 8k, 4k, 2k |
| Width | 0..100 |
| Volume | −30..+6 dB knob |

BPM is derived from the delay: it reads `60000 / delay` and writes the delay
register, `60000 / bpm`.

### 6.3 FX Return

Each hardware output has an **FX Return** knob in its Settings panel,
−65..0 dB. A linked pair writes both channels.

## 8. Output balance

Hardware output strips include a balance knob, −100..+100. The linked fader
is the louder channel (the pair level). On the UCX the knob writes the output
balance register, independent of the pair volume. Splitting converts the pair
level and balance into the two channel volumes; linking recomputes both from
those volumes. A split/re-link round trip therefore preserves the pair level
and balance. A split pair keeps two independent volumes and hides the balance
knob.

## 9. Pan and balance

A split input or software-playback strip exposes pan into the selected
submix; the knob writes the source's matrix cells for that submix. On a
linked pair the same knob is a balance derived from the two channel volumes,
and writing it changes the quieter channel's volume instead of a pan.

### 8.1 Signal model and submixes

Each hardware output is a bus. The selected output is the current submix;
every input and software-playback strip carries an independent send gain and
pan to that bus. The input/playback fader is the send level to the current
submix, not a single channel fader. The output strip's fader is the bus
master.

A split output pair has two independent mono buses. Selecting one channel
selects only that channel's bus: only its strip takes the active surface, and
the input/playback send faders read and write that mono matrix column. A
linked output pair keeps the pair-level submix and stereo send model.

| Term | Normal mixing desk | TotalMix FX | TotalControl |
|---|---|---|---|
| Input gain / trim | Preamp trim before the channel fader and meter | Hardware preamp gain; the T button is Trim Gain mode, not a separate gain stage | Trim pre-gain, common to every send of the channel |
| Channel fader | One per channel, feeding the bus assignment | None | None |
| Send fader | Aux send, usually pre- or post-channel fader | Per-routing send gain; the fader edits the selected output | Per-submix send gain; the fader edits the selected output |
| Bus summing | Group, aux or matrix bus | Submix for one hardware output | Submix for one hardware output |
| Bus master | Group or aux master fader | Output strip fader | Output strip fader |
| Bus to output | Patch or assignment; one bus can feed several outputs | One bus per hardware output | One bus per hardware output |
| Input VU tap | After input gain/trim, before the channel fader | After preamp gain and optional Post FX, before the send fader; T does not change it | After trim, before the send fader |
| Output VU tap | Post-fader bus or output | Post-fader output bus | Post-fader output bus |

In TotalMix, T is the **Trim Gain** mode: it gangs the sends of one channel.
It is not a pre-gain and does not move the input VU. TotalMix therefore has
an input VU after the preamp and optional FX, not “before trim”.

In TotalControl, T is a true pre-fader trim. The send fader displays the post-trim
level, so its maximum is +6 dB plus the trim value at the +6 dB send limit. An
analog desk changes the meter and signal with trim but leaves the channel fader
in place; TotalControl moves the send fader so the post-trim level is visible.

For an analog input, the signal paths are:

```text
Normal desk:
input -> input stage / sensitivity -> preamp / trim -> channel VU
      -> channel fader -> pan -> bus assignment -> bus sum
      -> bus master -> output VU -> output

TotalMix FX:
input -> input stage / reference -> preamp gain (where present) -> ADC
      -> input VU (post-preamp; Post FX optional)
      -> send fader for the selected output -> bus sum
      -> output fader -> output VU -> DAC

TotalControl:
input -> input stage / reference -> preamp gain (where present) -> ADC
      -> trim -> input VU -> send fader for the selected submix
      -> bus sum -> output fader -> output VU -> DAC
```

The preamp gain is present on the UCX analog inputs 1–4. Digital inputs enter
after the preamp stage. In TotalMix, T is a send-fader mode, not a gain stage.
In TotalControl, trim is a host-side pre-fader gain after the ADC.

Control Room destinations (Main, Speaker B, Phones) are ordinary output buses
with their assignment controls. Cue/PFL to selects which monitoring output
carries the Cue signal; it is not a channel.

## 10. Motion

All strip panels open and close with a width tween at a constant 1 px/ms;
a wider panel takes longer instead of sliding faster:

- Settings, EQ, Dynamics and Trim grow and shrink beside their strip;
- the FX column grows and shrinks beside the mixer, or over its right edge
  when the window is too narrow for both;
- Route grows and shrinks as a raised picker;
- Quick view uses the same tween;
- switching Settings, EQ, Dynamics or Trim on one strip tweens directly from
  the current width to the new width.

Toggling Stereo tweens the second half of the pair between zero and full width
at the same speed. Channels to the right reflow with the tween.

Assignment add, clear and replace move at the same 1 px/ms speed. A
destination change that keeps the Control Room channel count unchanged does
not animate.

## 11. Stereo link

Linking an input pair uses the left channel as the reference for mute, solo,
phantom, Inst, Level, reference level and AutoSet. It also copies the left
channel's FX send, low cut, EQ, dynamics and AutoLevel settings to the
right. The linked gain knob moves both preamps by the same step and preserves
their offset; the knob shows both values. Phase remains per-channel.

Linking does not copy or flatten the two channel volumes: the linked fader
displays the louder channel, and moving it shifts both channels by the same
dB offset so their difference is preserved.

The linked balance is derived from the two channel volumes:

```text
balance = round((right - left) / max(left, right) * 100)
```

The knob sets the quieter side's volume and leaves the louder side at the
pair level. Splitting sets the two pans hard left and hard right; split pan
changes do not alter the balance because the balance lives in the channel
volumes. The same rule applies to software-playback and hardware-output
pairs.

Software-playback pairs link as a host grouping. Mute and solo copy from left
to right; FX send remains per-channel and MS processing stays a pair control.

Hardware-output pairs link with the output stereo words. Mute, FX return,
low cut, EQ, dynamics and AutoLevel copy from left to right. The two channel
volumes encode the pair level and balance. The output reference level is a
line-output group value, not a per-pair link value. Phase L and Phase R keep
independent states across every transition. A pair assigned to the Control
Room is always linked and never splits.

## 12. Control Room

The Control Room panel is flush with the mixer's right edge beside the FX
column. Its button stack is one channel wide; every button in the stack has
the same width and height, and its label is centred. Long labels use the next
smaller type tier. The stack runs, top to bottom:

| Group | Controls |
|---|---|
| dim / recall / speaker B | Dim, Recall, Speaker B |
| mono / talkback / ext. input / mute FX | Mono, Talkback, Ext. input, Mute FX |
| bottom | Settings, immediately above Assign |

Engaged monitoring buttons use the dark face, orange text and 1 px orange
outline. Recall has no latched state and uses the normal press feedback;
Settings stays lit while its dialog is open. `assign` is a selector, not a
monitoring button: it uses the same compact row and white drop decoration as a
strip's output-bus trigger, and its row stays lit while the menu is open.

Recall acts on the assigned Main Out pair. Dim attenuates the assigned Main
Out and Speaker B pairs; the Phones outputs are not affected. Without an
assigned Main Out both controls leave the mix unchanged and never fall back
to a Phones pair.

Dim saves the current Main Out and Speaker B levels, lowers both speaker
pairs by the configured Dim amount and engages the device Dim state. Turning
Dim off restores the saved levels. A speaker fader move while Dim is engaged
changes the audible level but not the saved pre-Dim levels; turning Dim off
returns to the saved levels. Dim attenuates both speaker pairs, Main Out and
Speaker B, and does not touch the Phones outputs. Changing the Main Dim
amount while Dim is engaged re-applies the new amount to the saved levels.
Reassigning or clearing the Main Out or Speaker B releases Dim first.

Recall sets the underlying Main Out level to the configured listening volume
once; it does not latch. When Dim is engaged, Recall updates the saved pre-Dim
levels and the current dimmed levels together.

Dim and Speaker B are part of the current mix, snapshot and workspace. A
scene captured while Dim is active stores the pre-Dim speaker levels and the
Dim amount; applying it restores those levels and engages Dim once. Speaker B
keeps its saved monitor path and is applied after the saved assignments.
Speaker B requires assigned Main and Speaker B outputs; its button switches
the monitor routing between those pairs, and clearing the Speaker B
assignment also clears Speaker B. The `Link to Main` setting makes Speaker B
follow Main fader and Recall changes; Dim attenuates both speaker pairs
whether linked or not, while the Phones
outputs are unaffected.

Mono is a latching monitor check: it mixes the assigned Main Out and Speaker
B pairs to mono and feeds the same signal to both sides of each pair. It does
not mix the Phones outputs. Mono is stored in the device scene.

Mute FX is a latching scene control. It saves the FX-return levels of the
assigned Main Out and Speaker B pairs, writes `-65 dB`, and restores the
exact saved levels; it does not switch the Reverb or Echo processors off.
Phones is not modified.

Ext. input is a latching runtime control. It replaces the Main Out and
Speaker B submixes with the stereo hardware-input pair selected in Control
Room Settings at the configured gain, mutes those pairs' FX returns, and
sets monitor flag bit 5. Releasing restores the normal submix and the saved
FX returns. The button is disabled until a source pair is configured.
Changing the source or gain while active re-engages the path.

With `PFL replaces Solo`, each input strip's Solo button is labelled `PFL`
and routes the input pre-fader at unity to the Cue/PFL target, with the
target's other sources and FX returns suppressed. The global Solo master
shows `F` while a PFL is active and `S` otherwise; it turns active PFL inputs
off and can restore them. `Exclusive Solo/PFL` keeps one input pair active.

Control Room channels are always stereo. An assigned pair is linked and
cannot be unlinked, so the Settings panel's `STEREO` control stays engaged
and does nothing; a split pair is linked when it is assigned. `Set all
channels mono` leaves the assigned pairs linked.

### 11.1 Assign

`assign` opens a two-level menu. The first level lists the monitoring
destinations in this order, using the same names as the Control Room channels:

| Destination | Device |
|---|---|
| Main | output mode word `0xNN02` bit 1 on the chosen pair |
| Speaker B | the same bit, plus control-room state word `0x3d00 = 1` while active |
| Phones 1..4 | host-side; the unit receives nothing |
| Cue/PFL to | host-side; selects the Cue/PFL monitoring target |

The Cue button on a hardware output copies that output's submix into the
selected Cue/PFL target, replacing the target's sources. It is exclusive
across outputs. Releasing Cue, moving it to another output, or changing the
target restores the previous target submix and FX returns.

The second level for an output destination lists the predetermined hardware
channel names—`None` first, then every hardware output in display order, the
current output in bold. It never shows user names. Choosing `None` clears the
destination. Both levels use the same button width, and every label is
centred.

Clicking a first-level destination opens and pins its second level. Hovering
an unpinned first-level row opens or switches the second level without
pinning it. The second level closes only when an item is chosen, a click lands
outside the menu, or `assign` is clicked.

`Cue/PFL to` is the seventh entry and stays in the Assign list. Its sublist is
`Main`, `Phones 1`, `Phones 2`, `Phones 3`; it has no `None`; the current
target is bold. Choosing a target changes only where Cue/PFL is monitored; the
same target controls PFL monitoring. It assigns no hardware output, moves no
channel and creates no Control Room slot. The target is host-side and stored
with the GUI preferences.

The menu opens immediately left of the bottom `assign` button and covers the
section's channels while it is open.

Control Room slots run left to right: `Phones 1`, `Phones 2`, `Phones 3`,
`Phones 4`, `Speaker B`, `Main`. `Main` is beside the buttons. There are no
empty slots: every hardware output is either assigned in the Control Room or
unassigned in the Hardware Outputs row.

Control Room channel names are predetermined and are exactly the first six
Assign names: `Main`, `Speaker B`, `Phones 1`–`Phones 4`. View → Names does
not change them. They are not user editable; clicking the name selects the
channel but never opens the rename field. The Assign menu uses those names in
its first level and the predetermined hardware names in its second level.

Assigning a pair moves its channel out of the Hardware Outputs row into the
Control Room section, named for its destination, and the panel widens to hold
it. Clearing the destination returns the channel to the Hardware Outputs row
in its original position. Changing an output from one destination to another
keeps the fixed slot order: when the Control Room count does not change, the
strip takes its new destination slot and its name changes without animation.

The section carries its own `control room` title in the same line as the
HARDWARE OUTPUTS label. The title starts at the section's leftmost channel,
and a gap separates the two titles when the row has room to spare for it.

A Control Room channel's slideout opens to its right like any other. The
section shifts its own channels left by one slot to make the space, and its
left edge may overlap the channel row, which never moves.

Control Room channels are ordinary hardware outputs for routing too: clicking
one selects that output's submix and it takes the same brighter surface as any
other output does, which is how the manual describes every hardware output
("click on the Hardware Output channel ... This channel turns brighter, means
it is selected as current submix"). The Main output is a submix like any
other; FX Send can follow it.

Only the current submix takes the brighter card. Selecting a channel for
editing — with the mouse, the unit's front panel or a remote — draws no border
or tint on the strip; the front panel and the remote show their own selection.
The selected channel is the target for cursor-key nudges and typed values, and
it never joins or clears the global fader group.

Hovering a strip draws its 1 px silver outline and soft glow, so the click
target is clear before the click. The outline follows the pointer and is not a
persistent selection. The meaningful multi-channel selection is the global
fader group, shown by its members' cap colour.

Every click on a hardware output counts, not only the strip's empty surface:
its mute and cue switches, its slideout buttons (gear, EQ, D), its fader and
its name field select that output the same way, and the visible submix
switches to it. Clicking any of an output strip's controls focuses that
output and shows its submix; the global fader group is left untouched.

### 11.2 Assignment motion

An assignment change is one combined flow. Every affected strip, slot and row
gap starts in the same frame and moves at the same constant 1 px/ms as the
other panel transitions. Parts with different travel distances finish at
different times; there is no separate Control Room timing.

Adding an assignment slides only within each row; the channel does not
travel between the Hardware Outputs row and the Control Room.

- The output strip leaves the Hardware Outputs row, and the strips to its
  right slide left by the vacated width.
- The Control Room grows by one slot; the new slot takes its place in the
  fixed order; the slots to its left slide left; the slots to its right stay.
- The channel appears in its new Control Room slot.

Clearing an assignment reverses the row-internal motion: the Control Room
slots to the left of the removed slot slide right, the panel shrinks, and the
strips to the right of the channel's original row position slide right to open
the gap. The returning channel fills that gap from its right edge as the gap
opens; it is revealed progressively rather than appearing at full width.

Replacing an occupied destination, or changing an output from one destination
to another, keeps the Control Room channel count unchanged and therefore does
not animate. The fixed slot order is recomputed immediately; the channel and
its name appear in the new destination slot.

A split output pair moves as one dual channel. If a channel has an open
slideout, the slideout travels with it and stays open. The Hardware Outputs
row makes room for the strip plus the slideout; when there is not enough room,
the rightmost channels pass under the Control Room, which always remains
visible. At the destination the usual slideout rule applies: it opens to the
right, and the Control Room shifts its own channels left by one slot.

The moving channel is drawn once, above the rows and panel, and accepts no
pointer input until it settles. No motion runs when assignments are restored
from preferences at startup.

The Main and Speaker B assignments are application configuration stored with
the GUI preferences. A scene load or reapply keeps the live assignment; a
scene's stored assignment does not move the Control Room destinations.

The front-panel volume knob follows the output selected on the unit's display,
not the Control Room Main assignment; `PH` always addresses channels 7/8.

### 11.3 Control Room Settings

`settings` opens a modal dialog titled `CONTROL ROOM SETTINGS`. It edits a
draft; the stored values change only when `OK` is pressed. `Cancel`, `Escape`
and a click outside the dialog discard the draft.

| Section | Controls |
|---|---|
| Talkback | Input, Dim |
| Listenback | Input, Dim |
| Main Out | Recall, Dim, External Input, Gain, Speaker B link |
| Solo / PFL Mode | Live Mode, PFL replaces Solo; Exclusive Solo/PFL Mode |

The input lists start with `None`, then the mono hardware inputs by their
device names. External Input lists stereo hardware-input pairs. The five
level controls have a `-65.0 … 0.0 dB` range, a `0.5 dB` step and a
one-decimal value display.

`OK` stores the settings in `gui.json` for the selected device. The settings
are not part of the mixer scene, snapshot, undo stack or OSC scene state.
`Dim`, `Speaker B`, `Mono`, `Mute FX`, `Cue` and the sample rate are mixer
scene and workspace state. Active Talkback is momentary and starts off when a
workspace opens; the per-output Talkback receive flags live in the mixer
scene. External Input and the PFL listening set are workspace state. The
unit's six setups and device serial number remain outside the workspace.
External Input source/gain and the Solo/PFL modes are stored as settings and
applied to the running device on `OK`.

## 13. Latency display

The Settings panel shows the application/ALSA latency budget and its terms:
the host ring, USB queue, device lead and converter. The buffer-size picker
sets the graph quantum. The formulas and the device-side model are in
[`TOTALCONTROL-ARCHITECTURE.md`](../TOTALCONTROL-ARCHITECTURE.md) §10.

The displayed line is an estimate, not a measured acoustic round trip. The
panel labels it accordingly and keeps the four terms visible.

## 14. State references

Mixer-state contents, workspace storage, snapshot storage, startup mismatch,
undo and the split between mixer state and GUI preferences are defined in
[`TOTALCONTROL-ARCHITECTURE.md`](../TOTALCONTROL-ARCHITECTURE.md) §9. This
specification only names the visible surfaces: eight snapshot slots, six
layout slots, the Workspace menu, the current auto-saved mix and the GUI
preference file.

The serialized mixer-state type is called `Scene` in code.  The interface
never uses that word: it says **current mix** for the live state, **snapshot**
for a saved mix inside a workspace, **workspace** for the complete `.tcws`
configuration, and **device setup** for the unit’s six slots.

The startup mismatch dialog lists the decoded differences as a bullet list.
On the UCX it appears after the asynchronous pre-connect dump has been
compared; the window itself is shown first. More than six differences are
shown in a fixed-height scrollable list box instead of letting the dialog grow
beyond the window.

## 15. Control invariants

- Every channel control has a defined effect in the mixer model and its device
  path.
- Every button centres its label horizontally and vertically in the button
  face; an icon-only button centres the icon. The three masters and the group
  storage cells use the normal sidebar button height. Menu items are the one
  exception: they keep their text left-aligned.
- Engaged controls use their defined style: orange outline for Stereo, Inst,
  AutoSet, FX Return and Loopback; green outline for Phase; orange arc for
  gain, pan and output balance; solo colour for Dynamics.
- Settings, EQ, Dynamics, Trim and Route panels use the same height as their
  strip.
- Panel and row transitions use the constant-speed width tween.
- The right sidebar remains visible while expanded, at every window width.
- Menu items and dialog body text use `theme::TEXT_DIALOG` (16 px at scale 1.0).
- The app settings sheet is centered over a dimmed scrim.
- The three channel rows scroll together under one horizontal slider.
- Phase, pan, output balance, dynamics, FX and loopback survive scene recall.
- Output balance survives output-fader movement.
- Trim is a pre-fader gain that attenuates every send of a channel. It adds
  no device register; the value is applied when each send is written. `T` is
  the first icon in the strip's column on hardware inputs and software
  playbacks; it lights while trim is below 0 dB, and its tooltip states the
  applied value.
- The fader displays the post-trim level: `displayed = send × trim`. With
  the send limited to +6 dB, the fader's maximum is +6 dB plus the trim
  value, and moving trim moves the fader cap with it. The send written to
  the device is `displayed / trim`, so the audible level remains the fader
  value.
- Input and playback meters are post-trim: a constant input tone falls and
  rises with the trim value. Output meters remain post-fader.
- Width is host-side and has no width register: it rewrites the pair's
  matrix cells. Playback stereo/mono, No Trim and Talkback receive are
  host-side and do not produce device writes. Mono writes the monitor flags
  `0x3f9e` and `0x3f98`.
