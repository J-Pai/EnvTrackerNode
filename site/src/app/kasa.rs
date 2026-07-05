//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use egui::Color32;
use egui::Frame;
use egui::Margin;
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

use crate::app::EnvWidget;
use crate::app::PaneId;

#[derive(Clone, Deserialize)]
pub(super) struct KasaDeviceChildAlias(pub(super) String);
#[derive(Clone, Debug, Deserialize)]
pub(super) struct KasaDeviceChildId(pub(super) String);

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
    pub(super) fn new(api_endpoint: &String) -> Self {
        Self {
            api_endpoint: api_endpoint.clone(),
            data: Bind::new(true),
        }
    }

    pub(super) fn request_device_ids(
        devices: &mut Bind<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>, String>,
        api_endpoint: &String,
    ) {
        let api_endpoint = api_endpoint.clone();
        if devices.is_pending() {
            return;
        }
        devices.request(async move {
            let api_client = ClientBuilder::new(Client::new()).build();
            match api_client
                .get(format!("{api_endpoint}"))
                .query(&[("distinct", ""), ("column", "id")])
                .send()
                .await
            {
                Ok(mut data) => {
                    data = match data.error_for_status() {
                        Ok(data) => data,
                        Err(err) => return Err(err.to_string()),
                    };

                    let json = data.text().await.map_err(|e| e.to_string())?;

                    let data =
                        serde_json::from_str::<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>>(
                            &json,
                        )
                        .map_err(|e| e.to_string());

                    data
                }
                Err(e) => Err(e.to_string()),
            }
        });
    }

    pub(super) fn read_device_ids(
        devices: &mut Bind<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>, String>,
    ) -> Vec<(KasaDeviceChildId, KasaDeviceChildAlias)> {
        match devices.read() {
            Some(data) => match data {
                Ok(devices) => devices.clone(),
                Err(e) => {
                    log::error!("{e}");
                    vec![]
                }
            },
            None => vec![],
        }
    }
}

impl EnvWidget for Kasa {
    fn ui(&mut self, ui: &mut egui::Ui, id: &PaneId, alias: &String) -> egui_tiles::UiResponse {
        let color = match ui.theme() {
            egui::Theme::Dark => egui::epaint::Hsva::new(0.0, 0.0, 0.025, 1.0),
            egui::Theme::Light => egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0),
        };
        let mut drag = egui_tiles::UiResponse::None;

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

                        log::info!("query...");

                        data
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            5.0,
        );

        // BorrowPointsExample::default().show_plot(ui, &String::new(), false);
        egui::Panel::left(format!("data_panel_{}", id.0))
            .frame(Frame {
                fill: Color32::from(color),
                inner_margin: Margin::same(8),
                ..Frame::default()
            })
            .min_size(200.0)
            .max_size(200.0)
            .resizable(false)
            .show(ui, |ui| {
                ui.label(format!("{}", alias));
                ui.separator();
                let dragged = ui
                    .allocate_rect(ui.max_rect(), egui::Sense::click_and_drag())
                    .on_hover_cursor(egui::CursorIcon::Grab)
                    .dragged();
                if dragged {
                    drag = egui_tiles::UiResponse::DragStarted;
                } else {
                    drag = egui_tiles::UiResponse::None;
                }
            });
        egui::CentralPanel::no_frame()
            .frame(Frame {
                fill: Color32::from(color),
                ..Frame::default()
            })
            .show(ui, |ui| {
                egui::CentralPanel::default_margins()
                    .frame(Frame {
                        fill: Color32::from(color),
                        inner_margin: Margin {
                            right: 8,
                            top: 8,
                            ..Margin::ZERO
                        },
                        ..Frame::default()
                    })
                    .show(ui, |ui| {
                        BorrowPointsExample::default().show_plot(ui, id, false);
                    });
            });

        drag
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
    pub fn show_plot(&self, ui: &mut egui::Ui, id: &PaneId, reset: bool) -> Response {
        let mut plot = Plot::new(format!("plot-{}", id.0))
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
