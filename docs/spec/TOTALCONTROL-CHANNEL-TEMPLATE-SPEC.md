# Channel template schema

This specification defines the shared channel-template model used by the
GUI and the layout preview.  It is the developer-facing form of
`TOTALCONTROL-GUI-SPEC.md` §2.8.

## Layers

| Layer | Purpose |
|---|---|
| `Features` | Which optional controls exist: FX, Trim, EQ, Dynamics |
| `TemplateSpec` | A named standard channel: kind, features, mono/stereo |
| `ChannelElement` | One element: kind, placement and the binding its kind implies |
| `ChannelTemplate` | The elements of one template and where they sit |
| `StripLayout` | Logical rectangles resolved for one template and one zoom |
| Preview | Renders the resolved elements and the semantic colour scheme |
| Runtime | Selects a standard channel and fills its element bindings |

The specification remains the design authority.  `totalcontrol-layout` is
the executable form of the geometry and the template list.

## Standard templates

| Template | Kind | Stereo | FX | Trim | EQ | Dynamics |
|---|---|---|---|---|---|---|
| Input mono | input | no | yes | yes | yes | yes |
| Input stereo | input | yes | yes | yes | yes | yes |
| Input mono, no FX | input | no | no | yes | no | no |
| Playback stereo | playback | yes | yes | no | no | no |
| Output stereo | output | yes | yes | no | yes | yes |
| Output mono | output | no | yes | no | yes | yes |
| Control Room | output | yes | yes | no | yes | yes |

A template owns the presence of its optional elements.  The runtime must not
independently decide that a channel has a different set of elements.

## Elements and placement

A channel template is a list of channel elements.  An element is a kind and a
placement; the kind fixes the binding.  `totalcontrol-layout` resolves the
list to one `StripLayout`, so the preview and the runtime render the same
composition.

### Placements

Each placement is one position of the strip grid, using the references of
`TOTALCONTROL-GUI-SPEC.md` §2.8.

| Placement | Reference | Position |
|---|---|---|
| `Name` | T1 | name and type tag |
| `Pan` | T2 | pan or balance knob |
| `ButtonLeft` | TL3 | left button |
| `ButtonRight` | TR3 | right button |
| `ReadoutLeft` | M1L | left channel readout |
| `ReadoutRight` | M1R | right channel readout |
| `Meter` | ML | VU element |
| `Fader` | MC | channel fader |
| `Fx` | MR1 | FX send or return fader |
| `Stack(Trim)` | MR2 | Trim tab |
| `Stack(Settings)` | MR3 | settings gear |
| `Stack(Eq)` | MR4 | EQ tab |
| `Stack(Dynamics)` | MR5 | Dynamics tab |
| `Level` | B1 | dB readout |
| `Minimize` | B1R | collapse face |
| `Submix` | B2 | submix bar |

The five right-hand rows keep their positions even when a template places no
element in them, so the rows below a missing tab do not move.

### Kinds and bindings

| Kind | Binding | Drawn as |
|---|---|---|
| `Name` | channel name | name text |
| `Pan` | pan or balance value | knob |
| `Button(Mute)` | mute | `M` |
| `Button(Solo)` | solo | `S` |
| `Button(Cue)` | cue | `CUE` |
| `Readout(Left)` | left channel strength | text readout |
| `Readout(Right)` | right channel strength | text readout |
| `Meter` | meter level | VU bars and ruler |
| `Fader` | send or output level | fader slot, fill and cap |
| `FxFader` | FX send or return level | thin fader |
| `Tab(panel)` | the panel it opens | `T`, gear, `EQ`, `D` |
| `Level` | output level | dB text |
| `Minimize` | collapse | `−` |
| `Submix` | submix assignment | bottom bar |

### Standard composition

Every template places the name, the pan knob, both buttons, both readouts, the
meter, the fader, the level readout and the minimize face.  Inputs and
playbacks add the submix bar.  The FX fader follows the FX feature.  The four
tab rows follow Trim, EQ and Dynamics; the settings gear is always present.
An absent tab leaves its row empty.

