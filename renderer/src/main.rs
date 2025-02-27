use std::thread;
use chrono::{DateTime, Datelike, Duration, FixedOffset, Local, Timelike, Utc};
use reqwest::Url;
use reqwest::blocking::Client;
use anyhow::Result;
use brightsky::{self, CurrentWeather, HourlyWeather};

mod config;
use config::CONFIG;
mod weather;
use weather::WeatherData;
mod text;
mod dithering;
mod drawing;
mod rendering;

pub struct DisplayData<'a> {
    weather: WeatherData<'a>,
    kym_article: Option<knowyourmeme::Article>
}

fn main() -> Result<()> {
    let date_time: DateTime<FixedOffset> = Local::now().into();

    println!("Getting weather data...");
    let mut current_weather = brightsky::get_current_weather(&CONFIG.location, &CONFIG.timezone)?;
    let mut weather_forecast = brightsky::get_weather_forecast(&date_time, 5, &CONFIG.location, &CONFIG.timezone);

    let mut last_date_time = date_time;
    let mut last_current_weather = current_weather.clone();
    let mut last_weather_forecast = weather_forecast.clone();

    let mut last_kym_article = None;

    loop {
        println!("Getting Know Your Meme article...");
        let mut kym_article = knowyourmeme::get_newest_meme_article().ok();

        // Use last article if current article is invalid
        // or save current article for later usage
        if kym_article.is_none() {
            kym_article = last_kym_article.clone();
            println!("Couldn't find a new Know Your Meme article. Using last article.");
        } else {
            last_kym_article = kym_article.clone();
        }

        // Render image
        render(&current_weather, &weather_forecast, kym_article)?;

        wait_until_next_hour()?;
        
        println!();

        // Get new data
        let date_time: DateTime<FixedOffset> = chrono::Local::now().into();

        println!("Getting weather data...");
        current_weather = match brightsky::get_current_weather(&CONFIG.location, &CONFIG.timezone) {
            Ok(w) => w,
            Err(_) => last_current_weather.clone(),
        };

        weather_forecast = brightsky::get_weather_forecast(&date_time, 5, &CONFIG.location, &CONFIG.timezone);
        // Get last entry if current is invalid
        if weather_forecast.len() < 4 {
            println!("Error: Weather forecast has less than 4 entries. Using last weather forecast data.");
            
            if date_time.day() != last_date_time.day() {
                last_weather_forecast.remove(0); // Remove last day so that new day is the first
            }

            weather_forecast = last_weather_forecast.clone();
        }

        // Update last data
        last_date_time = date_time;
        last_current_weather = current_weather.clone();
        last_weather_forecast = weather_forecast.clone();
    }
}

fn render(current_weather: &CurrentWeather, weather_forecast: &Vec<HourlyWeather>, kym_article: Option<knowyourmeme::Article>) -> Result<()> {
    let display_data = DisplayData {
        weather: WeatherData {
            current_weather,
            weather_forecast,
        },
        kym_article
    };

    println!("Rendering image...");
    let image_buffer = rendering::render_image(display_data)?;

    println!("Uploading image...");
    let base_url = Url::parse(CONFIG.webdav_url)?;
    let image_url = base_url.join(CONFIG.image_name)?;
    let image_url_str = image_url.to_string();

    let response = Client::new().put(image_url).body(image_buffer).send()?;

    if response.status().is_success() {
        println!("Image upload successful: {}", image_url_str);
    } else {
        println!("Couldn't upload picture. Status code: {}", response.status());
    }
    Ok(())
}

fn wait_until_next_hour() -> Result<()> {
    let now = Utc::now().with_timezone(&CONFIG.timezone);
    
    let next_hour = now.checked_add_signed(Duration::hours(1)).unwrap();
    let next_hour = next_hour.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
    let next_hour_seconds = next_hour.signed_duration_since(now).to_std()?;

    // Wait until next hour starts
    println!("Waiting {}s ({:?})", next_hour_seconds.as_secs(), next_hour);
    thread::sleep(next_hour_seconds);

    Ok(())
}