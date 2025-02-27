use std::collections::HashMap;
use brightsky::{CurrentWeather, HourlyWeather};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};

pub type WeatherForecast = Vec<HourlyWeather>;

pub struct WeatherData<'a> {
    pub(crate) current_weather: &'a CurrentWeather,
    pub(crate) weather_forecast: &'a WeatherForecast
}

impl WeatherData<'_> {
    #[allow(dead_code)]
    /// Returns weekly and daily temperatures as (min, max, dailyminmax)
    /// Keys are simply the day of the month which will not overlap as long as the forecast is 5
    /// days (i.e. less than a full month)
    pub fn daily_minmax_temps(&self) -> HashMap<u32, (f32, f32)> {
        let mut day_min: f32 = 200.0;
        let mut day_max: f32 = -100.0;
        let mut daily_minmax: HashMap<u32, (f32, f32)> = HashMap::new();

        let mut current_day: u32 = self.weather_forecast[0].timestamp.day();
        for w in self.weather_forecast {
            let day = w.timestamp.day();
            let t = w.temperature.unwrap();

            if day != current_day {
                daily_minmax.insert(current_day, (day_min, day_max));
                day_min = 200.0;
                day_max = -100.0;
                current_day = day;
            }
            if t < day_min && w.timestamp.hour() > 12 {
                day_min = t;
            }
            if t > day_max {
                day_max = t;
            }
        }
        // below is commented to ignore update for the last day
        // if day != current_day {
        //     daily_minmax.insert(current_day, (day_min, day_max));
        // }

        daily_minmax
    }

    /// Returns min and max temps for the week
    pub fn week_minmax_temps(&self) -> (f32, f32) {
        let min_temp = self.weather_forecast.iter()
            .min_by_key(|w| w.temperature.unwrap() as i32).expect("no values in forecast").temperature.unwrap();
        let max_temp = self.weather_forecast.iter()
            .max_by_key(|w| w.temperature.unwrap() as i32).expect("no values in forecast").temperature.unwrap();
        (min_temp, max_temp)
    }

    pub fn week_max_rain(&self) -> f32 {
        self.weather_forecast.iter()
            .max_by_key(|w| w.precipitation.unwrap() as i32).expect("no values in forecast").precipitation.unwrap()
    }

    #[allow(dead_code)]
    pub fn smoother_forecast(&self) -> Vec<(DateTime<FixedOffset>, f32, f32)> {
        let mut smoothed_data = Vec::new();
    
        const WINDOW_SIZE: usize = 3;
        let data = self.weather_forecast;
    
        for i in 0..data.len() {
    
            // Skip by WINDOW_SIZE to make the result smooth and with less samples
            if i % WINDOW_SIZE != 0 {
                continue;
            }
    
            let mut sum_temp = 0.0;
            let mut sum_prep = 0.0;
            let window_start = if i >= WINDOW_SIZE / 2 { i - WINDOW_SIZE / 2 } else { 0 };
            let window_end = if i + WINDOW_SIZE / 2 < data.len() { i + WINDOW_SIZE / 2 } else { data.len() - 1 };
    
            for item in data[window_start..=window_end].iter() {
                sum_temp += item.temperature.unwrap();
                sum_prep += item.precipitation.unwrap();
            }
    
            let avg_temp = sum_temp / (window_end - window_start + 1) as f32;
            let avg_prep = sum_prep / (window_end - window_start + 1) as f32;
    
            smoothed_data.push((data[i].timestamp, avg_temp, avg_prep));
        }
    
        smoothed_data
    }
}
