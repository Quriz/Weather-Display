use anyhow::Result;

fn main() -> Result<()> {
    embuild::espidf::sysenv::output();

    let config = build_utils::get_config()?;

    let get_str = |key| build_utils::get_str(&config, key);

    // Generate Rust code to create a Config instance
    let code = format!(
        "pub const CONFIG: Config = Config {{
            wifi: wifi::WifiConfig {{
                ssid: {:?},
                psk: {:?}
            }},
            image_url: {:?},
            timezone: chrono_tz::{}
        }};",
        get_str("wifi.ssid")?,
        get_str("wifi.psk")?,
        get_str("image_url")?,
        get_str("timezone")?.replace("/", "::"),
    );

    build_utils::write_code(&code)?;

    Ok(())
}
