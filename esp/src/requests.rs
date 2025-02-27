use esp_idf_svc::http::client::EspHttpConnection;
use embedded_svc::http::client::Client as HttpClient;

use anyhow::Result;

use crate::http::{get_request, response_read_bytes};

pub fn request_image(client: &mut HttpClient<EspHttpConnection>, image_data_url: &str) -> Result<Vec<u8>> {
    log::info!("request image from server");
    
    let headers = [("accept", "application/octet-stream")];
    let mut response = get_request(client, image_data_url, &headers)?;

    let buffer_size = crate::get_buffer_size();
    let (buffer, bytes_read) = response_read_bytes(&mut response, buffer_size)?;

    if bytes_read == 0 {
        anyhow::bail!("Image data body was empty");
    }
    if bytes_read != buffer_size {
        anyhow::bail!("Image data body was wrong size. Expected {}, got {}", buffer_size, bytes_read);
    }

    Ok(buffer)
}

