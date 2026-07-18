use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use anyhow::{Context, Result};
use image::RgbaImage;

const ICON_WIDTH: u32 = 64;
const ICON_HEIGHT: u32 = 24;
const FONT_SIZE_PX: f32 = 12.0;
const BG_COLOR: [u8; 4] = [30, 30, 30, 255];
const TEXT_COLOR: [u8; 4] = [255, 255, 255, 255];
const WARNING_COLOR: [u8; 4] = [255, 200, 50, 255];

pub struct RenderedIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Render a balance amount as a tray icon.
pub fn render_balance_icon(text: &str) -> Result<RenderedIcon> {
    render_text_icon(text, TEXT_COLOR, BG_COLOR)
}

/// Render a status indicator as a tray icon.
pub fn render_status_icon(text: &str) -> Result<RenderedIcon> {
    render_text_icon(text, WARNING_COLOR, BG_COLOR)
}

fn render_text_icon(text: &str, fg: [u8; 4], bg: [u8; 4]) -> Result<RenderedIcon> {
    let font_data: &[u8] = include_bytes!("../assets/DejaVuSansMono.ttf");
    let font = FontRef::try_from_slice(font_data)
        .context("加载字体失败")?;

    let px_scale = PxScale::from(FONT_SIZE_PX);
    let scaled = font.as_scaled(px_scale);

    // Compute the pixel width of the text
    let text_width: f32 = text
        .chars()
        .map(|ch| scaled.h_advance(scaled.glyph_id(ch)))
        .sum();
    let text_width = text_width.ceil() as u32;

    let ascent = scaled.ascent().ceil() as u32;
    let descent = scaled.descent().ceil() as i32;
    let line_height = ascent + descent.max(0) as u32;

    let mut img = RgbaImage::from_pixel(ICON_WIDTH, ICON_HEIGHT, image::Rgba(bg));

    // Center text horizontally
    let offset_x = ((ICON_WIDTH.saturating_sub(text_width)) / 2) as f32;
    // Center text vertically
    let offset_y = ((ICON_HEIGHT.saturating_sub(line_height)) / 2) as f32 + ascent as f32;

    let mut cursor: f32 = offset_x;
    for ch in text.chars() {
        let glyph_id = scaled.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(px_scale, (cursor, offset_y));
        if let Some(outline) = scaled.outline_glyph(glyph) {
            outline.draw(|x, y, v| {
                let px = x as u32;
                let py = y as u32;
                if px < ICON_WIDTH && py < ICON_HEIGHT {
                    let alpha = (v * 255.0).min(255.0) as u8;
                    *img.get_pixel_mut(px, py) =
                        image::Rgba(blend_over(bg, [fg[0], fg[1], fg[2], alpha]));
                }
            });
        }
        cursor += scaled.h_advance(glyph_id);
    }

    Ok(RenderedIcon {
        rgba: img.into_raw(),
        width: ICON_WIDTH,
        height: ICON_HEIGHT,
    })
}

fn blend_over(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let a_f = fg[3] as f32 / 255.0;
    let a_b = bg[3] as f32 / 255.0;
    let a_out = a_f + a_b * (1.0 - a_f);
    if a_out < 0.001 {
        return [0, 0, 0, 0];
    }
    [
        ((fg[0] as f32 * a_f + bg[0] as f32 * a_b * (1.0 - a_f)) / a_out) as u8,
        ((fg[1] as f32 * a_f + bg[1] as f32 * a_b * (1.0 - a_f)) / a_out) as u8,
        ((fg[2] as f32 * a_f + bg[2] as f32 * a_b * (1.0 - a_f)) / a_out) as u8,
        (a_out * 255.0) as u8,
    ]
}
