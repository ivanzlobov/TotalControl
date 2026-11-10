//! Render the standard channel templates to an SVG sheet.
//!
//! ```text
//! cargo run -p totalcontrol-layout --bin channel-preview -- out.svg [scale]
//! ```

use std::fmt::Write as _;
use std::path::PathBuf;

use totalcontrol_layout::{
    Action, ChannelKind, ChannelSide, ColorScheme, ElementKind, ElementPlacement, GuiLayoutConfig,
    LayoutTemplate, Panel, Rect, TemplateId, VisualTokens, MONO_BAR_RATIO, VU_LABELS,
};

fn main() {
    let mut positional = Vec::new();
    let mut config_path = None;
    let mut example_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config_path = args.next().map(PathBuf::from),
            "--example" => example_path = args.next().map(PathBuf::from),
            _ => positional.push(arg),
        }
    }

    if let Some(path) = example_path {
        let config = GuiLayoutConfig::example();
        let schema_path = path.with_file_name(totalcontrol_layout::SCHEMA_FILE_NAME);
        if let Err(error) = std::fs::write(&path, config.to_pretty_json()) {
            eprintln!("could not write {}: {error}", path.display());
            std::process::exit(1);
        }
        if let Err(error) = std::fs::write(&schema_path, totalcontrol_layout::SCHEMA_JSON) {
            eprintln!("could not write {}: {error}", schema_path.display());
            std::process::exit(1);
        }
        println!("wrote example {}", path.display());
        println!(
            "wrote schema {} (referenced by the example)",
            schema_path.display()
        );
        return;
    }

    let config = match config_path {
        Some(path) => match GuiLayoutConfig::load(&path) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("invalid layout config: {error}");
                std::process::exit(1);
            }
        },
        None => GuiLayoutConfig::default(),
    };

    let out = positional
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("channel-preview.svg"));
    let scale = positional
        .get(1)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(2.0)
        .clamp(0.75, 3.0);

    let svg = sheet(scale, &config);
    if let Err(error) = std::fs::write(&out, svg) {
        eprintln!("could not write {}: {error}", out.display());
        std::process::exit(1);
    }
    println!("wrote {}", out.display());
}

fn sheet(scale: f32, config: &GuiLayoutConfig) -> String {
    let colors = &config.colors;
    let templates = &config.templates;
    let card_w = config.geometry.strip_w * scale;
    let card_h = config.geometry.strip_h * scale;
    let gutter = 24.0 * scale;
    let header = 48.0 * scale;
    let footer = 24.0 * scale;
    let margin = 20.0 * scale;
    let width = margin * 2.0
        + templates.len() as f32 * card_w
        + (templates.len().saturating_sub(1)) as f32 * gutter;
    let height = margin * 2.0 + header + card_h + footer;

    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.1}" height="{height:.1}" viewBox="0 0 {width:.1} {height:.1}">"#
    );
    let _ = write!(
        svg,
        r#"<rect width="100%" height="100%" fill="{}"/>"#,
        colors.surface.app_background
    );
    let _ = write!(
        svg,
        r#"<text x="{margin:.1}" y="{y:.1}" fill="{}" font-family="sans-serif" font-size="{size:.1}" font-weight="600">TotalControl channel templates</text>"#,
        colors.text.primary,
        y = margin + 18.0 * scale,
        size = 15.0 * scale,
    );

    for (index, template) in templates.iter().enumerate() {
        let x = margin + index as f32 * (card_w + gutter);
        let y = margin + header;
        let _ = write!(
            svg,
            r#"<text x="{x:.1}" y="{label_y:.1}" fill="{}" font-family="sans-serif" font-size="{size:.1}">{}</text>"#,
            colors.text.secondary,
            template.id.label(),
            label_y = y - 8.0 * scale,
            size = 9.0 * scale,
        );
        draw_template(&mut svg, template, x, y, scale, config);
    }

    let footer_y = margin + header + card_h + 16.0 * scale;
    let _ = write!(
        svg,
        r#"<text x="{x:.1}" y="{y:.1}" fill="{color}" font-family="sans-serif" font-size="{size:.1}">Geometry from totalcontrol-layout; colours from the semantic preview scheme. Dynamic states and slideouts are not shown.</text>"#,
        x = margin,
        y = footer_y,
        color = colors.text.secondary,
        size = 8.0 * scale,
    );
    let _ = write!(svg, "</svg>");
    svg
}

