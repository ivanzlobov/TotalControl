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

1. Change the logical constants or slot rules in
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

The runtime imports the same `StripLayout`; it must not calculate geometry
independently.

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
