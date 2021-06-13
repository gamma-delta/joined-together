use crate::{assets::Assets, ASPECT_RATIO, HEIGHT, WIDTH};

use macroquad::prelude::*;

/// Make a Color from an RRGGBBAA hex code.
pub fn hexcolor(code: u32) -> Color {
    let [r, g, b, a] = code.to_be_bytes();
    Color::from_rgba(r, g, b, a)
}

pub fn mouse_position_pixel() -> (f32, f32) {
    let (mx, my) = mouse_position();
    let (wd, hd) = width_height_deficit();
    let mx = (mx - wd / 2.0) / ((screen_width() - wd) / WIDTH);
    let my = (my - hd / 2.0) / ((screen_height() - hd) / HEIGHT);
    (mx, my)
}

pub fn width_height_deficit() -> (f32, f32) {
    if (screen_width() / screen_height()) > ASPECT_RATIO {
        // it's too wide! put bars on the sides!
        // the height becomes the authority on how wide to draw
        let expected_width = screen_height() * ASPECT_RATIO;
        (screen_width() - expected_width, 0.0f32)
    } else {
        // it's too tall! put bars on the ends!
        // the width is the authority
        let expected_height = screen_width() / ASPECT_RATIO;
        (0.0f32, screen_height() - expected_height)
    }
}
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Draw the text with the given size at the given position.
pub fn text(font: Font, text: &str, color: Color, size: u16, cx: f32, cy: f32, align: TextAlign) {
    let params = TextParams {
        font_size: size,
        font,
        color,
        ..Default::default()
    };
    for (idx, line) in text.lines().enumerate() {
        let offset = match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
        };
        let width = measure_text(line, Some(font), size, 1.).width;
        draw_text_ex(
            line,
            cx - offset * width,
            cy + size as f32 * idx as f32,
            params,
        );
    }
}

/// Draw the given text using the monospace font.
///
/// `cx` and `cy` refer to the upper-left corner.
///
/// Newlines are handled properly. If `max_width` is given, the text will wrap on this too.
pub(crate) fn pixel_text<S: AsRef<str>>(
    text: S,
    cx: f32,
    cy: f32,
    max_width: Option<usize>,
    color: Color,
    assets: &Assets,
) {
    let mut row = 0;
    let mut col = 0;
    // additional delta-y in pixels
    let mut addl_dy = 0.0;

    for c in text.as_ref().bytes() {
        // font atlas is 16x6 chars
        let char_idx = match c {
            b' '..=b'~' => c - 0x20,
            b'\n' => {
                row = 0;
                col += 1;
                continue;
            }
            b'\r' => {
                // Carriage return shifts down by one pixel.
                addl_dy += 1.0;
                continue;
            }
            // Out of bounds
            _ => b'~' + 1,
        };

        let x = cx + (row * 4) as f32;
        let y = cy + (col * 6) as f32 + addl_dy;
        draw_texture_ex(
            assets.textures.font,
            x,
            y,
            color,
            DrawTextureParams {
                source: Some(Rect {
                    x: (char_idx % 16) as f32 * 3.0,
                    y: (char_idx / 16) as f32 * 5.0,
                    w: 3.0,
                    h: 5.0,
                }),
                ..Default::default()
            },
        );

        row += 1;
        if matches!(max_width, Some(max_width) if row >= max_width) {
            row = 0;
            col += 1;
        }
    }
}
/// Draw a 9patch of a 3x3 grid of tiles.
pub fn patch9(
    tile_size: f32,
    corner_x: f32,
    corner_y: f32,
    width: usize,
    height: usize,
    tex: Texture2D,
) {
    for x in 0..width {
        for y in 0..height {
            let px = corner_x + x as f32 * tile_size;
            let py = corner_y + y as f32 * tile_size;

            let sx = tile_size
                * if x == 0 {
                    0.0
                } else if x == width - 1 {
                    2.0
                } else {
                    1.0
                };
            let sy = tile_size
                * if y == 0 {
                    0.0
                } else if y == height - 1 {
                    2.0
                } else {
                    1.0
                };

            draw_texture_ex(
                tex,
                px,
                py,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(sx, sy, 16.0, 16.0)),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn draw_space(assets: &Assets) {
    use macroquad::prelude::*;
    gl_use_material(assets.shaders.space);
    assets
        .shaders
        .space
        .set_uniform("time", macroquad::time::get_time() as f32);

    draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, BLACK);

    gl_use_default_material();
}
