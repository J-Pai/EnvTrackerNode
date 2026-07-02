//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use egui_async::Bind;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::console_log;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct KasaDeviceChild {
    /// Human-readable name of the device.
    alias: String,
    /// Unique identifier.
    id: String,
    /// On/Off state.
    state: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct EMeter {
    current_ma: u64,
    power_mw: u64,
    voltage_mv: u64,
    total_wh: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct KasaChildInfo {
    utc_ns: i64,
    info: KasaDeviceChild,
    emeter: EMeter,
}

pub(super) struct Kasa {
    api_endpoint: String,
    data: Bind<Vec<KasaChildInfo>, String>,
}

impl Kasa {
    pub(super) fn new(api_endpoint: &String, kasa_api_endpoint: &String) -> Self {
        Self {
            api_endpoint: format!("{}{}", api_endpoint, kasa_api_endpoint).to_string(),
            data: Bind::new(true),
        }
    }

    pub(super) fn ui(&mut self) {
        let api_endpoint = self.api_endpoint.clone();
        let api_client = ClientBuilder::new(Client::new()).build();

        self.data.request_every_sec(
            || async move {
                match api_client.get(format!("{api_endpoint}")).send().await {
                    Ok(mut data) => {
                        data = match data.error_for_status() {
                            Ok(data) => data,
                            Err(err) => return Err(err.to_string()),
                        };

                        let json = data.text().await.map_err(|e| e.to_string())?;

                        let data = serde_json::from_str::<Vec<KasaChildInfo>>(&json)
                            .map_err(|e| e.to_string());

                        console_log!(format!("{data:#?}"));

                        data
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            10.0,
        );
    }
}
