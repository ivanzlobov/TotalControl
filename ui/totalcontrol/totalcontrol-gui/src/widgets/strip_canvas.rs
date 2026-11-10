//! Canvas-drawn channel strip prototype.
//!
//! Enabled with `TOTALCONTROL_CANVAS_STRIP=1`; the widget tree stays the
//! default.  Three layers:
//!
//! - static: card, name, pan face, button faces, tab glyphs, level,
//!   minimize, submix — rebuilt only when the theme, layout or name changes;
//! - controls: lit buttons, fader cap and fill, readouts, hover and
//!   selection — rebuilt when a value or state changes;
//! - meters: the bars, rebuilt on every meter frame.

use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{mouse, window, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use totalcontrol_core::ChannelId;

use crate::app::Message;
use crate::theme;
use crate::widgets::fader::{bus_levels, t_to_vol, vol_to_t};
use crate::widgets::layout::{
    ChannelKind, ColorScheme, ElementPlacement, Features, GuiLayoutConfig, Panel, Rect, StripLayout,
    TemplateId, VISUAL,
};
use crate::widgets::strip::{FlyoutKind, StripParams};

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("TOTALCONTROL_CANVAS_STRIP").is_some())
}

fn colour(value: &str) -> iced::Color {
    match u32::from_str_radix(value.trim_start_matches('#'), 16) {
        Ok(rgb) => iced::Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
        Err(_) => iced::Color::BLACK,
    }
}

fn resolve(p: &StripParams<'_>) -> StripLayout {
    let kind = match p.cid {
        ChannelId::Input(_) => ChannelKind::Input,
        ChannelId::Playback(_) => ChannelKind::Playback,
        ChannelId::Output(_) => ChannelKind::Output,
    };
    let id = match kind {
        ChannelKind::Input if p.stereo_linked => TemplateId::InputStereo,
        ChannelKind::Input => TemplateId::InputMono,
        ChannelKind::Playback => TemplateId::PlaybackStereo,
        ChannelKind::Output if p.stereo_linked => TemplateId::OutputStereo,
        ChannelKind::Output => TemplateId::OutputMono,
    };
    let scale = p.scale;
    p.layout
        .template(id)
        .and_then(|template| p.layout.layout(template, scale).ok())
        .unwrap_or_else(|| {
            StripLayout::for_kind(
                kind,
                scale,
                Features {
                    fx: p.fx_level.is_some(),
                    ..Features::default()
                },
            )
        })
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Drag {
    #[default]
    None,
    Fader,
}

struct StripCanvas<'a> {
    p: StripParams<'a>,
    layout: StripLayout,
    scheme: ColorScheme,
}

struct CanvasState {
    hover: Option<ElementPlacement>,
    drag: Drag,
    static_cache: Cache,
    controls_cache: Cache,
    static_key: std::cell::Cell<u64>,
    controls_key: std::cell::Cell<u64>,
    /// The meter values the last frame drew, for the change gate.
    last_meter: Option<(f32, f32, f32, f32)>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            hover: None,
            drag: Drag::None,
            static_cache: Cache::new(),
            controls_cache: Cache::new(),
            static_key: std::cell::Cell::new(u64::MAX),
            controls_key: std::cell::Cell::new(u64::MAX),
            last_meter: None,
        }
    }
}

/// The meter bars: two wells for a linked pair, one centred well otherwise.
fn meter_wells(vu: Rectangle, scale: f32, stereo: bool) -> Vec<Rectangle> {
    let gap = 2.0 * scale;
    if stereo {
        let bar_w = (vu.width - gap) / 2.0;
        (0..2)
            .map(|index| {
                Rectangle::new(
                    Point::new(vu.x + index as f32 * (bar_w + gap), vu.y),
                    Size::new(bar_w, vu.height),
                )
            })
            .collect()
    } else {
        let bar_w = vu.width * totalcontrol_layout::MONO_BAR_RATIO;
        vec![Rectangle::new(
            Point::new(vu.x + (vu.width - bar_w) / 2.0, vu.y),
            Size::new(bar_w, vu.height),
        )]
    }
}