fn draw_template(
    svg: &mut String,
    template: &LayoutTemplate,
    ox: f32,
    oy: f32,
    scale: f32,
    config: &GuiLayoutConfig,
) {
    let layout = config
        .layout(template, scale)
        .unwrap_or_else(|error| panic!("preview template {:?}: {error}", template.id));
    let c = &config.colors;
    let v = &config.visual;
    let card = translated(layout.card, ox, oy);
    let active = matches!(template.kind, ChannelKind::Input | ChannelKind::Playback);
    fill_rect(
        svg,
        card,
        if active {
            &c.surface.card_active
        } else {
            &c.surface.card
        },
        &c.surface.border,
        8.0 * scale,
    );

    let vu = layout
        .rect(ElementPlacement::Meter)
        .map(|rect| translated(rect, ox, oy));
    for resolved in &layout.elements {
        let rect = translated(resolved.rect, ox, oy);
        match resolved.element.kind {
            ElementKind::Name => draw_text(
                svg,
                rect,
                sample_name(template.id),
                &c.text.primary,
                v.name_size * scale,
            ),
            ElementKind::Pan => draw_pan(svg, rect, c, v, scale),
            ElementKind::Button(action) => {
                let label = match action {
                    Action::Mute => "M",
                    Action::Solo => "S",
                    Action::Cue => "CUE",
                };
                draw_button(svg, rect, label, c, v, scale);
            }
            ElementKind::Readout(side) => {
                let value = match side {
                    ChannelSide::Left => "-12.3",
                    ChannelSide::Right => "-9.8",
                };
                draw_text_aligned(
                    svg,
                    rect,
                    value,
                    &c.text.secondary,
                    v.readout_size * scale,
                    "end",
                );
            }
            ElementKind::Meter => draw_meter(svg, rect, template.stereo, c, v, scale),
            ElementKind::Fader => {
                if let Some(vu) = vu {
                    draw_fader(svg, rect, vu, c, v, scale);
                }
            }
            ElementKind::FxFader => draw_fx(svg, rect, template.kind, c, v, scale),
            ElementKind::Tab(panel) => draw_tab(svg, rect, panel, c, v, scale),
            ElementKind::Level => {
                draw_text(svg, rect, "0.0", &c.text.primary, v.readout_size * scale)
            }
            ElementKind::Minimize => {
                let face = translated(resolved.face.unwrap_or(resolved.rect), ox, oy);
                fill_rect(
                    svg,
                    face,
                    &c.control.button_face,
                    &c.surface.border,
                    3.0 * scale,
                );
                draw_text(svg, face, "−", &c.text.secondary, 8.0 * scale);
            }
            ElementKind::Submix => draw_submix(svg, rect, c, v, scale),
        }
    }

    // An absent tab keeps its reserved row; outline it so the sheet shows
    // which tabs the template does not have.
    for panel in Panel::ALL {
        if layout.element(ElementKind::Tab(panel)).is_some() {
            continue;
        }
        let (glyph, _) = layout.stack_tabs[panel.stack_index()];
        let glyph = translated(glyph, ox, oy);
        let _ = write!(
            svg,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="none" stroke="{color}" stroke-width="{sw:.1}" rx="{r:.1}"/>"#,
            x = glyph.x,
            y = glyph.y,
            w = glyph.width,
            h = glyph.height,
            color = &c.surface.border,
            sw = 0.8 * scale,
            r = 2.0 * scale,
        );
    }
}

fn draw_pan(svg: &mut String, pan: Rect, c: &ColorScheme, v: &VisualTokens, scale: f32) {
    let _ = write!(
        svg,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{face}" stroke="{border}" stroke-width="{sw:.1}"/>"#,
        cx = pan.centre_x(),
        cy = pan.centre_y(),
        r = v.knob_diameter * scale / 2.0,
        face = &c.control.knob_face,
        border = &c.control.knob_border,
        sw = 1.0 * scale,
    );
    let _ = write!(
        svg,
        r#"<line x1="{cx:.1}" y1="{top:.1}" x2="{cx:.1}" y2="{tick:.1}" stroke="{color}" stroke-width="{sw:.1}"/>"#,
        cx = pan.centre_x(),
        top = pan.centre_y() - v.knob_diameter * scale / 2.0,
        tick = pan.centre_y() - v.knob_diameter * scale / 2.0 - 3.0 * scale,
        color = &c.text.secondary,
        sw = 1.0 * scale,
    );
    draw_text(svg, pan, "C", &c.text.primary, v.knob_label_size * scale);
}

fn draw_button(
    svg: &mut String,
    rect: Rect,
    label: &str,
    c: &ColorScheme,
    v: &VisualTokens,
    scale: f32,
) {
    fill_rect(
        svg,
        rect,
        &c.control.button_face,
        &c.surface.border,
        3.0 * scale,
    );
    draw_text(
        svg,
        rect,
        label,
        &c.text.secondary,
        v.button_text_size * scale,
    );
}

