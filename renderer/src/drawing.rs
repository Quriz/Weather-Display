use chrono::prelude::*;
use reqwest::blocking::Client;
use image::imageops::FilterType;

use imageproc::drawing::{Canvas, draw_line_segment_mut, BresenhamLineIter};
use image::{RgbImage, Rgb};
use rusttype::{Font, Scale};
use epd_waveshare::epd7in5b_v2::{WIDTH as EPD_WIDTH, HEIGHT as EPD_HEIGHT};

use crate::text::{draw_text_mut, measure_text, draw_text_wrapped, adjust_scale_to_fit_box, adjust_scale_to_fit_lines};
use crate::weather::WeatherData;
use crate::dithering::*;
use crate::config::CONFIG;

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);

/// Draw weather graph with temperature and rain
pub fn draw_graph(
    weather_data: &WeatherData, 
    min_temp_scale: i32, 
    max_temp_scale: i32, 
    max_rain_scale: i32, 
    width: i64, 
    height: i64, 
    font: &Font
) -> RgbImage {
    let mut image = RgbImage::from_fn(width as u32, height as u32, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });

    let height = height as f32;
    let width = width as f32;

    let date_time: DateTime<FixedOffset> = chrono::Local::now().into();

    let forecast_data = &weather_data.weather_forecast;
    let horizontal_spacing = (width-1.0) / (forecast_data.len()-1) as f32;

    // Convert rain probabilities 0-100 into pixel heights.
    // Convert temp into pixel heights based on max and min temps.
    // Y axis points down so we subtract from height.

    // Compute the pixel Y coordinate for each graph's data points based on the total height of the graph region.
    // The points are computed by scaling each point relative to the overall height, and for the
    // temperature graph, the min and max temperature. 
    // The rain precipitation is in mm so we just scale from 0 to max rain. 
    // There's a bit of tricky off by one stuff as well.
    let temp_x = (0..forecast_data.len()).map(|x| horizontal_spacing * x as f32);
    let temp_y = forecast_data.iter().map(|w| {
        height - ((w.temperature.unwrap() - min_temp_scale as f32) / (max_temp_scale - min_temp_scale) as f32) * (height - 1.0).floor() - 1.0
    });
    let rain_x = (0..forecast_data.len()).map(|x| horizontal_spacing * x as f32);
    let rain_y = forecast_data.iter().map(|w| {
        (height - ((w.precipitation.unwrap() / max_rain_scale as f32) * (height - 1.0)).floor() - 1.0).clamp(0.0, height)
    });

    let dates = forecast_data.iter().map(|w| w.timestamp);

    let temp_points: Vec<(f32, f32)> = temp_x.zip(temp_y).collect();
    let rain_points: Vec<(DateTime<_>, (f32, f32))> =
        dates.zip(rain_x.zip(rain_y)).collect();

    // draw vertical lines with legends to split into days of the week
    for (i, w) in rain_points.windows(2).enumerate() {
        let d1 = &w[0].0;
        let d2 = &w[1].0;
        let x = w[1].1.0;

        // Draw line for current time of day
        if d1.day() == date_time.day() && d1.hour() <= date_time.hour() && d2.hour() > date_time.hour() {
            draw_line_segment_dotted_mut(&mut image, (x, 0.0), (x, height), BLACK);
        }

        // Draw day separator
        if d1.day() != d2.day() {
            draw_line_segment_mut(&mut image, (x, 0.0), (x, height), BLACK);
        }
        // Draw weekday
        if d1.day() != d2.day() || i == 0 {
            let text = match d2.weekday() {
                Weekday::Mon => &CONFIG.weekday_names.monday,
                Weekday::Tue => &CONFIG.weekday_names.tuesday,
                Weekday::Wed => &CONFIG.weekday_names.wednesday,
                Weekday::Thu => &CONFIG.weekday_names.thursday,
                Weekday::Fri => &CONFIG.weekday_names.friday,
                Weekday::Sat => &CONFIG.weekday_names.saturday,
                Weekday::Sun => &CONFIG.weekday_names.sunday,
            };
            draw_text_left(&mut image, text, x + 5.0, 0.0, font, 24.0, BLACK);
        }
    }

    // draw the actual graph, iterating over the pixels directly so that we can keep track of the
    // heights of the graph for shading in under the graph later.
    let mut max_y = vec![None; width as usize];
    for w in rain_points.windows(2) {
        let p1 = w[0].1;
        let p2 = w[1].1;
        for (x, y) in BresenhamLineIter::new(p1, p2) {
            let x = x.clamp(0, width as i32) as usize;
            let y = y as u32;

            if max_y[x].is_none() || max_y[x].filter(|ym| y > *ym).is_some() {
                max_y[x] = Some(y);
            }
            image.draw_pixel(x as u32, y, Rgb([0u8,0u8,0u8]));
        }
    }

    // using some math that took me a bit to get correct, draw a nice little shading pattern under
    // the graph, using the above max_y to determine the limits of the pattern
    for (x, y, p) in image.enumerate_pixels_mut() {
        let ym = y%6;
        let yd = y/6;
        let xm = (x+2*yd)%6;
        let xcond = (xm == ym) || (xm == ym+1);
        let ycond = ym < 3;

        let g_cond = max_y[x as usize].filter(|ym| y > *ym).is_some();

        if xcond && ycond && g_cond {
            *p = Rgb([0u8,0u8,0u8]);
        }
    }

    // Draw the actual temperature graph last so it goes on top of everything
    for w in temp_points.windows(2) {
        let p1 = w[0];
        let p2 = w[1];
        draw_line_segment_mut(&mut image, (p1.0, p1.1 + 1.0), (p2.0, p2.1 + 1.0), RED);
        draw_line_segment_mut(&mut image, p1, p2, RED);
        draw_line_segment_mut(&mut image, (p1.0, p1.1 - 1.0), (p2.0, p2.1 - 1.0), RED);
    }

    image
}

