//! The product artwork, decoded from the same PNGs the packaging installs
//! into the icon theme: the tile (for the window to wear, see
//! `main::window_icon`) and the bare mark (for the in-app lockup in the
//! top bar and the About dialog). One place decodes the assets, so the
//! UI and the packaging can never drift apart on which file is the icon.
//!
//! Decoding uses the `png` crate directly and feeds iced's codec-free
//! image widget (`image-without-codecs`, seven extra crates). The full
//! `image` feature would pull ~79 crates of decoders this app already
//! answers with two files and `decode()` below.

use std::sync::OnceLock;

use iced::widget::image::Handle;

const PNG: &[u8] = include_bytes!("../assets/totalcontrol-256.png");
/// The mark alone: the orange gain arc and white T without the tile,
/// rendered from `assets/totalcontrol-logo.svg` at 256 px.
const MARK_PNG: &[u8] = include_bytes!("../assets/totalcontrol-mark-256.png");

/// The tray icon at the sizes panels ask for: the input-gain badge on a
/// small black disc (`assets/totalcontrol-tray.svg`), so the orange arc and
/// white T keep their contrast on both dark and light panels.
const TRAY_PNGS: [&[u8]; 4] = [
    include_bytes!("../assets/totalcontrol-tray-16.png"),
    include_bytes!("../assets/totalcontrol-tray-22.png"),
    include_bytes!("../assets/totalcontrol-tray-32.png"),
    include_bytes!("../assets/totalcontrol-tray-48.png"),
];

/// Every tray size as RGBA8 plus dimensions; a size that cannot be decoded
/// is skipped.
pub fn tray_rgba() -> Vec<(u32, u32, Vec<u8>)> {
    TRAY_PNGS.iter().filter_map(|p| decode(p)).collect()
}

/// The icon as RGBA8 plus its dimensions, or `None` when the PNG cannot be
/// decoded. A missing icon is cosmetic, so callers draw nothing instead of
/// failing the launch.
pub fn rgba() -> Option<(u32, u32, Vec<u8>)> {
    decode(PNG)
}

/// The bare mark as a ready-to-draw handle plus its width/height ratio,
/// decoded once per process; later calls clone the handle, not the pixels.
pub fn mark() -> Option<(Handle, f32)> {
    static MARK: OnceLock<Option<(Handle, f32)>> = OnceLock::new();
    MARK.get_or_init(|| {
        let (width, height, pixels) = decode(MARK_PNG)?;
        let handle = Handle::from_rgba(width, height, pixels);
        Some((handle, width as f32 / height as f32))
    })
    .clone()
}

fn decode(png: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let decoder = png::Decoder::new(png);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((info.width, info.height, buf))
}

#[cfg(test)]
mod tests {
    use super::{decode, mark, rgba, MARK_PNG};

    /// The packaged tile PNG is the one asset the window cannot live
    /// without: a replacement in another format, or at another size,
    /// would make `window_icon()` return `None` and the window silently
    /// lose its icon.
    #[test]
    fn the_packaged_icon_decodes_as_256_rgba() {
        let (width, height, pixels) = rgba().expect("the packaged PNG must decode");
        assert_eq!((width, height), (256, 256));
        assert_eq!(pixels.len(), (width * height * 4) as usize);
    }

    #[test]
    fn the_tray_icon_has_all_four_sizes() {
        let sizes: Vec<u32> = super::tray_rgba().iter().map(|(w, _, _)| *w).collect();
        assert_eq!(sizes, vec![16, 22, 32, 48]);
    }

    /// The mark feeds the top bar and the About dialog: it must decode,
    /// be square, and contain exactly the white T and the orange gain arc.
    /// An empty or solid-black rasterization would paint a blank box or a
    /// black one into the lockup.
    #[test]
    fn the_packaged_mark_is_white_and_orange_on_transparency() {
        let (width, height, pixels) = decode(MARK_PNG).expect("the mark must decode");
        assert_eq!((width, height), (256, 256));
        let mut white = 0usize;
        let mut orange = 0usize;
        for px in pixels.chunks_exact(4) {
            // Only fully opaque pixels are checked: the soft track and the
            // antialiased rim are allowed to blend.
            if px[3] > 230 {
                let (r, g, b) = (px[0], px[1], px[2]);
                if r > 0xdc && g > 0xdc && b > 0xdc {
                    white += 1;
                } else if r > 0xd0 && (0x60..=0xc8).contains(&g) && b < 0x90 {
                    orange += 1;
                } else {
                    panic!("unexpected opaque mark pixel: {:?}", &px[..3]);
                }
            }
        }
        assert!(white > 1000, "the white T must be drawn, got {white} pixels");
        assert!(
            orange > 500,
            "the orange arc must be drawn, got {orange} pixels"
        );
    }

    /// `mark()` memoizes: the second call must hand back the same square
    /// ratio without touching the PNG again.
    #[test]
    fn mark_is_decoded_once_and_keeps_a_sane_ratio() {
        let (_, first) = mark().expect("the mark must decode");
        let (_, second) = mark().expect("second call");
        assert_eq!(first, second);
        assert!((first - 1.0).abs() < 0.001, "unexpected ratio {first}");
    }
}