fn draw_meter(
    svg: &mut String,
    vu: Rect,
    stereo: bool,
    c: &ColorScheme,
    v: &VisualTokens,
    scale: f32,
) {
    fill_rect(svg, vu, "#1e1f23", "#1e1f23", 2.0 * scale);
    let bars = if stereo { 2 } else { 1 };
    let gap = 2.0 * scale;
    let bar_w = if bars == 2 {
        (vu.width - gap) / 2.0
    } else {
        vu.width * MONO_BAR_RATIO
    };
    for index in 0..bars {
        let bx = if bars == 2 {
            vu.x + index as f32 * (bar_w + gap)
        } else {
            vu.centre_x() - bar_w / 2.0
        };
        let bar = Rect::new(bx, vu.y, bar_w, vu.height);
        let level = 0.72 - index as f32 * 0.08;
        let filled = Rect::new(
            bar.x,
            bar.y + bar.height * (1.0 - level),
            bar.width,
            bar.height * level,
        );
        let color = if level > 0.88 {
            &c.meter.red
        } else if level > 0.65 {
            &c.meter.yellow
        } else {
            &c.meter.green
        };
        let _ = write!(
            svg,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{color}"/>"#,
            x = filled.x,
            y = filled.y,
            w = filled.width,
            h = filled.height,
        );
    }
    for (db, label) in VU_LABELS {
        let t = db_to_t(db);
        let y = vu.bottom() - vu.height * t;
        let _ = write!(
            svg,
            r#"<text x="{x:.1}" y="{y:.1}" fill="{color}" font-family="sans-serif" font-size="{size:.1}" text-anchor="middle" dominant-baseline="middle">{label}</text>"#,
            x = vu.centre_x(),
            y = y,
            color = &c.text.primary,
            size = v.micro_size * scale,
        );
    }
}

fn draw_fader(
    svg: &mut String,
    track: Rect,
    vu: Rect,
    c: &ColorScheme,
    v: &VisualTokens,
    scale: f32,
) {
    let slot_w = v.fader_slot_w * scale;
    let slot = Rect::new(
        track.centre_x() - slot_w / 2.0,
        track.y,
        slot_w,
        track.height,
    );
    fill_rect(
        svg,
        slot,
        &c.control.fader_slot,
        &c.control.fader_slot,
        1.0 * scale,
    );
    let cap_centre = track.bottom() - track.height * db_to_t(0.0);
    let fill = Rect::new(slot.x, cap_centre, slot.width, slot.bottom() - cap_centre);
    fill_rect(svg, fill, &c.control.fader_fill, &c.control.fader_fill, 0.0);
    let cap = Rect::new(
        track.centre_x() - 11.0 * scale,
        cap_centre - v.fader_cap_h * scale / 2.0,
        v.fader_cap_w * scale,
        v.fader_cap_h * scale,
    );
    fill_rect(svg, cap, "#c9cbd1", "#8b8e96", 2.0 * scale);
    for i in 1..=3 {
        let y = cap.y + cap.height * i as f32 / 4.0;
        let (x, w, h) = if i == 2 {
            (cap.x, cap.width, (2.0 * scale).max(2.0))
        } else {
            (
                cap.x + 3.0 * scale,
                cap.width - 6.0 * scale,
                (1.0 * scale).max(1.0),
            )
        };
        let _ = write!(
            svg,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{color}"/>"#,
            x = x,
            y = y - h / 2.0,
            w = w,
            h = h,
            color = if i == 2 {
                &c.control.cap_centre
            } else {
                &c.control.cap_ridge
            },
        );
    }

    // Graduations sit to the left of the slot.  +3 / -3 are half length;
    // the white 0 dB mark is repeated at the right edge of the slot.
    let under = 5.0 * scale;
    let mark_w = ((track.x + under - vu.right()).max(1.0)) * 0.60;
    for db in [-60.0f32, -40.0, -20.0, -10.0, -6.0, -3.0, 0.0, 3.0] {
        let y = track.bottom() - track.height * db_to_t(db);
        let w = if db == 3.0 || db == -3.0 {
            mark_w * 0.5
        } else {
            mark_w
        };
        let color = if db == 3.0 {
            &c.meter.red
        } else if db == -3.0 {
            &c.meter.green
        } else if db == 0.0 {
            "#f2f3f5"
        } else {
            "#6b6d75"
        };
        let _ = write!(
            svg,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{color}"/>"#,
            x = track.x + under - w,
            y = y - 0.5 * scale,
            w = w,
            h = 1.0 * scale,
        );
    }
    let ref_y = track.bottom() - track.height * db_to_t(0.0);
    let right_x = track.centre_x() + slot_w / 2.0 + 1.0 * scale;
    let _ = write!(
        svg,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{color}"/>"#,
        x = right_x,
        y = ref_y - 0.5 * scale,
        w = 5.0 * scale,
        h = 1.0 * scale,
        color = "#f2f3f5",
    );
}

