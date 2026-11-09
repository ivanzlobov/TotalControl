//! Shared geometry for a full channel strip.
//!
//! This is the executable form of `docs/spec/TOTALCONTROL-GUI-SPEC.md` §2.8.
//! `strip` lays every full strip out from this model; the tests pin the
//! geometry so element work cannot silently move a neighbouring control.

#![allow(dead_code)]

use iced::{Point, Rectangle, Size};

/// Full strip width at zoom 1.0.
pub const STRIP_W: f32 = 90.0;
/// Full strip height at zoom 1.0.
pub const STRIP_H: f32 = 350.0;
/// Horizontal content inset at zoom 1.0.
pub const CONTENT_X: f32 = 6.0;
/// Content width at zoom 1.0.
pub const CONTENT_W: f32 = 78.0;
/// Top content inset at zoom 1.0.  There is no bottom padding; B2 is flush
/// with the card's bottom edge.
pub const CONTENT_Y: f32 = 4.0;
/// Gap between the stacked rows at zoom 1.0.
pub const ROW_GAP: f32 = 1.0;

pub const NAME_H: f32 = 13.0;
pub const PAN_H: f32 = 54.0;
pub const BUTTON_H: f32 = 18.0;
pub const READOUT_H: f32 = 18.0;
pub const MIDDLE_H: f32 = 205.0;
pub const DB_READOUT_H: f32 = 14.0;
pub const SUBMIX_H: f32 = 18.0;

