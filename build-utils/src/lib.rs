use std::env;
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use anyhow::{Context, Result};

pub fn get_config() -> Result<serde_json::Value> {
    // Read the JSON configuration file
    let mut config_file = File::open("config.json").context("Failed to open config.json")?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents).context("Failed to read config.json")?;

    let config: serde_json::Value = serde_json::from_str(&config_contents).context("Failed to parse config.json")?;
    Ok(config)
}

pub fn write_code(code: &str) -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("config.rs");
    File::create(&dest_path)?
        .write_all(code.as_bytes())
        .context("Failed to write generated code")?;
    
    println!("cargo:rerun-if-changed=config.json");
    Ok(())
}

/// Get a string value from a JSON object
pub fn get_str(value: &serde_json::Value, key: &str) -> Result<String> {
    let val = get_value(value, key)?.as_str().context(format!("Field is not a string: {}", key))?;
    Ok(val.to_string())
}

/// Get a float value from a JSON object
pub fn get_f64(value: &serde_json::Value, key: &str) -> Result<f64> {
    let val = get_value(value, key)?.as_f64().context(format!("Field is not a number: {}", key))?;
    Ok(val)
}

/// Get a nested value from a JSON object
/// 
/// Example: You can use `get_value(&config, "weather_conditions.rain")` instead of `config["weather_conditions"]["rain"]`
fn get_value<'a>(value: &'a serde_json::Value, key: &'a str) -> Result<&'a serde_json::Value> {
    let path: Vec<&str> = key.split('.').collect();
    let val = path.iter().try_fold(value, |acc, &key| {
        acc.get(key).context(format!("Field not found: {}", key))
    })?;
    Ok(val)
}
