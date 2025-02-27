use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::gpio;
use esp_idf_hal::spi;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::modem::Modem;
use esp_idf_hal::delay::Delay;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use chrono::Duration;

use epd_waveshare::{
    color::*,
    epd7in5b_v2::{Epd7in5, WIDTH, HEIGHT},
    graphics::VarDisplay,
};
use epd_waveshare::prelude::*;

mod wifi;
mod time;
mod http;
mod requests;
mod config;
use config::CONFIG;

type SpiDev = SpiDeviceDriver<'static, SpiDriver<'static>>;

type EpdDriver = Epd7in5<
    SpiDev,
    PinDriver<'static, AnyInputPin, Input>,
    PinDriver<'static, AnyOutputPin, Output>,
    PinDriver<'static, AnyOutputPin, Output>,
    Delay>;

fn main() -> anyhow::Result<()> {
    esp_setup();

    let peripherals = Peripherals::take().unwrap();
    let (led_pin, modem, spi_driver, epd, delay) = gather_peripherals(peripherals)?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    disable_onboard_led(led_pin)?;

    // Start wifi
    let wifi = wifi::setup_wifi(
        CONFIG.wifi.ssid,
        CONFIG.wifi.psk,
        modem,
        sysloop,
        nvs,
    ).expect("Failed to connect to wifi");

    // Setup SNTP
    // Keep the object around or else the SNTP service will stop
    let sntp = time::setup_snpt()?;

    // Get image
    let mut http_client = http::create_client()?;
    let image_result = requests::request_image(&mut http_client, CONFIG.image_url);

    // Draw image on display
    if let Ok(image_data) = image_result {
        log::info!("Drawing image...");
        draw_epd(image_data, spi_driver, epd, delay)?;
    }
    else {
        log::error!("Getting image data failed: {:?}", image_result.unwrap_err());
    }

    // Get time duration until next hour
    let sleep_time = time::get_sleep_time();
    log::info!("Sleeping {} seconds...", sleep_time.num_seconds());
    time::disable_sntp(sntp);

    // Turn off wifi as it is not needed anymore
    log::info!("Turning off wifi");
    wifi::disable_wifi(wifi)?;

    // deep sleep for 1 hour
    enter_deep_sleep(sleep_time);

    unreachable!("In sleep");
}

fn esp_setup() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Wakeup: {:?}", esp_idf_hal::reset::WakeupReason::get());
}

fn gather_peripherals(peripherals: Peripherals) -> anyhow::Result<(Gpio2, Modem, SpiDev, EpdDriver, Delay)> {
    let pins = peripherals.pins;
    let ledpin = pins.gpio2;

    let modem = peripherals.modem;

    let spi_p = peripherals.spi3;
    let sclk: AnyOutputPin = pins.gpio13.into();
    let sdo: AnyOutputPin = pins.gpio14.into();
    let busy_in: AnyInputPin = pins.gpio25.into();
    let rst: AnyOutputPin = pins.gpio26.into();
    let dc: AnyOutputPin = pins.gpio27.into();

    info!("Create EPD driver");
    let (spi_driver, epd, delay) = create_epd_driver(spi_p, sclk, sdo, busy_in, rst, dc)?;

    Ok((ledpin, modem, spi_driver, epd, delay))
}

fn enter_deep_sleep(sleep_time: Duration) {
    info!("Entering deep sleep");
    unsafe {
        // TODO: measure current draw vs gpio_deep_sleep_hold_en
        // esp_idf_sys::rtc_gpio_hold_en(led.pin());
        // esp_idf_sys::gpio_deep_sleep_hold_en();

        // TODO: see if these need to be configured or if it makes a difference at all
        // esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_PERIPH, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_SLOW_MEM, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_FAST_MEM, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_XTAL, ESP_PD_OPTION_OFF);

        let sleep_time_us = sleep_time.num_microseconds().unwrap() as u64;
        println!("Sleep time us: {}", sleep_time_us);
        esp_idf_sys::esp_deep_sleep(sleep_time_us);
    }
    // unreachable!("we will be asleep by now");
}

/// Disable the onboard led during deep sleep
/// TODO: measure current draw vs gpio_deep_sleep_hold_en
/// currently unused because i desoldered the leds, but I wanted to keep the function so I just
/// made it pub
fn disable_onboard_led(ledpin: gpio::Gpio2) -> anyhow::Result<()> {
    log::info!("Disable onboard led");
    let mut led = PinDriver::output(ledpin)?;
    led.set_low()?;
    unsafe { esp_idf_sys::rtc_gpio_hold_en(led.pin()); }

    Ok(())
}

fn create_epd_driver(
	spi_p: spi::SPI3,
	sclk: AnyOutputPin,
	sdo: AnyOutputPin,
    busy_in: AnyInputPin,
    rst: AnyOutputPin,
    dc: AnyOutputPin,
    ) -> anyhow::Result<(SpiDev, EpdDriver, Delay)> {

    let mut driver = spi::SpiDeviceDriver::new_single(
        spi_p,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyOutputPin>::None,
        &spi::config::DriverConfig::new(),
        &spi::config::Config::new().baudrate(10.MHz().into()),
    )?;

    info!("Driver setup completed");
    let mut delay = Delay::default();

    // Setup EPD
    let epd_driver = Epd7in5::new(
        &mut driver,
        PinDriver::input(busy_in)?,
        PinDriver::output(dc)?,
        PinDriver::output(rst)?,
        &mut delay,
        None,
    )
    .unwrap();

    info!("Epd setup completed");

    Ok((driver, epd_driver, delay))
}

fn draw_epd(mut buffer: Vec<u8>, mut driver: SpiDev, mut epd: EpdDriver, mut delay: Delay) -> anyhow::Result<()> {
    let expected_len = get_buffer_size();

    // check that what we got from the server is actually the same size as when me make a
    // epd_waveshare buffer of size WIDTH*HEIGHT pixels
    let buffer_len = buffer.len();
    if buffer_len != expected_len {
        anyhow::bail!("Buffer len expected {}, got {}", expected_len, buffer_len);
    }
    let display = VarDisplay::<TriColor>::new(WIDTH, HEIGHT, &mut buffer, false).expect("failed to create display");

    epd
        .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
        .expect("Display frame");
    info!("Called display frame");

    Delay::default().delay_ms(20_000u32);

    info!("Done waiting");
    info!("Putting display to sleep");
    epd.sleep(&mut driver, &mut delay).expect("Failed to sleep");

    Ok(())
}

/// Retuns the size of a buffer necessary to hold the entire image
fn get_buffer_size() -> usize {
    // The height is multiplied by 2 because the red pixels essentially exist on a separate "layer"
    epd_waveshare::buffer_len(WIDTH as usize, HEIGHT as usize * 2)
}