### Resolution

Elements resolve in template order.  `StripLayout` exposes the rectangle and
the right-edge face per placement, the element per kind, and the four tab rows
whether or not a template occupies them.  A placement the channel kind cannot
hold, such as a submix bar on an output, is an error.

## Geometry

All constants are at zoom 1.0.  The resolver multiplies every value by the
zoom exactly once.

| Row | Height |
|---|---:|
| top padding | 4 |
| name | 13 |
| gap | 1 |
| pan | 54 |
| gap | 1 |
| M/S or M/CUE buttons | 18 |
| gap | 1 |
| M1 readouts | 18 |
| gap | 1 |
| M2: VU, fader and right-hand stack | 205 |
| gap | 1 |
| B1: dB readout and minimize | 14 |
| gap | 1 |
| B2: submix selector | 18 |
| **total** | **350** |

The card is 90 px wide.  The content box is 78 px wide, inset 6 px from
each side.  The VU is the left 13 px.  The fader track is 26 px wide and
centres on the content line.  The four tab faces reach the card's right
edge.

## Invariants

The schema tests enforce:

- all rows sum to 350 px;
- the VU and fader share top, bottom and height;
- the fader track centres on the content line;
- top rows and B1 share the same centre line;
- tab and minimize faces reach the card's right edge;
- B2 spans the full card and is flush with the bottom;
- visible elements do not overlap;
- every element stays inside the card;
- zoom scales every rectangle;
- an output has no B2.

## Changing the layout

### Developer workflow

