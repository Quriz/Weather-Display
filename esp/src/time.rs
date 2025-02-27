use anyhow::Result;
use chrono::{DateTime, Duration, NaiveTime, Timelike, Utc};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use log::info;
use crate::config::CONFIG;

pub fn setup_snpt() -> Result<EspSntp<'static>> {
	let sntp = EspSntp::new_default()?;

	log::info!("Synchronizing with NTP server...");
    while sntp.get_sync_status() != SyncStatus::Completed {}
    log::info!("Time sync completed");

	Ok(sntp)
}

pub fn disable_sntp(sntp: EspSntp<'static>) {
	drop(sntp);
}

fn get_current_time() -> DateTime<chrono_tz::Tz> {
    Utc::now().with_timezone(&CONFIG.timezone)
}

pub fn get_sleep_time() -> Duration {
    let now = get_current_time();

    // Refresh display every hour in daytime
    // and don't refresh at night
    if now.hour() >= 6 && now.hour() <= 22 {
        time_until_next_hour()
    } else {
        time_until_6am()
    }
}

fn time_until_next_hour() -> Duration {
    let now = get_current_time();
    info!("Current time: {:?}", now);
    
    let next_hour = now.checked_add_signed(Duration::hours(1)).unwrap();

    // Set an offset of 1 minute to wait for the new image to be generated
    let next_hour_datetime = next_hour.with_minute(1).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
    info!("Start of next hour: {:?}", next_hour_datetime);
    
    next_hour_datetime.signed_duration_since(now)
}

fn time_until_6am() -> Duration {
    let now = get_current_time();
    info!("Current time: {:?}", now);

    // Calculate the next 6am
    let next_6am = if now.time() < NaiveTime::from_hms_opt(6, 0, 0).unwrap() {
        now.date_naive().and_hms_opt(6, 0, 0).unwrap()
    } else {
        now.date_naive().succ_opt().unwrap().and_hms_opt(6, 0, 0).unwrap()
    };

    println!("Next 6am: {:?}", next_6am);

    next_6am.signed_duration_since(now.naive_local())
}
