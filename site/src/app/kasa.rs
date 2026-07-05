//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use chrono::DateTime;
use chrono::Local;
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
    plot: BorrowPointsExample,
}

impl Kasa {
    pub(super) fn new(api_endpoint: &String) -> Self {
        Self {
            api_endpoint: api_endpoint.clone(),
            data: Bind::new(true),
            plot: BorrowPointsExample::default(),
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
    fn ui(&mut self, ui: &mut egui::Ui, id: &PaneId, _alias: &String) -> egui_tiles::UiResponse {
        let color = match ui.theme() {
            egui::Theme::Dark => egui::epaint::Hsva::new(0.0, 0.0, 0.025, 1.0),
            egui::Theme::Light => egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0),
        };
        let mut drag = egui_tiles::UiResponse::None;

        let api_endpoint = self.api_endpoint.clone();
        let api_client = ClientBuilder::new(Client::new()).build();
        let device_id = id.0.clone();

        self.data.request_every_sec(
            || async move {
                match api_client
                    .get(format!("{api_endpoint}"))
                    .query(&[
                        ("limit", "1000"),
                        ("id", &device_id),
                        ("order_by", "desc"),
                        ("column", "utc_ns"),
                    ])
                    .send()
                    .await
                {
                    Ok(mut data) => {
                        let url = data.url().clone();
                        data = match data.error_for_status() {
                            Ok(data) => data,
                            Err(err) => return Err(err.to_string()),
                        };

                        let json = data.text().await.map_err(|e| e.to_string())?;

                        let data = serde_json::from_str::<Vec<KasaChildInfo>>(&json)
                            .map_err(|e| e.to_string());

                        if let Ok(data) = data.as_ref() {
                            let first = data.get(0).unwrap();
                            let dt = chrono::DateTime::from_timestamp_nanos(first.utc_ns);
                            let local_dt: DateTime<Local> = DateTime::from(dt);

                            log::info!("query {url} ... {local_dt:?}");
                        }

                        data
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            5.0,
        );

        let mut current_power_mw = 0.0;
        match self.data.read() {
            Some(data) => match data {
                Ok(data) => {
                    current_power_mw = data.get(0).unwrap().emeter.power_mw as f64 / 1000.0;
                    self.plot.update_points(
                        data.iter()
                            .rev()
                            .map(|d| d.emeter.power_mw as f64 / 1000.0)
                            .collect(),
                    )
                }
                Err(e) => {
                    log::error!("{e}");
                }
            },
            None => {}
        }

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
                ui.separator();
                ui.label(format!("POWER (Watts)"));
                ui.label(format!("{current_power_mw:.3}"));
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
                    .show(ui, |ui| self.plot.show_plot(ui, id));
            });

        drag
    }
}

pub struct BorrowPointsExample {
    points: Vec<PlotPoint>,
    reset: bool,
}

impl Default for BorrowPointsExample {
    fn default() -> Self {
        Self {
            points: vec![],
            reset: true,
        }
    }
}

impl BorrowPointsExample {
    pub fn update_points(&mut self, points: Vec<f64>) {
        self.points = points
            .iter()
            .enumerate()
            .map(|(i, p)| PlotPoint::new(i as f64, *p))
            .collect();
    }

    pub fn show_plot(&mut self, ui: &mut egui::Ui, id: &PaneId) -> Response {
        let mut plot = Plot::new(format!("plot-{}", id.0))
            .legend(Legend::default())
            .width(ui.available_width());

        if self.reset {
            plot = plot.reset();
            self.reset = false;
        }

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("power_w", PlotPoints::Borrowed(&self.points))
                    .name("power_w")
                    .color(Color32::BLUE),
            );
        })
        .response
    }
}