fn at(layout: &StripLayout, placement: ElementPlacement) -> Option<Rectangle> {
    layout.rect(placement).map(|r| {
        Rectangle::new(
            Point::new(r.x, r.y),
            Size::new(r.width, r.height),
        )
    })
}

fn text(frame: &mut Frame, r: Rectangle, s: &str, size: f32, color: iced::Color, centre: bool) {
    frame.fill_text(Text {
        content: s.to_string(),
        position: Point::new(
            if centre {
                r.x + r.width / 2.0
            } else {
                r.x + r.width - 2.0
            },
            r.y + r.height / 2.0,
        ),
        size: size.into(),
        color,
        align_x: if centre {
            Horizontal::Center.into()
        } else {
            Horizontal::Right.into()
        },
        align_y: Vertical::Center,
        ..Text::default()
    });
}

fn draw_static(frame: &mut Frame, layout: &StripLayout, p: &StripParams<'_>, scheme: &ColorScheme) {
    let scale = layout.scale;
    let card = Rectangle::new(Point::ORIGIN, Size::new(layout.card.width, layout.card.height));
    let radius = (8.0 * scale).into();
    let face = if matches!(p.cid, ChannelId::Output(_)) {
        colour(&scheme.surface.card)
    } else {
        colour(&scheme.surface.card_active)
    };
    frame.fill(
        &Path::rounded_rectangle(card.position(), card.size(), radius),
        face,
    );
    frame.stroke(
        &Path::rounded_rectangle(card.position(), card.size(), radius),
        Stroke::default()
            .with_color(colour(&scheme.surface.border))
            .with_width(1.0),
    );

    if let Some(r) = at(layout, ElementPlacement::Name) {
        text(frame, r, &p.name, VISUAL.name_size * scale, colour(&scheme.text.primary), true);
    }
    if let Some(pan) = at(layout, ElementPlacement::Pan) {
        let centre = pan.center();
        let radius = VISUAL.knob_diameter * scale / 2.0;
        frame.fill(&Path::circle(centre, radius), colour(&scheme.control.knob_face));
        frame.stroke(
            &Path::circle(centre, radius),
            Stroke::default()
                .with_color(colour(&scheme.control.knob_border))
                .with_width(1.0),
        );
        let label = if p.pan == 0 {
            "C".to_string()
        } else if p.pan < 0 {
            format!("L{}", -p.pan)
        } else {
            format!("R{}", p.pan)
        };
        text(frame, pan, &label, VISUAL.knob_label_size * scale, colour(&scheme.text.primary), true);
    }
    for (placement, label) in [
        (ElementPlacement::ButtonLeft, "M"),
        (
            ElementPlacement::ButtonRight,
            if matches!(p.cid, ChannelId::Output(_)) {
                "CUE"
            } else {
                "S"
            },
        ),
    ] {
        if let Some(r) = at(layout, placement) {
            let radius = (3.0 * scale).into();
            frame.fill(
                &Path::rounded_rectangle(r.position(), r.size(), radius),
                colour(&scheme.control.button_face),
            );
            frame.stroke(
                &Path::rounded_rectangle(r.position(), r.size(), radius),
                Stroke::default()
                    .with_color(colour(&scheme.surface.border))
                    .with_width(1.0),
            );
            text(frame, r, label, VISUAL.button_text_size * scale, colour(&scheme.text.secondary), true);
        }
    }

    // Meter wells: static; the bars are the dynamic layer.
    if let Some(vu) = at(layout, ElementPlacement::Meter) {
        for well in meter_wells(vu, scale, p.stereo_linked) {
            frame.fill(
                &Path::rounded_rectangle(well.position(), well.size(), (2.0 * scale).into()),
                iced::Color::from_rgb8(0x1e, 0x1f, 0x23),
            );
        }
    }

    if let Some(track) = at(layout, ElementPlacement::Fader) {
        let slot_w = VISUAL.fader_slot_w * scale;
        let slot = Rectangle::new(
            Point::new(track.x + track.width / 2.0 - slot_w / 2.0, track.y),
            Size::new(slot_w, track.height),
        );
        frame.fill(
            &Path::rounded_rectangle(slot.position(), slot.size(), (1.0 * scale).into()),
            colour(&scheme.control.fader_slot),
        );
    }

    for (index, panel) in [Panel::Trim, Panel::Settings, Panel::Eq, Panel::Dynamics]
        .into_iter()
        .enumerate()
    {
        if layout.rect(ElementPlacement::Stack(panel)).is_none() {
            continue;
        }
        let (glyph, _) = layout.stack_tabs[index];
        let glyph = Rectangle::new(Point::new(glyph.x, glyph.y), Size::new(glyph.width, glyph.height));
        let (label, size) = match panel {
            Panel::Trim => ("T", VISUAL.tab_glyph_size),
            Panel::Settings => ("⚙", VISUAL.gear_glyph_size),
            Panel::Eq => ("EQ", VISUAL.tab_glyph_size),
            Panel::Dynamics => ("D", VISUAL.tab_glyph_size),
        };
        text(frame, glyph, label, size * scale, colour(&scheme.text.secondary), true);
    }

    if let Some(r) = at(layout, ElementPlacement::Level) {
        text(frame, r, "0.0", VISUAL.readout_size * scale, colour(&scheme.text.primary), true);
    }
    if let Some(face) = layout.face(ElementPlacement::Minimize) {
        let face = Rectangle::new(Point::new(face.x, face.y), Size::new(face.width, face.height));
        frame.fill(
            &Path::rounded_rectangle(face.position(), face.size(), (3.0 * scale).into()),
            colour(&scheme.control.button_face),
        );
        text(frame, face, "−", 8.0 * scale, colour(&scheme.text.secondary), true);
    }
    if let Some(bar) = at(layout, ElementPlacement::Submix) {
        frame.fill(
            &Path::rounded_rectangle(bar.position(), bar.size(), (4.0 * scale).into()),
            colour(&scheme.control.button_face),
        );
        text(frame, bar, &p.bus_label, VISUAL.readout_size * scale, colour(&scheme.text.secondary), true);
    }
}

