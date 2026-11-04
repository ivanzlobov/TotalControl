//! Redraw-cost probe for iced 0.14.
//!
//! An isolated benchmark for the question "what does a meter frame actually
//! cost?".  It builds a window with a chosen size, a chosen number of
//! animated meters and a chosen amount of static content, then asks for
//! `PROBE_FPS` frames per second.
//!
//! Knobs (all optional):
//!
//! ```text
//! PROBE_W=1430 PROBE_H=1260       window size
//! PROBE_METERS=27                 animated meters
//! PROBE_LABELS=200                static labels (stand-in for the mixer UI)
//! PROBE_FPS=30                    frame rate while the meters animate
//! PROBE_BANK=1                    draw all meters in one canvas
//! PROBE_IMAGE=1                   replace the static labels with one
//!                                 full-window image (a baked static UI)
//! ```
//!
//! Examples:
//!
//! ```text
//! # 27 separate meter canvases, 200 static labels
//! PROBE_METERS=27 PROBE_LABELS=200 \
//!   cargo run --profile profiling --example redraw_probe -p totalcontrol-gui
//!
//! # same scene, but the static part is a single pre-rendered image
//! PROBE_METERS=27 PROBE_LABELS=0 PROBE_IMAGE=1 \
//!   cargo run --profile profiling --example redraw_probe -p totalcontrol-gui
//!
//! # all meters in one canvas
//! PROBE_METERS=27 PROBE_BANK=1 PROBE_LABELS=200 \
//!   cargo run --profile profiling --example redraw_probe -p totalcontrol-gui
//! ```
//!
//! `PROBE_METERS=0` makes the scene static (no scheduled frames): the floor
//! for the same window and content.

use std::time::Duration;

static FRAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

use iced::widget::canvas::{self, Canvas, Frame, Geometry};
use iced::widget::image::{self, Image};
use iced::widget::{column, container, row, text};
use iced::{mouse, window, Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme};

const METER_W: f32 = 18.0;
const METER_H: f32 = 120.0;
const METER_GAP: f32 = 6.0;

#[derive(Clone, Copy)]
struct Meter {
    period: Option<Duration>,
}

impl<Message> canvas::Program<Message> for Meter {
    type State = f32;

    fn update(
        &self,
        state: &mut f32,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let canvas::Event::Window(window::Event::RedrawRequested(now)) = event {
            FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            *state = (*state + 0.11) % std::f32::consts::TAU;
            if let Some(period) = self.period {
                return Some(canvas::Action::request_redraw_at(*now + period));
            }
        }
        None
    }

    fn draw(
        &self,
        state: &f32,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let level = 0.5 + 0.5 * state.sin();
        draw_bar(
            &mut frame,
            Rectangle::new(Point::ORIGIN, bounds.size()),
            level,
        );
        vec![frame.into_geometry()]
    }
}

/// All meters in one canvas: one program update, one draw, N bars.
#[derive(Clone, Copy)]
struct Bank {
    bars: usize,
    period: Option<Duration>,
}

impl<Message> canvas::Program<Message> for Bank {
    type State = f32;

    fn update(
        &self,
        state: &mut f32,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let canvas::Event::Window(window::Event::RedrawRequested(now)) = event {
            FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            *state = (*state + 0.11) % std::f32::consts::TAU;
            if let Some(period) = self.period {
                return Some(canvas::Action::request_redraw_at(*now + period));
            }
        }
        None
    }

    fn draw(
        &self,
        state: &f32,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        for i in 0..self.bars {
            let x = i as f32 * (METER_W + METER_GAP);
            let level = 0.5 + 0.5 * (state + i as f32 * 0.4).sin();
            draw_bar(
                &mut frame,
                Rectangle::new(Point::new(x, 0.0), Size::new(METER_W, bounds.height)),
                level,
            );
        }
        vec![frame.into_geometry()]
    }
}

