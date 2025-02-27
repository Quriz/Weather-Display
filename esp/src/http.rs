use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use embedded_svc::http::client::{Client, Response};
use embedded_svc::http::Method;
use embedded_svc::utils::io;

use anyhow::Result;

pub fn create_client() -> Result<Client<EspHttpConnection>> {
    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    Ok(Client::wrap(EspHttpConnection::new(&config)?))
}

pub fn get_request<'a>(http_client: &'a mut Client<EspHttpConnection>, url: &'a str, headers: &'a [(&str, &str)]) -> Result<Response<&'a mut EspHttpConnection>> {
    let mut combined_headers = vec![("connection", "close")];
	combined_headers.extend_from_slice(headers);

    log::info!("Making request: {}", url);
    let response = http_client.request(Method::Get, url, headers)?.submit()?;

    // Process response
    let status = response.status();
    if !(200..300).contains(&status) {
        anyhow::bail!("Response status is not a success: {}", status);
    }

    Ok(response)
}

/// Read out response body bytes into buffer of specified size
pub fn response_read_bytes(response: &mut Response<&mut EspHttpConnection>, buffer_size: usize) -> Result<(Vec<u8>, usize)> {
	let (_headers, mut body) = response.split();
    let mut buffer = vec![0u8; buffer_size];
    let bytes_read = io::try_read_full(&mut body, &mut buffer).map_err(|e| e.0)?;
    log::info!("Request body: read {} bytes", bytes_read);
	
    // Drain the remaining response bytes since io::try_read_full() stops if the buffer end was reached
    while body.read(&mut buffer)? > 0 {}

	Ok((buffer, bytes_read))
}