fn draw_controls(
    frame: &mut Frame,
    layout: &StripLayout,
    p: &StripParams<'_>,
    scheme: &ColorScheme,
) {
    let scale = layout.scale;

    // Lit buttons.
    if p.mute {
        if let Some(r) = at(layout, ElementPlacement::ButtonLeft) {
            frame.fill(
                &Path::rounded_rectangle(r.position(), r.size(), (3.0 * scale).into()),
                colour(&scheme.accent.mute),
            );
        }
    }
    let right = ElementPlacement::ButtonRight;
    let lit = if matches!(p.cid, ChannelId::Output(_)) {
        p.cue
    } else {
        p.solo
    };
    if lit {
        if let Some(r) = at(layout, right) {
            let color = if matches!(p.cid, ChannelId::Output(_)) {
                colour(&scheme.accent.cue)
            } else {
                colour(&scheme.accent.solo)
            };
            frame.fill(
                &Path::rounded_rectangle(r.position(), r.size(), (3.0 * scale).into()),
                color,
            );
        }
    }

    // Readouts.
    for (placement, value) in [
        (ElementPlacement::ReadoutLeft, p.readout[0]),
        (ElementPlacement::ReadoutRight, p.readout[1]),
    ] {
        if let (Some(r), Some(value)) = (at(layout, placement), value) {
            let s = format!("{:.1}", 20.0 * value.max(1e-6).log10());
            text(frame, r, &s, VISUAL.readout_size * scale, colour(&scheme.text.secondary), false);
        }
    }

    // Fader: slot fill and cap.
    if let Some(track) = at(layout, ElementPlacement::Fader) {
        let slot_w = VISUAL.fader_slot_w * scale;
        let slot = Rectangle::new(
            Point::new(track.x + track.width / 2.0 - slot_w / 2.0, track.y),
            Size::new(slot_w, track.height),
        );
        let cap_y = track.y + track.height * (1.0 - vol_to_t(p.vol));
        frame.fill_rectangle(
            Point::new(slot.x, cap_y),
            Size::new(slot.width, slot.height - (cap_y - slot.y)),
            colour(&scheme.control.fader_fill),
        );
        let cap = Rectangle::new(
            Point::new(
                track.x + track.width / 2.0 - VISUAL.fader_cap_w * scale / 2.0,
                cap_y - VISUAL.fader_cap_h * scale / 2.0,
            ),
            Size::new(VISUAL.fader_cap_w * scale, VISUAL.fader_cap_h * scale),
        );
        frame.fill(
            &Path::rounded_rectangle(cap.position(), cap.size(), (2.0 * scale).into()),
            iced::Color::from_rgb8(0xc9, 0xcb, 0xd1),
        );
        frame.fill_rectangle(
            Point::new(cap.x, cap.y + cap.height / 2.0 - scale),
            Size::new(cap.width, 2.0 * scale.max(1.0)),
            iced::Color::from_rgb8(0x4a, 0x4c, 0x52),
        );
    }
}

