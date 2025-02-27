use crate::wifi;

pub struct Config {
    pub wifi: wifi::WifiConfig,
    pub image_url: &'static str,
    pub timezone: chrono_tz::Tz
}

include!(concat!(env!("OUT_DIR"), "/config.rs"));
