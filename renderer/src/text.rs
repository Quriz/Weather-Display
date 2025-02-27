use imageproc::drawing::Canvas;
use image::{RgbImage, Rgb};
use rusttype::{point, Font, PositionedGlyph, Rect, Scale};
use std::cmp::max;

// Code mostly taken wholesale from
// https://github.com/image-rs/imageproc/blob/master/src/drawing/text.rs

fn layout_glyphs(
    scale: Scale,
    font: &Font,
    text: &str,
    mut f: impl FnMut(PositionedGlyph, Rect<i32>),
) -> (i32, i32) {
    let v_metrics = font.v_metrics(scale);

    let (mut w, mut h) = (0, 0);

    for g in font.layout(text, scale, point(0.0, v_metrics.ascent)) {
        if let Some(bb) = g.pixel_bounding_box() {
            w = max(w, bb.max.x);
            h = max(h, bb.max.y);
            f(g, bb);
        }
    }

    (w, h)
}

pub fn measure_text(font: &Font, text: &str, font_size: f32) -> (f32, f32) {
    let font_size = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(font_size);

    let xpad = 0f32;
    let ypad = 0f32;

    let glyphs: Vec<_> = font
        .layout(text, font_size, point(xpad, ypad + v_metrics.ascent))
        .collect();

    let height = (v_metrics.ascent - v_metrics.descent).ceil();
    let width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as f32
    };

    (width, height)
}

/// Get the width and height of the given text, rendered with the given font and scale.
///
/// Note that this function *does not* support newlines, you must do this manually.
pub fn text_size(font: &Font, text: &str, scale: Scale) -> (f32, f32) {
    let (w, h) = layout_glyphs(scale, font, text, |_, _| {});
    (w as f32, h as f32)
}

pub fn adjust_scale_to_fit_box(
    box_width: f32,
    box_height: f32,
    line_spacing: f32,
    font: &Font,
    start_scale: f32,
    text: &str,
) -> f32 {
    let mut scale = start_scale;

    loop {
        // Calculate size of text with current scale
        let text_dimensions = get_text_wrapped_size( box_width, line_spacing, Scale::uniform(scale), font, text);

        // Check if text fits in the box
        if text_dimensions.0 <= box_width && text_dimensions.1 <= box_height {
            break; // Break if finished
        }
        else {
            scale -= 1.0;
        }

        // Cancel if text scale got too small
        if scale < 13.0 {
            println!("Text scale is too small. Returning {}", scale);
            break;
        }
    }

    scale
}

pub fn adjust_scale_to_fit_lines(
    line_width: f32,
    line_count: u32,
    font: &Font,
    start_scale: f32,
    text: &str,
) -> f32 {
    let mut scale = start_scale;

    loop {
        // Calculate size of text with current scale
        let text_line_count = get_text_wrapped_line_count(line_width, Scale::uniform(scale), font, text);

        // Check if text fits in line count
        if text_line_count <= line_count {
            break; // Break if finished
        }
        else {
            scale -= 1.0;
        }

        // Cancel if text scale got too small
        if scale < 15.0 {
            println!("Text scale is too small. Returning {}. Target line_count: {}, got: {}", scale, line_count, text_line_count);
            break;
        }
    }

    scale
}

/// Draws colored text on an image in place.
///
/// `scale` is augmented font scaling on both the x and y axis (in pixels).
///
/// Note that this function *does not* support newlines, you must do this manually.
pub fn draw_text_mut<'a>(
    canvas: &'a mut RgbImage,
    color: Rgb<u8>,
    x: i32,
    y: i32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) where
{
    let image_width = canvas.width() as i32;
    let image_height = canvas.height() as i32;

    layout_glyphs(scale, font, text, |g, bb| {
        g.draw(|gx, gy, gv| {
            let gx = gx as i32 + bb.min.x;
            let gy = gy as i32 + bb.min.y;

            let image_x = gx + x;
            let image_y = gy + y;

            if (0..image_width).contains(&image_x) && (0..image_height).contains(&image_y) {

                // code edited here from original, if there's any coverage just make it uniformly
                // the same color, else don't draw
                if gv > 0.1 {
                    canvas.draw_pixel(image_x as u32, image_y as u32, color);
                }
            }
        })
    });
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_wrapped<'a>(
    image: &'a mut RgbImage,
    color: Rgb<u8>,
    x: f32,
    y: f32,
    max_width: f32,
    line_spacing: f32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) -> (f32, f32) {
    let mut current_x = x;
    let mut current_y = y;
    let mut max_line_width = 0f32;
    let mut total_height = 0f32;

    for word in text.split_whitespace() {
        let (word_width, word_height) = text_size(font, word, scale);

        // If adding the new word exceeds the max width, wrap to a new line.
        if current_x + word_width > x + max_width {
            current_x = x; // Reset X position to the initial X for a new line
            current_y += word_height + line_spacing; // Move Y position down by one line
        }

        // Draw the word on the image
        draw_text_mut(image, color, current_x as i32, current_y as i32, scale, font, word);

        // Update the current X position for the next word
        current_x += text_size(font, format!(" {}", word).as_str(), scale).0; // Add a space after each word

        // Keep track of the maximum line width
        max_line_width = max_line_width.max(current_x);

        // Keep track of the total height
        total_height = current_y + word_height - y;
    }

    (max_line_width, total_height)
}

fn get_text_wrapped_size<'a>(
    max_width: f32,
    line_spacing: f32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) -> (f32, f32) {
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut max_line_width = 0f32;
    let mut total_height = 0f32;

    for word in text.split_whitespace() {
        let (word_width, word_height) = text_size(font, word, scale);

        // If adding the new word exceeds the max width, wrap to a new line.
        if current_x + word_width > max_width {
            current_x = 0.0; // Reset X position to the initial X for a new line
            current_y += word_height + line_spacing; // Move Y position down by one line
        }

        // Update the current X position for the next word
        current_x += text_size(font, format!(" {}", word).as_str(), scale).0; // Add a space after each word

        // Keep track of the maximum line width
        max_line_width = max_line_width.max(current_x);

        // Keep track of the total height
        total_height = current_y + word_height;
    }

    (max_line_width, total_height)
}

fn get_text_wrapped_line_count<'a>(
    max_width: f32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) -> u32 {
    let mut current_x = 0.0;
    let mut line_count = 1;

    for word in text.split_whitespace() {
        let (word_width, _word_height) = text_size(font, word, scale);

        // If adding the new word exceeds the max width, wrap to a new line.
        if current_x + word_width > max_width {
            current_x = 0.0; // Reset X position to the initial X for a new line
            line_count += 1;
        }

        // Update the current X position for the next word
        current_x += text_size(font, format!(" {}", word).as_str(), scale).0; // Add a space after each word
    }

    line_count
}


