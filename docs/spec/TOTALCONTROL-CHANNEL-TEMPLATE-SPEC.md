# Channel template schema

This specification defines the shared channel-template model used by the
GUI and the layout preview.  It is the developer-facing form of
`TOTALCONTROL-GUI-SPEC.md` §2.8.

## Layers

| Layer | Purpose |
|---|---|
| `Features` | Which optional controls exist: FX, Trim, EQ, Dynamics |
| `TemplateSpec` | A named standard channel: kind, features, mono/stereo |
| `StripLayout` | Logical rectangles resolved for one template and one zoom |
| Preview | Renders the resolved rectangles and the semantic colour scheme |
| Runtime | Selects a standard channel and fills its dynamic slots |

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

A template owns the presence of its optional slots.  The runtime must not
independently decide that a channel has a different set of slots.

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

1. Change the logical defaults or slot rules in
   `ui/totalcontrol/totalcontrol-layout/src/lib.rs`.
2. Add or update the relevant invariant test.
3. Render the preview:

   ```sh
   cd ui/totalcontrol
   cargo run -p totalcontrol-layout --bin channel-preview -- preview.svg 2
   ```

   The second argument is the zoom.  The SVG can be converted to PNG with
   `magick preview.svg preview.png`.
4. Review the rendered sheet.
5. Only then change runtime slot filling.

## Layout configuration

`totalcontrol-layout` reads a complete layout document in JSON.  The document
is the configuration form of this specification: one geometry, one visual
token set, one colour scheme and a list of channel templates.

The document describes the whole sheet, not the placement of an individual
channel element.  Placing each channel element and binding it to data is the
next step.

The preview is the only consumer.  The runtime still renders the compiled
defaults.

### Document

| Key | Type | Meaning |
|---|---|---|
| `$schema` | string | Optional path or URI of the schema file.  Ignored when rendering. |
| `schema` | integer | Document version.  Must be `1`. |
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
| `features` | object | `fx`, `trim`, `eq`, `dynamics` booleans; see [Standard templates](#standard-templates). |
| `stereo` | boolean | Linked pair: the VU draws two bars and the name uses the pair form. |

The list must contain at least one entry.

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
shares geometry, dimensions, semantic colours and text sizes.  Font
rasterisation, shadows, hover timing, meter interpolation and open slideouts
are not reproduced.  Use the preview to approve structure; use
`ui/totalcontrol/tools/ui-shot.sh` or the live GUI to approve final pixels.

The next fidelity step, if needed, is to extract a shared draw list from the
live canvas widgets and have both the iced renderer and the SVG preview
consume it.  Sharing the iced widget code directly would require an offscreen
iced renderer and is a larger change.
