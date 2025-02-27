use anyhow::Result;

fn main() -> Result<()> {
    let config = build_utils::get_config()?;

    let get_str = |key| build_utils::get_str(&config, key);
    let get_f64 = |key| build_utils::get_f64(&config, key);

    // Generate Rust code to create a Config instance
    let code = format!(
        "pub const CONFIG: Config = Config {{
            webdav_url: {:?},
            image_name: {:?},
            location: LatLon {{
                lat: {},
                lon: {}
            }},
            timezone: chrono_tz::{},
            time_format: {:?},
            weekday_names: WeekdayNames {{
                monday: {:?},
                tuesday: {:?},
                wednesday: {:?},
                thursday: {:?},
                friday: {:?},
                saturday: {:?},
                sunday: {:?},
            }},
            weather_conditions: WeatherConditions {{
                dry: {:?},
                fog: {:?},
                rain: {:?},
                sleet: {:?},
                snow: {:?},
                hail: {:?},
                thunderstorm: {:?},
                null: {:?},
            }},
        }};",
        get_str("webdav_url")?,
        get_str("image_name")?,
        get_f64("location.lat")?,
        get_f64("location.lon")?,
        get_str("timezone")?.replace("/", "::"),
        get_str("time_format")?,
        get_str("weekday_names.monday")?,
        get_str("weekday_names.tuesday")?,
        get_str("weekday_names.wednesday")?,
        get_str("weekday_names.thursday")?,
        get_str("weekday_names.friday")?,
        get_str("weekday_names.saturday")?,
        get_str("weekday_names.sunday")?,
        get_str("weather_conditions.dry")?,
        get_str("weather_conditions.fog")?,
        get_str("weather_conditions.rain")?,
        get_str("weather_conditions.sleet")?,
        get_str("weather_conditions.snow")?,
        get_str("weather_conditions.hail")?,
        get_str("weather_conditions.thunderstorm")?,
        get_str("weather_conditions.null")?,
    );

    build_utils::write_code(&code)?;

    Ok(())
}
