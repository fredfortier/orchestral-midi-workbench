use std::{error::Error, time::Duration};

use clap::Args;

use crate::osc_session::{API_PATH, DEFAULT_WAIT_MS, Request};

#[derive(Args)]
pub struct Osc {
    /// Bridge HTTP port.
    #[arg(long, default_value_t = notes_transformer::api::DEFAULT_PORT)]
    port: u16,
    /// Observe this exact feedback address after submitting. Repeat as needed.
    #[arg(long)]
    observe: Vec<String>,
    /// Feedback window in milliseconds when observing addresses.
    #[arg(long, default_value_t = DEFAULT_WAIT_MS)]
    wait_ms: u16,
    /// Raw OSC address, for example /refresh or /action/1.
    address: String,
    /// Zero or more arguments, each i:INT, f:FLOAT, or s:STRING.
    arguments: Vec<String>,
}

pub async fn run(args: Osc) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(Duration::from_secs(5))
        .build()?;
    let response = client
        .post(format!("http://127.0.0.1:{}{API_PATH}", args.port))
        .json(&Request {
            address: args.address,
            arguments: args.arguments,
            observe: args.observe,
            wait_ms: args.wait_ms,
        })
        .send()
        .await
        .map_err(|error| format!("OSC service request failed; submission unconfirmed: {error}"))?;
    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err("OSC is unavailable; start the bridge with --osc-port".into());
    }
    let body = response
        .text()
        .await
        .map_err(|error| format!("OSC service response lost; submission unconfirmed: {error}"))?;
    if !status.is_success() {
        return Err(format!("OSC service returned {status}: {body}").into());
    }
    println!("{body}");
    Ok(())
}
