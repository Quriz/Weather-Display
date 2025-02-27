use anyhow::Result;
use chrono::{DateTime, Duration, FixedOffset};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct LatLon {
    pub lat: f32,
    pub lon: f32
}

#[derive(Deserialize)]
struct CurrentWeatherResponse {
    weather: CurrentWeather
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CurrentWeather {
    /// ISO 8601-formatted timestamp of this weather record
    #[serde(with = "date_serde")]
    pub timestamp: DateTime<FixedOffset>,

    /// Bright Sky source ID for this record
    pub source_id: i32,

    /// Total cloud cover at timestamp
    ///
    /// DWD Unit: %
    pub cloud_cover: Option<f32>,

    /// Current weather conditions. Unlike the numerical parameters, this field is not taken as-is from the raw data (because it does not exist),
    /// but is calculated from different fields in the raw data as a best effort. Not all values are available for all source types.
    pub condition: Option<Condition>,

    /// Dew point at timestamp, 2 m above ground
    ///
    /// DWD Unit: °C
    pub dew_point: Option<f32>,

    /// Icon alias suitable for the current weather conditions. Unlike the numerical parameters, this field is not taken as-is from the raw data (because it does not exist),
    /// but is calculated from different fields in the raw data as a best effort. Not all values are available for all source types.
    pub icon: Option<Icon>,

    /// Total precipitation during previous 10 minutes
    ///
    /// DWD Unit: mm
    pub precipitation_10: Option<f32>,
    /// Total precipitation during previous 30 minutes
    ///
    /// DWD Unit: mm
    pub precipitation_30: Option<f32>,
    /// Total precipitation during previous 60 minutes
    ///
    /// DWD Unit: mm
    pub precipitation_60: Option<f32>,

    /// Atmospheric pressure at timestamp, reduced to mean sea level
    ///
    /// DWD Unit: hPa
    pub pressure_msl: Option<f32>,

    /// Relative humidity at timestamp
    ///
    /// DWD Unit: %
    pub relative_humidity: Option<f32>,

    /// Solar irradiation during previous 10 minutes
    ///
    /// DWD Unit: kWh / m²
    pub solar_10: Option<f32>,
    /// Solar irradiation during previous 30 minutes
    ///
    /// DWD Unit: kWh / m²
    pub solar_30: Option<f32>,
    /// Solar irradiation during previous 60 minutes
    ///
    /// DWD Unit: kWh / m²
    pub solar_60: Option<f32>,

    /// Sunshine duration during previous 30 minutes
    ///
    /// DWD Unit: min
    pub sunshine_30: Option<f32>,
    /// Sunshine duration during previous 60 minutes
    ///
    /// DWD Unit: min
    pub sunshine_60: Option<f32>,

    /// Air temperature at timestamp, 2 m above the ground
    ///
    /// DWD Unit: °C
    pub temperature: Option<f32>,

    /// Visibility at timestamp
    ///
    /// DWD Unit: m
    pub visibility: Option<f32>,

    /// Mean wind direction during previous 10 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_direction_10: Option<f32>,
    /// Mean wind direction during previous 30 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_direction_30: Option<f32>,
    /// Mean wind direction during previous 60 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_direction_60: Option<f32>,

    /// Mean wind speed during previous previous 10 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_speed_10: Option<f32>,
    /// Mean wind speed during previous previous 30 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_speed_30: Option<f32>,
    /// Mean wind speed during previous previous 60 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_speed_60: Option<f32>,

    /// Direction of maximum wind gust during previous 10 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_gust_direction_10: Option<f32>,
    /// Direction of maximum wind gust during previous 30 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_gust_direction_30: Option<f32>,
    /// Direction of maximum wind gust during previous 60 minutes, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_gust_direction_60: Option<f32>,

    /// Speed of maximum wind gust during previous 10 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_gust_speed_10: Option<f32>,
    /// Speed of maximum wind gust during previous 30 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_gust_speed_30: Option<f32>,
    /// Speed of maximum wind gust during previous 60 minutes, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_gust_speed_60: Option<f32>,

    /// Object mapping meteorological parameters to the source IDs of alternative sources that were used to fill up missing values in the main source
    pub fallback_source_ids: Option<Value>
}

#[derive(Deserialize)]
struct HourlyWeatherResponse {
    weather: Vec<HourlyWeather>
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HourlyWeather {
    /// ISO 8601-formatted timestamp of this weather record
    #[serde(with = "date_serde")]
    pub timestamp: DateTime<FixedOffset>,

    /// Bright Sky source ID for this record
    pub source_id: i32,

    /// Total cloud cover at timestamp
    ///
    /// DWD Unit: %
    pub cloud_cover: Option<f32>,

    /// Current weather conditions. Unlike the numerical parameters, this field is not taken as-is from the raw data (because it does not exist),
    /// but is calculated from different fields in the raw data as a best effort. Not all values are available for all source types.
    pub condition: Option<Condition>,

    /// Dew point at timestamp, 2 m above ground
    ///
    /// DWD Unit: °C
    pub dew_point: Option<f32>,

    /// Icon alias suitable for the current weather conditions. Unlike the numerical parameters, this field is not taken as-is from the raw data (because it does not exist),
    /// but is calculated from different fields in the raw data as a best effort. Not all values are available for all source types.
    pub icon: Option<Icon>,

    /// Total precipitation during previous 60 minutes
    ///
    /// DWD Unit: mm
    pub precipitation: Option<f32>,

    /// Probability of more than 0.1 mm of precipitation in the previous hour (only available in forecasts)
    ///
    /// DWD Unit: %
    pub precipitation_probability: Option<f32>,

    /// Probability of more than 0.2 mm of precipitation in the previous 6 hours (only available in forecasts at 0:00, 6:00, 12:00, and 18:00 UTC)
    ///
    /// DWD Unit: %
    pub precipitation_probability_6h: Option<f32>,

    /// Atmospheric pressure at timestamp, reduced to mean sea level
    ///
    /// DWD Unit: hPa
    pub pressure_msl: Option<f32>,

    /// Relative humidity at timestamp
    ///
    /// DWD Unit: %
    pub relative_humidity: Option<f32>,

    /// Solar irradiation during previous 60 minutes
    ///
    /// DWD Unit: kWh / m²
    pub solar: Option<f32>,

    /// Sunshine duration during previous 60 minutes
    ///
    /// DWD Unit: min
    pub sunshine: Option<f32>,

    /// Air temperature at timestamp, 2 m above the ground
    ///
    /// DWD Unit: °C
    pub temperature: Option<f32>,

    /// Visibility at timestamp
    ///
    /// DWD Unit: m
    pub visibility: Option<f32>,

    /// Mean wind direction during previous hour, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_direction: Option<f32>,

    /// Mean wind speed during previous previous hour, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_speed: Option<f32>,

    /// Direction of maximum wind gust during previous hour, 10 m above the ground
    ///
    /// DWD Unit: °
    pub wind_gust_direction: Option<f32>,

    /// Speed of maximum wind gust during previous hour, 10 m above the ground
    ///
    /// DWD Unit: km / h
    pub wind_gust_speed: Option<f32>,

    /// Object mapping meteorological parameters to the source IDs of alternative sources that were used to fill up missing values in the main source
    pub fallback_source_ids: Option<Value>
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Condition {
    #[serde(rename = "dry")]
    Dry,
    #[serde(rename = "fog")]
    Fog,
    #[serde(rename = "rain")]
    Rain,
    #[serde(rename = "sleet")]
    Sleet,
    #[serde(rename = "snow")]
    Snow,
    #[serde(rename = "hail")]
    Hail,
    #[serde(rename = "thunderstorm")]
    Thunderstorm,
    #[serde(rename = "null")]
    Null,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Icon {
    #[serde(rename = "clear-day")]
    ClearDay,
    #[serde(rename = "clear-night")]
    ClearNight,
    #[serde(rename = "partly-cloudy-day")]
    PartlyCloudyDay,
    #[serde(rename = "partly-cloudy-night")]
    PartlyCloudyNight,
    #[serde(rename = "cloudy")]
    Cloudy,
    #[serde(rename = "fog")]
    Fog,
    #[serde(rename = "wind")]
    Wind,
    #[serde(rename = "rain")]
    Rain,
    #[serde(rename = "sleet")]
    Sleet,
    #[serde(rename = "snow")]
    Snow,
    #[serde(rename = "hail")]
    Hail,
    #[serde(rename = "thunderstorm")]
    Thunderstorm,
    #[serde(rename = "null")]
    Null,
}

const CURRENT_WEATHER_URL: &str = "https://api.brightsky.dev/current_weather";
const HOURLY_WEATHER_URL: &str = "https://api.brightsky.dev/weather";

/// Get current weather of position
pub fn get_current_weather(position: &LatLon, time_zone: &chrono_tz::Tz) -> Result<CurrentWeather> {
    let query: &[(&str, &str)] = &[
        ("lat", &format!("{}", position.lat)),
        ("lon", &format!("{}", position.lon)),
        ("tz", time_zone.name())
    ];
    let response: CurrentWeatherResponse = Client::new().get(CURRENT_WEATHER_URL).query(query).send()?.json()?;
    Ok(response.weather)
}

/// Get the hourly weather of position of the given day
///
/// Example: 2023-08-07 => Weather of 2023-08-07 00:00 - 2023-08-07 23:00
pub fn get_hourly_weather(date_time: &DateTime<FixedOffset>, position: &LatLon, time_zone: &chrono_tz::Tz) -> Result<Vec<HourlyWeather>> {
    let date = format!("{}", date_time.format("%Y-%m-%d"));
    let query: &[(&str, &str)] = &[
        ("date", &date),
        ("lat", &format!("{}", position.lat)),
        ("lon", &format!("{}", position.lon)),
        ("tz", time_zone.name())
    ];
    let response: HourlyWeatherResponse = Client::new().get(HOURLY_WEATHER_URL).query(query).send()?.json()?;
    Ok(response.weather.into_iter().take(24).collect()) // returns 25 records but we want only 24
}

/// Get the hourly weather of position of the given day
///
/// Example: 2023-08-07 => Weather of 2023-08-07 00:00 - 2023-08-08 00:00
pub fn get_weather_forecast(date_time: &DateTime<FixedOffset>, days: i64, position: &LatLon, time_zone: &chrono_tz::Tz) -> Vec<HourlyWeather> {
    let mut forecast: Vec<HourlyWeather> = Vec::new();

    for i in 0..days {
        let date = *date_time + Duration::days(i);
        let result = get_hourly_weather(&date, position, time_zone);

        if let Ok(mut weather) = result {
            forecast.append(&mut weather);
        }
    }
    forecast
}

mod date_serde {
    use chrono::{DateTime, FixedOffset};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSITION: LatLon = LatLon {
        lat: 52.52,
        lon: 13.4
    };

    const TIME_ZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

    #[test]
    fn test_current_weather() {
        let weather = get_current_weather(&POSITION, &TIME_ZONE);

        assert!(weather.is_ok());
        println!("{:?}", weather)
    }

    #[test]
    fn test_hourly_weather() {
        let date_time: DateTime<FixedOffset> = chrono::Local::now().into();
        let weather_records = get_hourly_weather(&date_time, &POSITION, &TIME_ZONE);

        assert!(weather_records.is_ok());
        for record in weather_records.unwrap() {
            println!("{:?}", record)
        }
    }

    #[test]
    fn test_forecast() {
        let days: i64 = 7;
        let date_time: DateTime<FixedOffset> = chrono::Local::now().into();
        let weather_records = get_weather_forecast(&date_time, days, &POSITION, &TIME_ZONE);

        println!("Items for {} days: {:?}", days, weather_records.len());
        for record in weather_records {
            println!("{:?}", record)
        }
    }
}