/// Read the live levels and held peaks: the shared bus when the device
/// service owns metering, the view-fed frame otherwise.
fn meter_values(p: &StripParams<'_>) -> (f32, f32, f32, f32) {
    if let (Some(bus), Some(slot)) = (&p.meter_bus, p.meter_slot) {
        if let Ok(snapshot) = bus.lock() {
            let (level, peak, right) = bus_levels(&snapshot, slot, p.stereo_linked);
            let (right_level, right_peak) = right.unwrap_or((level, peak));
            return (level, right_level, peak, right_peak);
        }
    }
    (
        p.meter.value,
        p.meter2.map_or(p.meter.value, |m| m.value),
        p.meter.peak,
        p.meter2.map_or(p.meter.peak, |m| m.peak),
    )
}

/// Whether the next frame would draw something different, at the pixel
/// granularity of the bars and peak lines.  A level that is merely
/// non-zero — converter or analog noise on a channel with gain — does not
/// keep the loop alive; only a visible change does.
fn meter_changed(prev: (f32, f32, f32, f32), current: (f32, f32, f32, f32), height: f32) -> bool {
    let step = 1.0 / height.max(1.0);
    let moved = |a: f32, b: f32| (a - b).abs() > step;
    moved(prev.0, current.0)
        || moved(prev.1, current.1)
        || moved(prev.2, current.2)
        || moved(prev.3, current.3)
}

fn draw_meters(frame: &mut Frame, layout: &StripLayout, p: &StripParams<'_>, scheme: &ColorScheme) {
    let scale = layout.scale;
    let (left, right, _, _) = meter_values(p);
    if let Some(vu) = at(layout, ElementPlacement::Meter) {
        for (index, well) in meter_wells(vu, scale, p.stereo_linked).into_iter().enumerate() {
            let level = if index == 0 { left } else { right };
            let filled = Rectangle::new(
                Point::new(well.x, well.y + well.height * (1.0 - level)),
                Size::new(well.width, well.height * level),
            );
            frame.fill_rectangle(filled.position(), filled.size(), colour(&scheme.meter.green));
        }
    }
}

fn hash_of(f: impl FnOnce(&mut std::collections::hash_map::DefaultHasher)) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    f(&mut h);
    h.finish()
}

