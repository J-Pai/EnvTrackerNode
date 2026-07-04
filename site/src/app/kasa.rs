//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use egui::Response;
use egui_async::Bind;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
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

pub(super) struct Kasa {
    api_endpoint: String,
    data: Bind<Vec<KasaChildInfo>, String>,
}

impl Kasa {
    pub(super) fn new(kasa_api_endpoint: &String) -> Self {
        Self {
            api_endpoint: kasa_api_endpoint.clone(),
            data: Bind::new(true),
        }
    }

    pub(super) fn logic(&mut self) {
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

                        log::info!("{data:#?}");

                        data
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            10.0,
        );
    }
}

pub struct BorrowPointsExample {
    points: Vec<PlotPoint>,
}

impl Default for BorrowPointsExample {
    fn default() -> Self {
        let points: Vec<[f64; 2]> =
            vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0], [4.0, 5.0], [5.0, 9.0]];
        let points = points.iter().map(|p| PlotPoint::new(p[0], p[1])).collect();
        Self { points }
    }
}

impl BorrowPointsExample {
    pub fn show_plot(&self, ui: &mut egui::Ui, nr: i32, reset: bool) -> Response {
        let mut plot = Plot::new(format!("plot{nr}"))
            .legend(Legend::default())
            .width(ui.available_width());

        if reset {
            plot = plot.reset();
        }

        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new("curve", PlotPoints::Borrowed(&self.points)).name("curve"));
        })
        .response
    }
}