pub const VU_X: f32 = 0.0;
pub const VU_W: f32 = 13.0;
pub const TRACK_X: f32 = 26.0;
pub const TRACK_W: f32 = 26.0;
/// Full meter + gap + track canvas, from the content's left edge.
pub const FADER_CANVAS_W: f32 = 55.0;
pub const TAB_X: f32 = 56.0;
pub const TAB_W: f32 = 20.0;
pub const TAB_FACE_W: f32 = STRIP_W - (CONTENT_X + TAB_X);
pub const FX_W: f32 = 26.0 * 0.62;
pub const FX_H: f32 = 66.0;
pub const TAB_TOP_WITH_FX: f32 = FX_H + 8.0;
pub const TAB_H: f32 = 22.0;
pub const GEAR_H: f32 = 26.0;
pub const TAB_GAP: f32 = 6.0;
pub const COLLAPSE_X: f32 = 59.0;
pub const COLLAPSE_W: f32 = 19.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelKind {
    Input,
    Playback,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    /// The unit has an FX bus, so the thin MR1 fader exists.
    pub fx: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self { fx: true }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StripLayout {
    pub scale: f32,
    pub card: Rectangle,
    pub content: Rectangle,
    pub name: Rectangle,
    pub pan: Rectangle,
    pub buttons: [Rectangle; 2],
    pub readouts: [Rectangle; 2],
    pub vu: Rectangle,
    /// The full meter+ruler+track canvas.  `fader` is the track rectangle
    /// inside it.
    pub fader_canvas: Rectangle,
    pub fader: Rectangle,
    pub fx: Option<Rectangle>,
    /// Glyph columns for T, gear, EQ, D.
    pub tab_glyphs: [Rectangle; 4],
    /// Faces for T, gear, EQ, D.  Each reaches the channel's right edge.
    pub tab_faces: [Rectangle; 4],
    pub db_readout: Rectangle,
    pub collapse_glyph: Rectangle,
    pub collapse_face: Rectangle,
    pub submix: Option<Rectangle>,
}

impl StripLayout {
    pub fn for_kind(kind: ChannelKind, scale: f32, features: Features) -> Self {
        let r = |x: f32, y: f32, w: f32, h: f32| {
            Rectangle::new(
                Point::new(x * scale, y * scale),
                Size::new(w * scale, h * scale),
            )
        };

        let content = r(CONTENT_X, CONTENT_Y, CONTENT_W, STRIP_H - CONTENT_Y);
        let name = r(CONTENT_X, CONTENT_Y, CONTENT_W, NAME_H);
        let pan_y = CONTENT_Y + NAME_H + ROW_GAP;
        let pan = r(CONTENT_X + 12.0, pan_y, 54.0, PAN_H);
        let buttons_y = pan_y + PAN_H + ROW_GAP;
        let buttons = [
            r(CONTENT_X, buttons_y, 38.0, BUTTON_H),
            r(CONTENT_X + 40.0, buttons_y, 38.0, BUTTON_H),
        ];
        let readouts_y = buttons_y + BUTTON_H + ROW_GAP;
        let readouts = [
            r(CONTENT_X, readouts_y, 38.0, READOUT_H),
            r(CONTENT_X + 40.0, readouts_y, 38.0, READOUT_H),
        ];
        let middle_y = readouts_y + READOUT_H + ROW_GAP;
        let vu = r(CONTENT_X + VU_X, middle_y, VU_W, MIDDLE_H);
        let fader_canvas = r(CONTENT_X, middle_y, FADER_CANVAS_W, MIDDLE_H);
        let fader = r(CONTENT_X + TRACK_X, middle_y, TRACK_W, MIDDLE_H);
        let fx = features
            .fx
            .then(|| r(CONTENT_X + TAB_X + (TAB_W - FX_W) / 2.0, middle_y, FX_W, FX_H));
        let tab_y = middle_y
            + if features.fx {
                TAB_TOP_WITH_FX
            } else {
                0.0
            };
        let tab_glyphs = [
            r(CONTENT_X + TAB_X, tab_y, TAB_W, TAB_H),
            r(CONTENT_X + TAB_X, tab_y + TAB_H + TAB_GAP, TAB_W, GEAR_H),
            r(
                CONTENT_X + TAB_X,
                tab_y + TAB_H + GEAR_H + 2.0 * TAB_GAP,
                TAB_W,
                TAB_H,
            ),
            r(
                CONTENT_X + TAB_X,
                tab_y + 2.0 * TAB_H + GEAR_H + 3.0 * TAB_GAP,
                TAB_W,
                TAB_H,
            ),
        ];
        let tab_faces = tab_glyphs.map(|g| {
            Rectangle::new(g.position(), Size::new(TAB_FACE_W * scale, g.height))
        });

        let db_y = middle_y + MIDDLE_H + ROW_GAP;
        let db_readout = r(CONTENT_X + 20.0, db_y, 38.0, DB_READOUT_H);
        let collapse_glyph = r(
            CONTENT_X + COLLAPSE_X,
            db_y,
            COLLAPSE_W,
            DB_READOUT_H,
        );
        let collapse_face = Rectangle::new(
            collapse_glyph.position(),
            Size::new((STRIP_W - (CONTENT_X + COLLAPSE_X)) * scale, collapse_glyph.height),
        );
        let submix = match kind {
            ChannelKind::Input | ChannelKind::Playback => Some(r(
                0.0,
                STRIP_H - SUBMIX_H,
                STRIP_W,
                SUBMIX_H,
            )),
            ChannelKind::Output => None,
        };

        Self {
            scale,
            card: r(0.0, 0.0, STRIP_W, STRIP_H),
            content,
            name,
            pan,
            buttons,
            readouts,
            vu,
            fader_canvas,
            fader,
            fx,
            tab_glyphs,
            tab_faces,
            db_readout,
            collapse_glyph,
            collapse_face,
            submix,
        }
    }

    pub fn centre_x(&self) -> f32 {
        self.content.x + self.content.width / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(kind: ChannelKind) -> StripLayout {
        StripLayout::for_kind(kind, 1.0, Features::default())
    }

    fn rects_overlap(a: Rectangle, b: Rectangle) -> bool {
        a.x < b.x + b.width
            && b.x < a.x + a.width
            && a.y < b.y + b.height
            && b.y < a.y + a.height
    }

    fn inside(outer: Rectangle, inner: Rectangle) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width
            && inner.y + inner.height <= outer.y + outer.height
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 0.01, "expected {b}, got {a}");
    }

    #[test]
    fn rows_sum_to_the_standard_height() {
        let l = layout(ChannelKind::Input);
        approx(l.card.height, STRIP_H);
        approx(l.name.y, CONTENT_Y);
        approx(l.pan.y, 18.0);
        approx(l.buttons[0].y, 73.0);
        approx(l.readouts[0].y, 92.0);
        approx(l.vu.y, 111.0);
        approx(l.fader.y, 111.0);
        approx(l.db_readout.y, 317.0);
        approx(l.submix.unwrap().y, 332.0);
        approx(l.submix.unwrap().y + l.submix.unwrap().height, STRIP_H);
    }

    #[test]
    fn vu_and_fader_share_top_bottom_height_and_centre() {
        let l = layout(ChannelKind::Input);
        approx(l.vu.y, l.fader.y);
        approx(l.vu.y + l.vu.height, l.fader.y + l.fader.height);
        approx(l.vu.width, VU_W);
        approx(l.fader.width, TRACK_W);
        approx(l.fader_canvas.x, CONTENT_X);
        approx(l.fader_canvas.width, FADER_CANVAS_W);
        assert!(inside(l.fader_canvas, l.vu));
        assert!(inside(l.fader_canvas, l.fader));
        approx(l.fader.x, CONTENT_X + TRACK_X);
        assert!(l.vu.x + l.vu.width < l.fader.x);
        approx(l.fader.x + l.fader.width / 2.0, l.centre_x());
    }

    #[test]
    fn top_rows_and_readout_share_the_centre_line() {
        let l = layout(ChannelKind::Input);
        approx(l.name.x + l.name.width / 2.0, l.centre_x());
        approx(l.pan.x + l.pan.width / 2.0, l.centre_x());
        approx(
            l.buttons[0].x + l.buttons[0].width + (l.buttons[1].x
                - (l.buttons[0].x + l.buttons[0].width))
                / 2.0,
            l.centre_x(),
        );
        approx(
            l.readouts[0].x + l.readouts[0].width + (l.readouts[1].x
                - (l.readouts[0].x + l.readouts[0].width))
                / 2.0,
            l.centre_x(),
        );
        approx(l.db_readout.x + l.db_readout.width / 2.0, l.centre_x());
    }

    #[test]
    fn tab_faces_reach_the_right_edge_and_keep_the_glyph_centred() {
        let l = layout(ChannelKind::Input);
        for (glyph, face) in l.tab_glyphs.iter().zip(l.tab_faces.iter()) {
            approx(face.x + face.width, STRIP_W);
            approx(glyph.x + glyph.width / 2.0, face.x + TAB_W / 2.0);
            assert!(inside(*face, *glyph));
        }
        approx(
            l.collapse_face.x + l.collapse_face.width,
            STRIP_W,
        );
        approx(
            l.collapse_glyph.x + l.collapse_glyph.width / 2.0,
            l.collapse_face.x + COLLAPSE_W / 2.0,
        );
    }

    #[test]
    fn the_submix_bar_is_full_width_and_flush_with_the_bottom() {
        for kind in [ChannelKind::Input, ChannelKind::Playback] {
            let l = layout(kind);
            let s = l.submix.unwrap();
            approx(s.x, 0.0);
            approx(s.width, STRIP_W);
            approx(s.y + s.height, STRIP_H);
        }
        assert!(layout(ChannelKind::Output).submix.is_none());
    }

    #[test]
    fn visible_elements_do_not_overlap() {
        let l = layout(ChannelKind::Input);
        let mut slots = vec![
            l.name,
            l.pan,
            l.buttons[0],
            l.buttons[1],
            l.readouts[0],
            l.readouts[1],
            l.vu,
            l.fader,
            l.db_readout,
            l.collapse_glyph,
            l.submix.unwrap(),
        ];
        if let Some(fx) = l.fx {
            slots.push(fx);
        }
        for glyph in l.tab_glyphs {
            slots.push(glyph);
        }
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                assert!(
                    !rects_overlap(slots[i], slots[j]),
                    "slots {} and {} overlap: {:?} / {:?}",
                    i,
                    j,
                    slots[i],
                    slots[j]
                );
            }
        }
    }

    #[test]
    fn every_element_stays_inside_the_card() {
        for kind in [ChannelKind::Input, ChannelKind::Playback, ChannelKind::Output] {
            let l = layout(kind);
            let mut slots = vec![
                l.name,
                l.pan,
                l.buttons[0],
                l.buttons[1],
                l.readouts[0],
                l.readouts[1],
                l.vu,
                l.fader_canvas,
                l.fader,
                l.db_readout,
                l.collapse_glyph,
                l.collapse_face,
            ];
            if let Some(fx) = l.fx {
                slots.push(fx);
            }
            slots.extend(l.tab_glyphs);
            slots.extend(l.tab_faces);
            if let Some(s) = l.submix {
                slots.push(s);
            }
            for slot in slots {
                assert!(inside(l.card, slot), "{slot:?} leaves the card");
            }
        }
    }

    #[test]
    fn zoom_scales_every_rectangle() {
        let l = StripLayout::for_kind(ChannelKind::Input, 1.5, Features::default());
        approx(l.card.width, STRIP_W * 1.5);
        approx(l.card.height, STRIP_H * 1.5);
        approx(l.vu.height, MIDDLE_H * 1.5);
        approx(l.fader.x + l.fader.width / 2.0, l.centre_x());
        for (glyph, face) in l.tab_glyphs.iter().zip(l.tab_faces.iter()) {
            approx(face.x + face.width, STRIP_W * 1.5);
            approx(glyph.x + glyph.width / 2.0, face.x + TAB_W * 1.5 / 2.0);
        }
    }

    #[test]
    fn an_output_keeps_the_trim_slot_but_has_no_submix() {
        let l = layout(ChannelKind::Output);
        assert!(l.fx.is_some());
        assert!(l.submix.is_none());
        approx(l.tab_glyphs[0].y, l.vu.y + FX_H + 8.0);
    }
}
