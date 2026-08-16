//! Building Kasa UI elements.
//!
//! Try to keep this as close as possible to node/src/services/kasa.rs

use std::ops::RangeInclusive;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use egui::Color32;
use egui::Frame;
use egui::Margin;
use egui::Response;
use egui::mutex::RwLock;
use egui_async::Bind;
use egui_plot::GridInput;
use egui_plot::GridMark;
use egui_plot::HoverPosition;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotBounds;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_plot::log_grid_spacer;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

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
    api_uri: Url,
    data: Bind<Vec<KasaChildInfo>, String>,
    plot: KasaPlot,
    current_power_w: f64,
}

impl Kasa {
    const POLL_EVERY_SECONDS: f64 = 10.0;
    const INITIAL_BATCH_SIZE: usize = 1000;

    pub(super) fn new(api_uri: &Url) -> Self {
        Self {
            api_uri: api_uri.to_owned(),
            data: Bind::new(true),
            plot: KasaPlot::default(),
            current_power_w: 0.0,
        }
    }

    pub(super) fn request_device_ids(
        devices: &mut Bind<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>, String>,
        api_uri: &Url,
    ) {
        let api_uri = api_uri.to_owned();
        if devices.is_pending() {
            return;
        }

        devices.request(async move {
            let api_client = ClientBuilder::new(Client::new()).build();
            match api_client
                .get(api_uri)
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

                    serde_json::from_str::<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>>(&json)
                        .map_err(|e| e.to_string())
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

    pub(super) fn reset_plot(&mut self) {
        self.plot.reset = true;
    }
}

impl EnvWidget for Kasa {
    fn ui(&mut self, ui: &mut egui::Ui, id: &PaneId, _alias: &str) -> egui_tiles::UiResponse {
        let color = match ui.theme() {
            egui::Theme::Dark => egui::epaint::Hsva::new(0.0, 0.0, 0.025, 1.0),
            egui::Theme::Light => egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0),
        };

        let api_uri = self.api_uri.clone();
        let api_client = ClientBuilder::new(Client::new()).build();
        let device_id = id.0.clone();
        self.plot.device_id = device_id.clone();

        self.data.request_every_sec(
            || async move {
                match api_client
                    .get(api_uri)
                    .query(&[
                        ("limit", Self::INITIAL_BATCH_SIZE.to_string().as_str()),
                        ("id", &device_id),
                        ("order_by", "desc"),
                        ("column", "utc_ns"),
                    ])
                    .send()
                    .await
                {
                    Ok(mut data) => {
                        data = match data.error_for_status() {
                            Ok(data) => data,
                            Err(err) => return Err(err.to_string()),
                        };

                        let json = data.text().await.map_err(|e| e.to_string())?;

                        serde_json::from_str::<Vec<KasaChildInfo>>(&json).map_err(|e| e.to_string())
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            Self::POLL_EVERY_SECONDS,
        );

        self.data.on_finished(|data| match data {
            Ok(data) => {
                self.current_power_w = match data.first() {
                    Some(d) => d.emeter.power_mw as f64 / 1000.0,
                    None => 0.0,
                };

                let converted_data: Vec<(f64, f64)> = data
                    .iter()
                    .rev()
                    .map(|d| {
                        (
                            (d.utc_ns / 1000000) as f64 / 1000.0,
                            d.emeter.power_mw as f64 / 1000.0,
                        )
                    })
                    .collect();
                self.plot.update_points(&converted_data);
            }
            Err(e) => {
                log::error!("{e}");
            }
        });

        egui::Panel::left(format!("data_panel_{}", id.0))
            .frame(Frame {
                fill: Color32::from(color),
                inner_margin: Margin::same(8),
                ..Frame::default()
            })
            .min_size(150.0)
            .max_size(150.0)
            .resizable(false)
            .show(ui, |ui| {
                ui.separator();
                ui.label("POWER (Watts)");
                ui.label(format!("{:.3}", self.current_power_w));
                ui.separator();
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

        egui_tiles::UiResponse::None
    }
}
struct KasaPlot {
    device_id: String,
    points: Vec<PlotPoint>,
    reset: bool,
    plot_bounds: Arc<RwLock<Option<PlotBounds>>>,
}

impl Default for KasaPlot {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            points: vec![],
            reset: true,
            plot_bounds: Arc::new(RwLock::new(None)),
        }
    }
}

impl KasaPlot {
    fn update_points(&mut self, points: &[(f64, f64)]) {
        self.points = points.iter().map(|(t, p)| PlotPoint::new(*t, *p)).collect();
    }

    fn label_formatter(pos: &HoverPosition<'_>) -> Option<String> {
        match pos {
            HoverPosition::NearDataPoint {
                plot_name,
                position,
                index: _,
            } => {
                let datetime: DateTime<Utc> =
                    DateTime::from_timestamp_secs(position.x as i64).unwrap();
                let local: DateTime<Local> = DateTime::from(datetime);
                Some(format!(
                    "{plot_name}\n{}\nPower(w): {:.3}",
                    local, position.y
                ))
            }
            HoverPosition::Elsewhere { position: _ } => None,
        }
    }

    fn x_axis_formatter(mark: GridMark, _range: &RangeInclusive<f64>) -> String {
        let Some(datetime): Option<DateTime<Utc>> =
            DateTime::from_timestamp_secs(mark.value as i64)
        else {
            return String::new();
        };
        let local: DateTime<Local> = DateTime::from(datetime);

        // format!(
        //     "{}\n{}\n{}",
        //     mark.step_size,
        //     local.date_naive(),
        //     local.time()
        // )
        // .to_string()
        format!("{}", local.time()).to_string()
    }

    fn x_grid_spacer(grid: GridInput) -> Vec<GridMark> {
        log_grid_spacer(10)(grid)
    }

    fn show_plot(&mut self, ui: &mut egui::Ui, id: &PaneId) -> Response {
        let mut plot = Plot::new(format!("plot-{}", id.0))
            .legend(Legend::default())
            .label_formatter(Self::label_formatter)
            .x_axis_formatter(Self::x_axis_formatter)
            .x_grid_spacer(Self::x_grid_spacer)
            .show_background(false)
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
            let mut plot_bounds = self.plot_bounds.write();
            plot_bounds.replace(plot_ui.plot_bounds());
        })
        .response
    }
}
