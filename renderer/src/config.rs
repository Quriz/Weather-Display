use brightsky::LatLon;

pub struct WeekdayNames {
    pub monday: &'static str,
    pub tuesday: &'static str,
    pub wednesday: &'static str,
    pub thursday: &'static str,
    pub friday: &'static str,
    pub saturday: &'static str,
    pub sunday: &'static str,
}

pub struct WeatherConditions {
    pub dry: &'static str,
    pub fog: &'static str,
    pub rain: &'static str,
    pub sleet: &'static str,
    pub snow: &'static str,
    pub hail: &'static str,
    pub thunderstorm: &'static str,
    pub null: &'static str,
}

pub struct Config {
    pub webdav_url: &'static str,
    pub image_name: &'static str,
    pub location: LatLon,
    pub timezone: chrono_tz::Tz,
    pub time_format: &'static str,
    pub weekday_names: WeekdayNames,
    pub weather_conditions: WeatherConditions,
}

include!(concat!(env!("OUT_DIR"), "/config.rs"));