pub fn draw_meme(image: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>, font: &Font<'_>, article: Option<knowyourmeme::Article>, pos_y: i64) {
    if article.is_none() {
        return;
    }

    const PADDING: i64 = 15;

    let pos_y = pos_y + PADDING;

    let article = article.unwrap();

    let article_img_x = PADDING;
    let article_img_height = (EPD_HEIGHT as i64 - pos_y - PADDING) as u32;
    let article_img_width = (article_img_height as f32 * 1.76) as u32;

    // Download article image
    let response = Client::new().get(article.image_url.as_str()).send().unwrap();
    let bytes = response.bytes().unwrap();
    let article_image = image::load_from_memory(&bytes).unwrap();
    let article_image = article_image.resize_exact(article_img_width, article_img_height, FilterType::Nearest);

    // Dither image
    let article_image = dither_image_grayscale(article_image);
    image::imageops::overlay(image, &article_image, article_img_x, pos_y);

    // General values
    let text_x = (article_img_x + article_img_width as i64 + PADDING) as f32;
    let text_max_width = EPD_WIDTH as f32 - text_x - PADDING as f32;

    // Title
    let title_text = article.title.as_str();
    let title_y = pos_y as f32;
    let title_font_size = adjust_scale_to_fit_lines(text_max_width, 2, font, 28.0, title_text);
    let (_title_width, title_height) = draw_text_left_wrapped(image, title_text, text_x, title_y, text_max_width, 5.0, font, title_font_size, BLACK);

    // Meme name
    let meme_name_y = title_y + title_height + 5.0;
    let mut meme_name_height = 0.0;
    if article.meme_name.is_some() {
        let text = article.meme_name.as_ref().unwrap().as_str();

        let font_size = (title_font_size - 8.0).max(13.0);

        let (_width, height) = draw_text_left_wrapped(image, text, text_x, meme_name_y, text_max_width, 4.0, font, font_size, BLACK);
        meme_name_height = height + 3.0;
    }

    // Summary
    let summary_text = article.summary.as_str();
    let summary_y = meme_name_y + meme_name_height + 8.0;
    let summary_max_height = article_img_height as f32 - 10.0;
    let summary_spacing = 3.0;
    let summary_font_size = adjust_scale_to_fit_box(text_max_width, summary_max_height, summary_spacing, font, title_font_size - 4.0, summary_text);
    draw_text_left_wrapped(image, summary_text, text_x, summary_y, text_max_width, summary_spacing, font, summary_font_size, BLACK);
}

// Based on draw_line_segment_mut()
fn draw_line_segment_dotted_mut<C>(canvas: &mut C, start: (f32, f32), end: (f32, f32), color: C::Pixel)
where
    C: Canvas,
{
    let (width, height) = canvas.dimensions();
    let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

    let line_iterator = BresenhamLineIter::new(start, end);

    for (i, point) in line_iterator.enumerate() {
        let x = point.0;
        let y = point.1;

        if i % 4 == 0 && in_bounds(x, y) {
            canvas.draw_pixel(x as u32, y as u32, color);
        }
    }
}

pub fn draw_text_left(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, x as i32, y as i32, scale, font, text);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_left_wrapped(image: &mut RgbImage, text: &str, x: f32, y: f32, max_width: f32, line_spacing: f32, font: &Font, scale: f32, color: Rgb<u8>) -> (f32, f32) {
    let scale = Scale::uniform(scale);
    
    draw_text_wrapped(image, color, x, y, max_width, line_spacing, scale, font, text)
}

pub fn draw_text_right(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (text_width, _text_height) = measure_text(font, text, scale);
    let scale = Scale::uniform(scale);
    let text_x = x - text_width;

    draw_text_mut(image, color, text_x as i32, y as i32, scale, font, text);
}

pub fn draw_text_bottom(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (_text_width, text_height) = measure_text(font, text, scale);
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, x as i32, (y-text_height) as i32, scale, font, text);
}

pub fn draw_text_bottom_right(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (text_width, text_height) = measure_text(font, text, scale);
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, (x-text_width) as i32, (y-text_height) as i32, scale, font, text);
}