fn draw_bar(frame: &mut Frame, r: Rectangle, level: f32) {
    frame.fill_rectangle(r.position(), r.size(), Color::from_rgb8(0x08, 0x08, 0x0a));
    let h = (r.height - 4.0) * level;
    frame.fill_rectangle(
        Point::new(r.x + 2.0, r.y + r.height - 2.0 - h),
        Size::new(r.width - 4.0, h),
        Color::from_rgb8(0x3d, 0xc8, 0x6a),
    );
}

struct Probe {
    meters: usize,
    labels: usize,
    period: Option<Duration>,
    bank: bool,
    banks: usize,
    image: Option<image::Handle>,
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// A flat, opaque window-sized image: stands in for a static UI that has
/// been baked once into a texture.
fn baked(w: u32, h: u32) -> image::Handle {
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let v = (x * 3 + y) % 32;
            pixels.extend_from_slice(&[20 + v as u8, 20 + v as u8, 24 + v as u8, 255]);
        }
    }
    image::Handle::from_rgba(w, h, pixels)
}

fn main() -> iced::Result {
    let width = env_f32("PROBE_W", 1430.0);
    let height = env_f32("PROBE_H", 1260.0);
    let meters = env_usize("PROBE_METERS", 27);
    let labels = env_usize("PROBE_LABELS", 200);
    let fps = env_usize("PROBE_FPS", 30);
    let bank = env_usize("PROBE_BANK", 0) != 0;
    let banks = env_usize("PROBE_BANKS", 1).max(1);
    let image = env_usize("PROBE_IMAGE", 0) != 0;
    let aa = env_usize("PROBE_AA", 1) != 0;

    if env_usize("PROBE_COUNT", 0) != 0 {
        std::thread::spawn(|| loop {
            std::thread::sleep(Duration::from_secs(1));
            let n = FRAMES.swap(0, std::sync::atomic::Ordering::Relaxed);
            eprintln!("PROBE frames/s={n}");
        });
    }

    iced::application(
        move || Probe {
            meters,
            labels,
            bank,
            banks,
            period: (fps > 0).then(|| Duration::from_millis(1000 / fps as u64)),
            image: image.then(|| baked(width as u32, height as u32)),
        },
        update,
        view,
    )
    .title("redraw probe")
    .theme(|_: &Probe| Theme::Dark)
    .window(window::Settings {
        size: Size::new(width, height),
        ..window::Settings::default()
    })
    .antialiasing(aa)
    .run()
}

fn update(_state: &mut Probe, _message: ()) -> Task<()> {
    Task::none()
}

fn view(state: &Probe) -> Element<'_, ()> {
    let meter_widget: Element<'_, ()> = if state.bank {
        let per = (state.meters + state.banks - 1) / state.banks;
        let mut row = row![].spacing(METER_GAP);
        for b in 0..state.banks {
            let bars = per.min(state.meters.saturating_sub(b * per));
            if bars == 0 {
                break;
            }
            let w = bars as f32 * METER_W + bars.saturating_sub(1) as f32 * METER_GAP;
            row = row.push(
                Canvas::new(Bank {
                    bars,
                    period: state.period,
                })
                .width(Length::Fixed(w))
                .height(Length::Fixed(METER_H)),
            );
        }
        row.into()
    } else {
        let mut meters = row![].spacing(METER_GAP);
        for _ in 0..state.meters {
            meters = meters.push(
                Canvas::new(Meter {
                    period: state.period,
                })
                .width(Length::Fixed(METER_W))
                .height(Length::Fixed(METER_H)),
            );
        }
        meters.into()
    };

    let static_part: Element<'_, ()> = if let Some(handle) = &state.image {
        Image::new(handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        let mut labels = column![].spacing(2);
        for i in 0..state.labels {
            labels = labels.push(text(format!("static label {i}")).size(10));
        }
        labels.into()
    };

    container(column![meter_widget, static_part].spacing(12))
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb8(0x1a, 0x1a, 0x1e).into()),
            ..container::Style::default()
        })
        .into()
}
