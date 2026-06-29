//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use egui_async::Bind;
use serde::Deserialize;
use serde::Serialize;

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

struct Kasa {
    data: Bind<Vec<KasaChildInfo>, Box<dyn std::error::Error>>,
}

impl Kasa {
    pub(super) fn new() -> Self {
        Self {
            data: Bind::new(true),
        }
    }
}
