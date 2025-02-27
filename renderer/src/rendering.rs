use std::cmp::{min, max};
use chrono::Timelike;
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use rusttype::Font;
use embedded_graphics::prelude::*;
use epd_waveshare::{
    color::*,
    epd7in5b_v2::{WIDTH as EPD_WIDTH, HEIGHT as EPD_HEIGHT},
    graphics::VarDisplay,
};
use epd_waveshare::buffer_len;
use brightsky::Condition;

use crate::text::measure_text;
use crate::DisplayData;
use crate::drawing::*;
use crate::config::CONFIG;

pub type EpdBuffer = Vec<u8>;

const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/Comfortaa-Regular.ttf");
const HUMIDITY_ICON_DATA: &[u8] = include_bytes!("../assets/img/humidity.png");
const WIND_ICON_DATA: &[u8] = include_bytes!("../assets/img/wind.png");

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);

pub fn render_image(display_data: DisplayData) -> Result<EpdBuffer> {
    let font = Font::try_from_bytes(FONT_DATA).expect("Failed to open font");

    let graph_x = 50i64;
    let graph_y = 100i64;
    let graph_width = 700i64;
    let graph_height = 200i64;
    let graph_temp_text_x = graph_x as f32 - 10.0;
    let graph_rain_text_x = (graph_x + graph_width + 10) as f32;
    let graph_text_y = graph_y as f32;

    let weather = &display_data.weather;
    let current_weather = &weather.current_weather;

    let (min_temp, max_temp) = weather.week_minmax_temps();
    // Scale the temp values so that the temperature graph doesn't go right to the border
    let min_temp_scale = min((0.8 * min_temp).floor() as i32, 0);
    let max_temp_scale = max((1.2 * max_temp).ceil() as i32, 20);

    let max_rain = weather.week_max_rain();
    let max_rain_scale = max((1.2 * max_rain).ceil() as i32, 5);

    let graph = draw_graph(weather, min_temp_scale, max_temp_scale, max_rain_scale, graph_width, graph_height, &font);
    //let daily_temps = weather.daily_minmax_temps();

    let mut image = RgbImage::from_fn(800, 480, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });

    let current_temp = current_weather.temperature.unwrap();
    let temp_x = 10.0;
    let temp_y = 0.0;
    let temp_size = 100.0;
    let temp_text = format!("{}°", current_temp);
    let temp_color = if current_temp < 27.0 { BLACK } else { RED };

    let mut current_time = chrono::Utc::now().with_timezone(&CONFIG.timezone);
    if current_time.minute() > 30 {
        current_time = current_time.with_hour((current_time.hour() + 1) % 23).unwrap();
    }

    // Try to get todays highest and lowest temperature
    // let current_day = current_time.day();
    // let mut today_temps_text = String::new();
    // match daily_temps.get(&current_day) {
    //     Some((today_low, today_high)) => {
    //         today_temps_text = format!("{}° bis {}°", today_low, today_high);
    //     }
    //     None => {
    //         println!("No daily_temps entry found for key {}", current_day);
    //     }
    // }

    let time_text = format!("{}", current_time.format(CONFIG.time_format));

    let (temp_width, temp_height) = measure_text(&font, &temp_text, temp_size);
    let desc_x = temp_x + temp_width + 20.0;
    let desc_y = temp_y + temp_height / 2.0;

    let temp_min_text = if min_temp_scale < 0 { min_temp.round().to_string() } else { min_temp_scale.to_string() };
    let temp_max_text = if max_temp_scale > 20 { max_temp.round().to_string() } else { max_temp_scale.to_string() };
    let rain_min_text = "0";
    let rain_max_text = if max_rain_scale > 5 { max_rain.round().to_string() } else { max_rain_scale.to_string() };

    let condition = weather.current_weather.condition.as_ref().unwrap_or(&Condition::Null);

    let condition_text = match condition {
        Condition::Dry => &CONFIG.weather_conditions.dry,
        Condition::Fog => &CONFIG.weather_conditions.fog,
        Condition::Rain => &CONFIG.weather_conditions.rain,
        Condition::Sleet => &CONFIG.weather_conditions.sleet,
        Condition::Snow => &CONFIG.weather_conditions.snow,
        Condition::Hail => &CONFIG.weather_conditions.hail,
        Condition::Thunderstorm => &CONFIG.weather_conditions.thunderstorm,
        Condition::Null => &CONFIG.weather_conditions.null,
    }.to_string();

    let humidity_icon = load_icon(HUMIDITY_ICON_DATA)?;
    let humidity_icon_x = desc_x - 5.0;
    let humidity_icon_y = 7.0;

    let humidity = weather.current_weather.relative_humidity;
    let humidity_text = format!("{}%", humidity.unwrap_or(0.0));
    let humidity_text_size = measure_text(&font, &humidity_text, 32.0);
    let humidity_x = if humidity.is_some() { humidity_icon_x + humidity_icon.width() as f32 + 2.0 } else { 0.0 };
    let humidity_y = 10.0;

    let wind_icon = load_icon(WIND_ICON_DATA)?;
    let wind_icon_x = humidity_x + humidity_text_size.0 + 15.0;
    let wind_icon_y = 8.0;

    let wind = weather.current_weather.wind_speed_30;
    let wind_text = format!("{}km/h", wind.unwrap_or(0.0).round());
    let wind_x = wind_icon_x + wind_icon.width() as f32 + 5.0;
    let wind_y = 10.0;

    // Current temperature
    draw_text_left(&mut image, &temp_text, temp_x, temp_y, &font, temp_size, temp_color);

    // Weather condition text
    draw_text_left(&mut image, &condition_text, desc_x, desc_y, &font, 36.0, BLACK);

    // Current humidity
    if humidity.is_some() {
        image::imageops::overlay(&mut image, &humidity_icon, humidity_icon_x as i64, humidity_icon_y as i64);
        draw_text_left(&mut image, &humidity_text, humidity_x, humidity_y, &font, 32.0, BLACK);
    }

    // Current wind
    if wind.is_some() {
        image::imageops::overlay(&mut image, &wind_icon, wind_icon_x as i64, wind_icon_y as i64);
        draw_text_left(&mut image, &wind_text, wind_x, wind_y, &font, 32.0, BLACK);
    }

    // Current date and time
    draw_text_right(&mut image, &time_text, 790.0, 10.0, &font, 36.0, BLACK);

    // Graph temp min max
    draw_text_right(&mut image, &temp_max_text, graph_temp_text_x, graph_text_y + 18.0, &font, 24.0, RED);
    draw_text_bottom_right(&mut image, &temp_min_text, graph_temp_text_x, graph_text_y + graph_height as f32, &font, 24.0, RED);

    // Graph rain min max
    draw_text_left(&mut image, &rain_max_text, graph_rain_text_x, graph_text_y + 18.0, &font, 24.0, BLACK);
    draw_text_bottom(&mut image, rain_min_text, graph_rain_text_x, graph_text_y + graph_height as f32, &font, 24.0, BLACK);

    // Draw graph
    image::imageops::overlay(&mut image, &graph, graph_x, graph_y);

    let meme_y = graph_y + graph_height;
    draw_meme(&mut image, &font, display_data.kym_article, meme_y);
    
    // Save image as PNG in debug mode
    if cfg!(debug_assertions) {
        image.save("output.png").context("Couldn't save image")?;
    }

    let epd_buffer = rgb_image_to_epd_image(image);
    Ok(epd_buffer)
}

fn load_icon(data: &[u8]) -> Result<RgbImage> {
    let img = image::load_from_memory(data)?.to_rgb8();
    Ok(img)
}

fn rgb_image_to_epd_image(image: RgbImage) -> EpdBuffer {
    let mut buffer = vec![TriColor::White.get_byte_value(); buffer_len(EPD_WIDTH as usize, 2 * EPD_HEIGHT as usize)];
    let mut display = VarDisplay::<TriColor>::new(EPD_WIDTH, EPD_HEIGHT, &mut buffer, true).expect("Failed to create display");

    for (x, y, p) in image.enumerate_pixels() {
        let pt = Point::new(x as i32, y as i32);
        match *p {
            WHITE => display.set_pixel(Pixel(pt, TriColor::White)),
            BLACK => display.set_pixel(Pixel(pt, TriColor::Black)),
            RED => display.set_pixel(Pixel(pt, TriColor::Chromatic)),
            _ => display.set_pixel(Pixel(pt, TriColor::White)),
        }
    }

    buffer
}