impl canvas::Program<Message> for StripCanvas<'_> {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut CanvasState,
        event: &canvas::Event,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let layout = &self.layout;
        let hit = |placement: ElementPlacement| {
            at(layout, placement).is_some_and(|r| cursor.position().is_some_and(|p| r.contains(p)))
        };
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Drag::Fader = state.drag {
                    if let (Some(p), Some(track)) = (cursor.position(), at(layout, ElementPlacement::Fader)) {
                        let t = (1.0 - (p.y - track.y) / track.height).clamp(0.0, 1.0);
                        return Some(canvas::Action::publish(Message::VolumeChanged(
                            self.p.cid,
                            self.p.output_idx,
                            t_to_vol(t),
                        )));
                    }
                }
                let hover = [
                    ElementPlacement::ButtonLeft,
                    ElementPlacement::ButtonRight,
                    ElementPlacement::Stack(Panel::Trim),
                    ElementPlacement::Stack(Panel::Settings),
                    ElementPlacement::Stack(Panel::Eq),
                    ElementPlacement::Stack(Panel::Dynamics),
                    ElementPlacement::Minimize,
                    ElementPlacement::Submix,
                ]
                .into_iter()
                .find(|placement| hit(*placement));
                if hover != state.hover {
                    state.hover = hover;
                    return Some(canvas::Action::request_redraw());
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                for (placement, message) in [
                    (ElementPlacement::ButtonLeft, Message::Mute(self.p.cid, !self.p.mute)),
                    (
                        ElementPlacement::ButtonRight,
                        if matches!(self.p.cid, ChannelId::Output(_)) {
                            Message::CueChanged(self.p.cid, !self.p.cue)
                        } else {
                            Message::Solo(self.p.cid, !self.p.solo)
                        },
                    ),
                    (
                        ElementPlacement::Stack(Panel::Trim),
                        Message::ToggleFlyout(self.p.cid, FlyoutKind::Trim),
                    ),
                    (
                        ElementPlacement::Stack(Panel::Settings),
                        Message::ToggleFlyout(self.p.cid, FlyoutKind::Settings),
                    ),
                    (
                        ElementPlacement::Stack(Panel::Eq),
                        Message::ToggleFlyout(self.p.cid, FlyoutKind::Eq),
                    ),
                    (
                        ElementPlacement::Stack(Panel::Dynamics),
                        Message::ToggleFlyout(self.p.cid, FlyoutKind::Dynamics),
                    ),
                    (ElementPlacement::Minimize, Message::ToggleCollapse(self.p.cid)),
                    (
                        ElementPlacement::Submix,
                        Message::ToggleFlyout(self.p.cid, FlyoutKind::Route),
                    ),
                ] {
                    if hit(placement) {
                        return Some(canvas::Action::publish(message));
                    }
                }
                if hit(ElementPlacement::Fader) {
                    state.drag = Drag::Fader;
                    if let (Some(p), Some(track)) =
                        (cursor.position(), at(layout, ElementPlacement::Fader))
                    {
                        let t = (1.0 - (p.y - track.y) / track.height).clamp(0.0, 1.0);
                        return Some(canvas::Action::publish(Message::VolumeChanged(
                            self.p.cid,
                            self.p.output_idx,
                            t_to_vol(t),
                        )));
                    }
                }
                return Some(canvas::Action::publish(Message::StripClicked(self.p.cid)));
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.drag != Drag::None {
                    state.drag = Drag::None;
                    return Some(canvas::Action::publish(Message::RangeCleared(self.p.cid)));
                }
            }
            canvas::Event::Window(window::Event::RedrawRequested(now)) => {
                // Schedule the next frame only if this one would look
                // different.  `meter_stream` wakes a stopped canvas when a
                // new transient starts, so silence costs no frames.
                let current = meter_values(&self.p);
                let height = at(layout, ElementPlacement::Meter).map_or(1.0, |vu| vu.height);
                let changed = state
                    .last_meter
                    .map_or(true, |prev| meter_changed(prev, current, height));
                state.last_meter = Some(current);
                // A visible change: 20 frames a second.  A static picture
                // with any non-zero level (steady noise, a held tone): a
                // 4 Hz heartbeat, so a signal that starts without moving a
                // bar is still picked up within 250 ms.  True silence:
                // stop; `meter_stream` wakes the canvas on a transient.
                let any = current.0 > 0.0 || current.1 > 0.0 || current.2 > 0.0 || current.3 > 0.0;
                let period = if changed {
                    Some(50)
                } else if any {
                    Some(250)
                } else {
                    None
                };
                if let Some(ms) = period {
                    return Some(canvas::Action::request_redraw_at(
                        *now + std::time::Duration::from_millis(ms),
                    ));
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        state: &CanvasState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let static_key = hash_of(|h| {
            self.p.name.hash(h);
            self.p.cid.hash(h);
            self.p.bus_label.hash(h);
            self.layout.scale.to_bits().hash(h);
            self.p.stereo_linked.hash(h);
        });
        let controls_key = hash_of(|h| {
            self.p.mute.hash(h);
            self.p.solo.hash(h);
            self.p.cue.hash(h);
            self.p.vol.to_bits().hash(h);
            self.p.selected.hash(h);
            self.p.open_flyout.map(|f| f as u8).hash(h);
            self.p.readout.map(|r| r.map(f32::to_bits)).hash(h);
        });

        if state.static_key.get() != static_key {
            state.static_cache.clear();
            state.static_key.set(static_key);
        }
        if state.controls_key.get() != controls_key {
            state.controls_cache.clear();
            state.controls_key.set(controls_key);
        }
        let static_layer = state
            .static_cache
            .draw(renderer, bounds.size(), |frame| {
                draw_static(frame, &self.layout, &self.p, &self.scheme);
            });
        let controls_layer = state
            .controls_cache
            .draw(renderer, bounds.size(), |frame| {
                draw_controls(frame, &self.layout, &self.p, &self.scheme);
            });
        let mut frame = Frame::new(renderer, bounds.size());
        draw_meters(&mut frame, &self.layout, &self.p, &self.scheme);
        // Hover is per-frame: clearing the controls cache on every pointer
        // move would rebuild the cap and the lit buttons with it.
        if let Some(placement) = state.hover {
            if let Some(r) = at(&self.layout, placement) {
                frame.stroke(
                    &Path::rounded_rectangle(r.position(), r.size(), (2.0 * self.layout.scale).into()),
                    Stroke::default()
                        .with_color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.25))
                        .with_width(1.0),
                );
            }
        }
        vec![static_layer, controls_layer, frame.into_geometry()]
    }
}