The change workflow, the file for each kind of value, and the checks that
follow are in
[`TOTALCONTROL-GUI-SPEC.md` §2.8.10](TOTALCONTROL-GUI-SPEC.md#2810-changing-the-layout).

The preview renders the sheet:

```sh
cd ui/totalcontrol
cargo run -p totalcontrol-layout --bin channel-preview -- preview.svg 2
```

The second argument is the zoom.  The SVG can be converted to PNG with
`magick preview.svg preview.png`.

## Layout configuration

`totalcontrol-layout` reads a complete layout document in JSON.  The document
is the configuration form of this specification: one geometry, one visual
token set, one colour scheme and a list of channel templates.

The document describes the whole sheet: one geometry, one visual token set,
one colour scheme and the templates.  Each template declares the channel
elements it contains and where each one sits; the bindings follow from the
kinds (see [Elements and placement](#elements-and-placement)).

The preview is the only consumer.  The runtime still renders the compiled
defaults.

### Document

| Key | Type | Meaning |
|---|---|---|
| `$schema` | string | Optional path or URI of the schema file.  Ignored when rendering. |
| `schema` | integer | Document version.  Must be `2`. |
| `geometry` | object | Strip geometry at zoom 1.0. |
| `visual` | object | Font sizes and control dimensions at zoom 1.0. |
| `colors` | object | Semantic colour scheme. |
| `templates` | array | Channel templates, in sheet order. |

Every key is optional.  A missing section, a missing field or an empty
document (`{}`) takes the built-in value, and a partial section overrides only
the fields it names:

```json
{ "colors": { "accent": { "solo": "#ffcc00" } } }
```

`templates` is the exception: the list is replaced, not merged, and each entry
is a complete template object.

### Geometry

Pixels at zoom 1.0.  The resolver multiplies each value by the zoom exactly
once.  Widths and heights must be greater than zero; offsets may be zero.

`Default` is the shipped value.  The live values are `LayoutMetrics` in
`ui/totalcontrol/totalcontrol-layout/src/lib.rs`, and the schema carries the
same defaults for editors; a test keeps the two in step.

**Card and content**

| Field | Default | Meaning |
|---|---:|---|
| `strip_w` | 90 | Card width.  Tab and minimize faces derive their width from it. |
| `strip_h` | 350 | Card height.  Rows are stacked from `content_y`, so it must hold every row. |
| `content_x` | 6 | Left inset of the content box. |
| `content_w` | 78 | Content box width.  In the shipped layout it is `strip_w - 2 × content_x`. |
| `content_y` | 4 | Top inset of the content box.  There is no bottom inset. |
| `row_gap` | 1 | Gap between stacked rows. |

**Top area**

| Field | Default | Meaning |
|---|---:|---|
| `name_h` | 13 | Channel name row height.  The name spans the content box. |
| `pan_x` | 12 | Left edge of the pan knob area from the content box. |
| `pan_w` | 54 | Pan knob area width. |
| `pan_h` | 54 | Pan knob area height. |
| `button_w` | 38 | Width of one mute/solo button.  Both readouts use it. |
| `button_gap` | 2 | Gap between the two buttons and between the two readouts. |
| `button_h` | 18 | Mute/solo button height. |
| `readout_h` | 18 | M1 readout row height. |

**Middle area**

| Field | Default | Meaning |
|---|---:|---|
| `middle_h` | 205 | Middle row height, shared by the VU, the channel fader canvas and the FX fader. |
| `vu_x` | 0 | Left edge of the VU from the content box. |
| `vu_w` | 13 | VU width. |
| `fader_canvas_w` | 55 | Width of the canvas that holds the graduations, cap and track. |
| `track_x` | 26 | Left edge of the fader track from the content box. |
| `track_w` | 26 | Fader track width. |
| `fx_w` | 16.12 | FX fader width, centred in the tab column. |
| `fx_h` | 66 | FX fader height from the middle-area top. |
| `tab_x` | 56 | Left edge of the tab column from the content box. |
| `tab_w` | 20 | Tab glyph width. |
| `tab_top_with_fx` | 74 | Distance from the middle-area top to the first tab when the FX fader exists. |
| `tab_h` | 22 | Trim, EQ and Dynamics tab height. |
| `gear_h` | 26 | Settings gear tab height. |
| `tab_gap` | 6 | Gap between tab faces. |

**Bottom area**

| Field | Default | Meaning |
|---|---:|---|
| `db_x` | 20 | Left edge of the dB readout from the content box. |
| `db_w` | 38 | dB readout width. |
| `db_readout_h` | 14 | B1 row height, shared with the minimize face. |
| `collapse_x` | 59 | Left edge of the minimize glyph from the content box. |
| `collapse_w` | 19 | Minimize glyph width. |
| `submix_h` | 18 | B2 submix bar height. |

### Visual tokens

Pixels at zoom 1.0; every value must be greater than zero.

`Default` is the shipped value; the live values are `VISUAL` in
`ui/totalcontrol/totalcontrol-layout/src/lib.rs`.

| Field | Default | Meaning |
|---|---:|---|
| `name_size` | 13 | Channel name font size. |
| `button_text_size` | 11 | Mute/solo and cue label size. |
| `readout_size` | 10 | M1 readout, dB readout and submix label size. |
| `micro_size` | 8 | VU ruler label size. |
| `tab_glyph_size` | 13 | Trim, EQ and Dynamics glyph size. |
| `gear_glyph_size` | 14 | Settings gear glyph size. |
| `knob_diameter` | 42 | Pan knob diameter. |
| `knob_label_size` | 15 | Pan knob value label size. |
| `fader_slot_w` | 4.34 | Fader slot width, shared by the channel and FX faders. |
| `fader_cap_w` | 22 | Channel fader cap width. |
| `fader_cap_h` | 15 | Channel fader cap height. |
| `fx_cap_w` | 13.64 | FX fader cap width. |
| `fx_cap_h` | 9.3 | FX fader cap height. |

### Colours

Every value is a `#rrggbb` literal.  The five categories and their tokens are
listed under [Colour categories](#colour-categories).

| Category | Tokens |
|---|---|
| `surface` | `app_background`, `card`, `card_active`, `border` |
| `text` | `primary`, `secondary` |
| `control` | `button_face`, `knob_face`, `knob_border`, `fader_slot`, `fader_fill`, `cap_ridge`, `cap_centre`, `tab_face` |
| `accent` | `mute`, `solo`, `cue` |
| `meter` | `green`, `yellow`, `red` |

### Templates

Each entry is a complete template:

| Field | Type | Meaning |
|---|---|---|
| `id` | one of the standard templates | Template identity; must be unique in the list. |
| `kind` | `Input`, `Playback` or `Output` | Channel row.  Inputs and playbacks have the B2 submix bar. |
| `stereo` | boolean | Linked pair: the VU draws two bars and the name uses the pair form. |
| `elements` | array | The channel elements, each `{ "kind": ..., "at": ... }`. |

An element names what it is and where it sits:

```json
{ "kind": "eq_tab", "at": "MR4" }
```

`kind` is one of the kinds in [Kinds and bindings](#kinds-and-bindings);
`at` is one of the placements in [Placements](#placements).  The kind must be
the one that belongs at that placement; `TR3` accepts `solo` and `cue`.

The list must contain at least one element, and a placement appears at most
once.  An omitted element leaves its row empty, and the rows below keep their
position; the optional controls of a template therefore follow from its
element list, not from a feature block.

### Derived placement

The resolver derives these placements from the values above.  They are not
configurable:

| Element | Rule |
|---|---|
| content box | `content_x`, `content_y`, `content_w`, height `strip_h - content_y` |
| top rows | stacked from `content_y` in the order name, pan, buttons, readouts, each followed by `row_gap` |
| middle row | starts below the readouts; `middle_h` tall |
| tabs | start at the middle-area top plus `tab_top_with_fx` when the FX fader exists, otherwise at the middle-area top; stacked with `tab_gap` |
| tab and minimize faces | start at their glyph and reach the card's right edge |
| B2 | full `strip_w`, flush with the bottom edge |
| zoom | every rectangle is scaled once |

Five placements follow the content box rather than the card:

| Relationship | Constraint |
|---|---|
| pan knob | `pan_x = (content_w - pan_w) / 2` |
| dB readout | `db_x = (content_w - db_w) / 2` |
| fader track | `track_x = (content_w - track_w) / 2` |
| button pair | `content_w = 2 × button_w + button_gap` |
| readout pair | shares the button width and gap, so it starts at the content box edge |

### Validation

The preview rejects a document and renders nothing when:

- `schema` is not `1`;
- a width or height is zero or negative, or any number is not finite;
- an offset is negative;
- a colour is not a `#rrggbb` literal;
- the template list is empty or repeats an `id`;
- a template has no elements, repeats a placement, or pairs a kind with the
  wrong placement;
- an element kind or placement token is not in the lists above;
- a template places an element outside the card;
- the fader track, the pan knob, the button pair, the readout pair or the dB
  readout leaves the content centre line;
- an unknown field appears at any level.

The error names the failing field, for example
`colors.accent.mute must be a #rrggbb value`.

### Previewing a document

A candidate file is rendered without rebuilding:

```sh
cd ui/totalcontrol
cargo run -p totalcontrol-layout --bin channel-preview -- \
    --config ~/.config/totalcontrol/gui-layout.json \
    /tmp/my-layout.svg 2
```

The second positional argument is the zoom, clamped to `0.75..3.0`.
`--focus <template>` renders one strip instead of the sheet, and `--state`
selects its element state: `rest`, `hover`, `open` or `active`.

`--example` writes a complete starting document and the matching schema
beside it:

```sh
cargo run -p totalcontrol-layout --bin channel-preview -- \
    --example /tmp/gui-layout.json
```

The checked-in files are
[`../examples/gui-layout.json`](../examples/gui-layout.json) and
[`../schema/gui-layout.schema.json`](../schema/gui-layout.schema.json).

The intended runtime location is
`$XDG_CONFIG_HOME/totalcontrol/gui-layout.json`, normally
`~/.config/totalcontrol/gui-layout.json`.

### Editor support

`docs/schema/gui-layout.schema.json` is a JSON Schema (draft 2020-12) for the
document.  It carries every field, its type, its default and its range or
enumeration, so an editor validates and completes the file as it is typed.
The schema is embedded in `totalcontrol-layout` and written next to the
`--example` document; a `$schema` value pointing at that file is enough for
editors that read the key.

A crate test compares the schema with the compiled model: property names,
defaults, the required feature list and the template identities must match, so
the reference cannot drift from the code.

### Common changes

Widen the card; the content box stays where it is and the tab and minimize
faces take the extra width:

```json
{ "geometry": { "strip_w": 96 } }
```

Widen the content box and move everything that centres on it:

```json
{
  "geometry": {
    "strip_w": 96,
    "content_w": 84,
    "button_w": 41,
    "pan_x": 15,
    "db_x": 23,
    "track_x": 29
  }
}
```

The relationships in [Derived placement](#derived-placement) fix those values:
`content_w` is 84, so the button pair needs `2 × 41 + 2`, and the knob, dB
readout and track need `(84 - width) / 2`.

Give the level area more travel and the card the extra height:

```json
{ "geometry": { "middle_h": 215, "strip_h": 360 } }
```

Recolour the accents:

```json
{ "colors": { "accent": { "mute": "#4f8ff7", "solo": "#ffcc00" } } }
```

Enlarge the channel fader cap:

```json
{ "visual": { "fader_cap_w": 24, "fader_cap_h": 16 } }
```

Drop the FX fader from a template by leaving its element out:

```json
{
  "templates": [
    {
      "id": "InputMono",
      "kind": "Input",
      "stereo": false,
      "elements": [
        { "kind": "name", "at": "T1" },
        { "kind": "pan", "at": "T2" },
        { "kind": "mute", "at": "TL3" },
        { "kind": "solo", "at": "TR3" },
        { "kind": "readout_left", "at": "M1L" },
        { "kind": "readout_right", "at": "M1R" },
        { "kind": "meter", "at": "ML" },
        { "kind": "fader", "at": "MC" },
        { "kind": "trim_tab", "at": "MR2" },
        { "kind": "settings_tab", "at": "MR3" },
        { "kind": "eq_tab", "at": "MR4" },
        { "kind": "dynamics_tab", "at": "MR5" },
        { "kind": "level_readout", "at": "B1" },
        { "kind": "minimize", "at": "B1R" },
        { "kind": "submix", "at": "B2" }
      ]
    }
  ]
}
```

The tab rows keep their positions: `MR2` still holds the trim tab and the
rows below it do not move.  `templates` replaces the whole list, so a partial
override is not possible here.

## Colour categories

The preview uses a semantic colour scheme with five categories:

| Category | Tokens |
|---|---|
| surface | app background, card, active card, border |
| text | primary, secondary |
| control | button face, knob face, knob border, fader slot, fader fill, tab face |
| accent | mute, solo, cue |
| meter | green, yellow, red |

The GUI theme will consume the same categories in a follow-up change.  Until
then the preview palette is the design reference for the colour grouping,
not a second runtime theme.

## Fidelity

`VisualTokens` in the schema holds the tuned visual values: text sizes, tab
glyph sizes, knob size, fader slot width and main/FX cap dimensions.  A GUI
test compares the preview tokens with the runtime constants so a live tweak
cannot silently diverge.

The preview is a layout proof, not a pixel copy of the iced renderer.  It
draws each resolved element by kind; a right-hand row with no tab is outlined
so the sheet shows the reserved space.  Font rasterisation, shadows, hover
timing, meter interpolation and open slideouts are not reproduced.  Use the
preview to approve structure; use `ui/totalcontrol/tools/ui-shot.sh` or the
live GUI to approve final pixels.

The next fidelity step, if needed, is to extract a shared draw list from the
live canvas widgets and have both the iced renderer and the SVG preview
consume it.  Sharing the iced widget code directly would require an offscreen
iced renderer and is a larger change.