fn draw_fx(
    svg: &mut String,
    fx: Rect,
    kind: ChannelKind,
    c: &ColorScheme,
    v: &VisualTokens,
    scale: f32,
) {
    let fx_slot_w = v.fader_slot_w * scale;
    let fx_slot = Rect::new(fx.centre_x() - fx_slot_w / 2.0, fx.y, fx_slot_w, fx.height);
    fill_rect(
        svg,
        fx_slot,
        &c.control.fader_slot,
        &c.control.fader_slot,
        1.0 * scale,
    );
    let fx_cap_h = v.fx_cap_h * scale;
    let fx_cap_w = v.fx_cap_w * scale;
    let fx_cap_centre = fx.y + fx.height * 0.38;
    let fx_fill = Rect::new(
        fx_slot.x,
        fx_cap_centre,
        fx_slot.width,
        fx_slot.bottom() - fx_cap_centre,
    );
    let fx_color = if matches!(kind, ChannelKind::Output) {
        c.accent.cue.as_str()
    } else {
        "#ff9f43"
    };
    fill_rect(svg, fx_fill, fx_color, fx_color, 0.0);
    let fx_cap = Rect::new(
        fx.centre_x() - fx_cap_w / 2.0,
        fx_cap_centre - fx_cap_h / 2.0,
        fx_cap_w,
        fx_cap_h,
    );
    fill_rect(svg, fx_cap, "#c9cbd1", "#8b8e96", 1.0 * scale);
}

fn draw_tab(
    svg: &mut String,
    glyph: Rect,
    panel: Panel,
    c: &ColorScheme,
    v: &VisualTokens,
    scale: f32,
) {
    // At rest a tab shows only its glyph; the dark face is used for hover or
    // while its slideout is open.
    let (label, size) = match panel {
        Panel::Trim => ("T", v.tab_glyph_size),
        Panel::Settings => ("⚙", v.gear_glyph_size),
        Panel::Eq => ("EQ", v.tab_glyph_size),
        Panel::Dynamics => ("D", v.tab_glyph_size),
    };
    draw_text(svg, glyph, label, &c.text.secondary, size * scale);
}

fn draw_submix(svg: &mut String, bar: Rect, c: &ColorScheme, v: &VisualTokens, scale: f32) {
    fill_rect(
        svg,
        bar,
        &c.control.button_face,
        &c.surface.border,
        4.0 * scale,
    );
    draw_text(
        svg,
        bar,
        "Phones 7/8",
        &c.text.secondary,
        v.readout_size * scale,
    );
    let _ = write!(
        svg,
        r#"<text x="{x:.1}" y="{y:.1}" fill="{color}" font-family="sans-serif" font-size="{size:.1}" text-anchor="end" dominant-baseline="middle">▸</text>"#,
        x = bar.right() - 5.0 * scale,
        y = bar.centre_y(),
        color = &c.text.secondary,
        size = v.readout_size * scale,
    );
}

fn sample_name(id: TemplateId) -> &'static str {
    match id {
        TemplateId::InputMono => "AN1",
        TemplateId::InputStereo => "AN1/2",
        TemplateId::InputMonoNoFx => "AN3",
        TemplateId::PlaybackStereo => "SW1/2",
        TemplateId::OutputStereo => "OUT1/2",
        TemplateId::OutputMono => "OUT3",
        TemplateId::ControlRoom => "MAIN",
    }
}

fn db_to_t(db: f32) -> f32 {
    // Preview-only approximation of the shared fader scale; the exact
    // mapping lives with the fader widget.
    let floor = -65.0;
    let top = 6.0;
    let t = ((db - floor) / (top - floor)).clamp(0.0, 1.0);
    t.powf(0.85)
}

fn translated(rect: Rect, ox: f32, oy: f32) -> Rect {
    Rect::new(rect.x + ox, rect.y + oy, rect.width, rect.height)
}

fn fill_rect(svg: &mut String, r: Rect, fill: &str, stroke: &str, radius: f32) {
    let _ = write!(
        svg,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw:.1}" rx="{radius:.1}"/>"#,
        x = r.x,
        y = r.y,
        w = r.width,
        h = r.height,
        sw = 1.0,
        radius = radius,
    );
}

fn draw_text(svg: &mut String, r: Rect, text: &str, color: &str, size: f32) {
    draw_text_aligned(svg, r, text, color, size, "middle");
}

fn draw_text_aligned(svg: &mut String, r: Rect, text: &str, color: &str, size: f32, align: &str) {
    let x = match align {
        "end" => r.right() - 2.0,
        "start" => r.x + 2.0,
        _ => r.centre_x(),
    };
    let _ = write!(
        svg,
        r#"<text x="{x:.1}" y="{y:.1}" fill="{color}" font-family="sans-serif" font-size="{size:.1}" text-anchor="{align}" dominant-baseline="middle">{text}</text>"#,
        x = x,
        y = r.centre_y(),
        size = size,
    );
}