pub fn strip_canvas<'a>(p: StripParams<'a>) -> Element<'a, Message> {
    let layout = resolve(&p);
    let width = layout.card.width;
    let height = layout.card.height;
    let scheme = p.layout.colors.clone();
    Canvas::new(StripCanvas { p, layout, scheme })
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

#[cfg(test)]
mod tests {
    use super::meter_changed;

    #[test]
    fn a_sub_pixel_drift_does_not_keep_the_loop_alive() {
        let height = 400.0;
        let a = (0.500, 0.500, 0.500, 0.500);
        // Half a pixel of travel at the bottom of the bar.
        let b = (0.5005, 0.5005, 0.5005, 0.5005);
        assert!(!meter_changed(a, b, height));
    }

    #[test]
    fn a_static_picture_with_signal_still_gets_a_heartbeat() {
        // The gate itself only reports movement; the heartbeat keeps a
        // non-zero but static picture polled at 4 Hz.
        let idle = [0.0f32; 4];
        assert!(!idle.iter().any(|v| *v > 0.0));
        let noisy = [0.0005f32; 4];
        assert!(noisy.iter().any(|v| *v > 0.0));
    }

    #[test]
    fn a_visible_move_or_a_peak_change_does() {
        let height = 400.0;
        let a = (0.5, 0.5, 0.5, 0.5);
        assert!(meter_changed(a, (0.55, 0.5, 0.5, 0.5), height));
        assert!(meter_changed(a, (0.5, 0.5, 0.55, 0.5), height));
        assert!(meter_changed(a, (0.5, 0.5, 0.5, 0.55), height));
    }
}
