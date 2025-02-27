use anyhow::{bail, Ok, Result};
use embedded_svc::wifi::{
    AuthMethod, ClientConfiguration, Configuration,
};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{EspWifi, BlockingWifi},
};
use log::info;

pub struct WifiConfig {
    pub ssid: &'static str,
    pub psk: &'static str,
}

pub fn setup_wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    storage: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    info!("Connecting to Wifi...");

    let mut auth_method = AuthMethod::WPAWPA2Personal;

    if ssid.is_empty() {
        bail!("Missing Wifi name")
    }

    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }

    let wifi = EspWifi::new(modem, sysloop.clone(), Some(storage))?;
    let mut wifi = BlockingWifi::wrap(wifi, sysloop).expect("Failed to create blocking wifi");

    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: ssid.try_into().unwrap(),
            password: pass.try_into().unwrap(),
            auth_method,
            ..Default::default()
        }
    ))?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connectd");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}

pub fn disable_wifi(mut wifi: BlockingWifi<EspWifi<'static>>) -> Result<()> {
    wifi.disconnect()?;
    wifi.stop()?;
    drop(wifi);

    Ok(())
}