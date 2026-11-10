//! Canvas-strip prototype: what does a channel strip cost when one canvas
//! draws the whole strip, instead of a widget tree with a canvas inside?
//!
//! ```text
//! PROBE_STRIPS=12 PROBE_FPS=30 \
//!   cargo run --profile profiling --example strip_canvas_probe -p totalcontrol-gui
//! PROBE_BANK=1 ...     # all strips in one canvas
//! PROBE_ANIM=0 ...     # static scene, the floor
//! ```
//!
//! Prints `PROBE frames/s=N` once a second.  Compare the CPU of the same
//! scene against the live mixer, which builds the strip from widgets.

use std::time::{Duration, Instant};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::widget::row;
use iced::{mouse, window, Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme};
use totalcontrol_layout::{
    ChannelKind, ColorScheme, ElementPlacement, Features, Panel, Rect, StripLayout, VISUAL,
};

static FRAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const STRIP_GAP: f32 = 8.0;

fn env(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn colour(value: &str) -> Color {
    match u32::from_str_radix(value.trim_start_matches('#'), 16) {
        Ok(rgb) => Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
        Err(_) => Color::BLACK,
    }
}

struct Scene {
    layouts: Vec<StripLayout>,
    period: Option<Duration>,
    animate: bool,
    bank: bool,
    cached: bool,
}

fn rect(r: Rect, origin: Point) -> Rectangle {
    Rectangle::new(
        Point::new(origin.x + r.x, origin.y + r.y),
        Size::new(r.width, r.height),
    )
}

fn label(frame: &mut Frame, r: Rectangle, s: &str, size: f32, color: Color, centre: bool) {
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

fn draw_strip(frame: &mut Frame, layout: &StripLayout, origin: Point, phase: f32, c: &ColorScheme) {
    draw_static(frame, layout, origin, c);
    draw_dynamic(frame, layout, origin, phase, c);
}

fn draw_static(frame: &mut Frame, layout: &StripLayout, origin: Point, c: &ColorScheme) {
    let scale = layout.scale;
    let card = rect(layout.card, origin);
    frame.fill(
        &Path::rounded_rectangle(card.position(), card.size(), (8.0 * scale).into()),
        colour(&c.surface.card_active),
    );
    frame.stroke(
        &Path::rounded_rectangle(card.position(), card.size(), (8.0 * scale).into()),
        Stroke::default()
            .with_color(colour(&c.surface.border))
            .with_width(1.0),
    );

    let at = |placement| layout.rect(placement).map(|r| rect(r, origin));

    if let Some(r) = at(ElementPlacement::Name) {
        label(frame, r, "AN1/2", VISUAL.name_size * scale, colour(&c.text.primary), true);
    }
    if let Some(pan) = at(ElementPlacement::Pan) {
        let centre = pan.center();
        let radius = VISUAL.knob_diameter * scale / 2.0;
        frame.fill(
            &Path::circle(centre, radius),
            colour(&c.control.knob_face),
        );
        frame.stroke(
            &Path::circle(centre, radius),
            Stroke::default()
                .with_color(colour(&c.control.knob_border))
                .with_width(1.0),
        );
        label(frame, pan, "C", VISUAL.knob_label_size * scale, colour(&c.text.primary), true);
    }
    for (placement, text) in [
        (ElementPlacement::ButtonLeft, "M"),
        (ElementPlacement::ButtonRight, "S"),
    ] {
        if let Some(r) = at(placement) {
            frame.fill(
                &Path::rounded_rectangle(r.position(), r.size(), (3.0 * scale).into()),
                colour(&c.control.button_face),
            );
            frame.stroke(
                &Path::rounded_rectangle(r.position(), r.size(), (3.0 * scale).into()),
                Stroke::default()
                    .with_color(colour(&c.surface.border))
                    .with_width(1.0),
            );
            label(frame, r, text, VISUAL.button_text_size * scale, colour(&c.text.secondary), true);
        }
    }
    for (placement, text) in [
        (ElementPlacement::ReadoutLeft, "-12.3"),
        (ElementPlacement::ReadoutRight, "-9.8"),
    ] {
        if let Some(r) = at(placement) {
            label(frame, r, text, VISUAL.readout_size * scale, colour(&c.text.secondary), false);
        }
    }

    if let Some(r) = at(ElementPlacement::Level) {
        label(frame, r, "0.0", VISUAL.readout_size * scale, colour(&c.text.primary), true);
    }
    if let Some(face) = layout.face(ElementPlacement::Minimize) {
        let face = rect(face, origin);
        frame.fill(
            &Path::rounded_rectangle(face.position(), face.size(), (3.0 * scale).into()),
            colour(&c.control.button_face),
        );
        label(frame, face, "−", 8.0 * scale, colour(&c.text.secondary), true);
    }
    if let Some(bar) = at(ElementPlacement::Submix) {
        frame.fill(
            &Path::rounded_rectangle(bar.position(), bar.size(), (4.0 * scale).into()),
            colour(&c.control.button_face),
        );
        label(frame, bar, "Phones 7/8", VISUAL.readout_size * scale, colour(&c.text.secondary), true);
    }

    // Right-hand stack: glyphs only; the faces are not drawn at rest.
    for (index, panel) in [Panel::Trim, Panel::Settings, Panel::Eq, Panel::Dynamics]
        .into_iter()
        .enumerate()
    {
        if layout
            .rect(ElementPlacement::Stack(panel))
            .is_none()
        {
            continue;
        }
        let (glyph, _) = layout.stack_tabs[index];
        let glyph = rect(glyph, origin);
        let text = match panel {
            Panel::Trim => "T",
            Panel::Settings => "⚙",
            Panel::Eq => "EQ",
            Panel::Dynamics => "D",
        };
        let size = match panel {
            Panel::Settings => VISUAL.gear_glyph_size,
            _ => VISUAL.tab_glyph_size,
        };
        label(frame, glyph, text, size * scale, colour(&c.text.secondary), true);
    }
}

fn draw_dynamic(frame: &mut Frame, layout: &StripLayout, origin: Point, phase: f32, c: &ColorScheme) {
    let scale = layout.scale;
    let at = |placement| layout.rect(placement).map(|r| rect(r, origin));

    // Meter: two wells, two bars, animated by the phase.
    if let Some(vu) = at(ElementPlacement::Meter) {
        let gap = 2.0 * scale;
        let bar_w = (vu.width - gap) / 2.0;
        for index in 0..2 {
            let well = Rectangle::new(
                Point::new(vu.x + index as f32 * (bar_w + gap), vu.y),
                Size::new(bar_w, vu.height),
            );
            frame.fill(
                &Path::rounded_rectangle(well.position(), well.size(), (2.0 * scale).into()),
                Color::from_rgb8(0x1e, 0x1f, 0x23),
            );
            let level = 0.45 + 0.25 * (phase + index as f32 * 0.7).sin().abs();
            let filled = Rectangle::new(
                Point::new(well.x, well.y + well.height * (1.0 - level)),
                Size::new(well.width, well.height * level),
            );
            frame.fill(
                &Path::rounded_rectangle(filled.position(), filled.size(), (2.0 * scale).into()),
                colour(&c.meter.green),
            );
        }
    }

    // Channel fader: slot, fill below the cap, cap with a centre ridge.
    if let Some(track) = at(ElementPlacement::Fader) {
        let slot_w = VISUAL.fader_slot_w * scale;
        let slot = Rectangle::new(
            Point::new(track.x + track.width / 2.0 - slot_w / 2.0, track.y),
            Size::new(slot_w, track.height),
        );
        frame.fill(
            &Path::rounded_rectangle(slot.position(), slot.size(), (1.0 * scale).into()),
            colour(&c.control.fader_slot),
        );
        let cap_centre = track.y + track.height * 0.18;
        frame.fill_rectangle(
            Point::new(slot.x, cap_centre),
            Size::new(slot.width, slot.height - (cap_centre - slot.y)),
            colour(&c.control.fader_fill),
        );
        let cap = Rectangle::new(
            Point::new(
                track.x + track.width / 2.0 - VISUAL.fader_cap_w * scale / 2.0,
                cap_centre - VISUAL.fader_cap_h * scale / 2.0,
            ),
            Size::new(VISUAL.fader_cap_w * scale, VISUAL.fader_cap_h * scale),
        );
        frame.fill(
            &Path::rounded_rectangle(cap.position(), cap.size(), (2.0 * scale).into()),
            Color::from_rgb8(0xc9, 0xcb, 0xd1),
        );
        frame.fill_rectangle(
            Point::new(cap.x, cap.y + cap.height / 2.0 - scale),
            Size::new(cap.width, 2.0 * scale.max(1.0)),
            Color::from_rgb8(0x4a, 0x4c, 0x52),
        );
    }

}

impl canvas::Program<()> for Scene {
    type State = (f32, canvas::Cache);

    fn update(
        &self,
        state: &mut (f32, canvas::Cache),
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        if let canvas::Event::Window(window::Event::RedrawRequested(_now)) = event {
            FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if !self.animate {
                return None;
            }
            state.0 += 0.15;
            self.period
                .map(|period| canvas::Action::request_redraw_at(Instant::now() + period))
        } else {
            None
        }
    }

    fn draw(
        &self,
        (phase, cache): &(f32, canvas::Cache),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let scheme = ColorScheme::default();
        if !self.cached {
            let mut frame = Frame::new(renderer, bounds.size());
            for (index, layout) in self.layouts.iter().enumerate() {
                let origin = Point::new(index as f32 * (layout.card.width + STRIP_GAP), 0.0);
                draw_strip(&mut frame, layout, origin, *phase + index as f32 * 0.3, &scheme);
            }
            return vec![frame.into_geometry()];
        }
        // Static layer: card, text, controls.  Built once and reused until
        // the scene changes, so a frame only rebuilds the moving parts.
        let static_layer = cache.draw(renderer, bounds.size(), |frame| {
            for (index, layout) in self.layouts.iter().enumerate() {
                let origin = Point::new(index as f32 * (layout.card.width + STRIP_GAP), 0.0);
                draw_static(frame, layout, origin, &scheme);
            }
        });
        let mut frame = Frame::new(renderer, bounds.size());
        for (index, layout) in self.layouts.iter().enumerate() {
            let origin = Point::new(index as f32 * (layout.card.width + STRIP_GAP), 0.0);
            draw_dynamic(&mut frame, layout, origin, *phase + index as f32 * 0.3, &scheme);
        }
        vec![static_layer, frame.into_geometry()]
    }
}

fn view(state: &Scene) -> Element<'_, ()> {
    if state.bank {
        let width: f32 = state
            .layouts
            .iter()
            .map(|layout| layout.card.width + STRIP_GAP)
            .sum();
        let height = state.layouts.first().map_or(0.0, |layout| layout.card.height);
        Canvas::new(Scene {
            layouts: state.layouts.clone(),
            period: state.period,
            animate: state.animate,
            bank: true,
            cached: state.cached,
        })
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
    } else {
        let mut content = row![].spacing(STRIP_GAP);
        for layout in &state.layouts {
            content = content.push(
                Canvas::new(Scene {
                    layouts: vec![layout.clone()],
                    period: state.period,
                    animate: state.animate,
                    bank: false,
                    cached: state.cached,
                })
                .width(Length::Fixed(layout.card.width))
                .height(Length::Fixed(layout.card.height)),
            );
        }
        content.into()
    }
}

fn main() -> iced::Result {
    let strips = env("PROBE_STRIPS", 12.0) as usize;
    let scale = env("PROBE_SCALE", 1.0);
    let fps = env("PROBE_FPS", 30.0) as u64;
    let bank = env("PROBE_BANK", 0.0) != 0.0;
    let animate = env("PROBE_ANIM", 1.0) != 0.0;

    let layouts: Vec<StripLayout> = (0..strips.max(1))
        .map(|_| StripLayout::for_kind(ChannelKind::Input, scale, Features::default()))
        .collect();
    let card = layouts[0].card;
    let width = strips as f32 * (card.width + STRIP_GAP);
    let height = card.height;
    let period = (fps > 0).then(|| Duration::from_millis(1000 / fps));

    std::thread::spawn(|| loop {
        std::thread::sleep(Duration::from_secs(1));
        let n = FRAMES.swap(0, std::sync::atomic::Ordering::Relaxed);
        eprintln!("PROBE frames/s={n}");
    });

    iced::application(
        move || Scene {
            layouts: layouts.clone(),
            period,
            animate,
            bank,
            cached: env("PROBE_CACHE", 1.0) != 0.0,
        },
        update,
        view,
    )
    .title("strip canvas probe")
    .theme(|_: &Scene| Theme::Dark)
    .window(window::Settings {
        size: Size::new(width + 32.0, height + 32.0),
        ..window::Settings::default()
    })
    .run()
}

fn update(_state: &mut Scene, _message: ()) -> Task<()> {
    Task::none()
}
