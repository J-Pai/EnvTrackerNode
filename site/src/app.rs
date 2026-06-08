use egui::Hyperlink;
use egui::OpenUrl;
use egui::Response;
use egui::Widget;
use egui::util::History;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_tiles::SimplificationOptions;

#[allow(unused)]
macro_rules! console_log {
    ($expr:expr) => {
        web_sys::console::log_1(&web_sys::wasm_bindgen::JsValue::from_str($expr.as_str()));
    };
}

pub struct FrameHistory {
    frame_times: History<f32>,
}

impl Default for FrameHistory {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        let frame_times = History::new(0..max_len, max_age);
        Self { frame_times }
    }
}

impl FrameHistory {
    // Called first
    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previous_frame_time); // projected
    }

    pub fn mean_frame_time(&self) -> f32 {
        self.frame_times.average().unwrap_or_default()
    }

    pub fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.label(format!(
            "Mean CPU usage: {:.2} ms / frame",
            1e3 * self.mean_frame_time()
        ))
        .on_hover_text(
            "Includes all app logic, egui layout, tessellation, and rendering.\n\
            Does not include waiting for vsync.",
        );
        ui.label(format!("FPS: {:.2}", self.fps())).on_hover_text(
            "Includes all app logic, egui layout, tessellation, and rendering.\n\
            Does not include waiting for vsync.",
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
        let mut plot = Plot::new(format!("plot{}", nr))
            .legend(Legend::default())
            .width(ui.available_width() - 10.0);

        if reset {
            plot = plot.reset();
        }

        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new("curve", PlotPoints::Borrowed(&self.points)).name("curve"));
        })
        .response
    }
}

/// Tile/Pane rendering.
#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Pane {
    nr: i32,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct TreeBehavior {
    reset: bool,
}

impl TreeBehavior {
    fn reset_plot(&mut self) {
        self.reset = true;
    }
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        SimplificationOptions {
            prune_empty_tabs: true,
            prune_single_child_tabs: true,
            prune_empty_containers: true,
            prune_single_child_containers: true,
            all_panes_must_have_tabs: true,
            join_nested_linear_containers: true,
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            ui.add_space(10.0);
            BorrowPointsExample::default().show_plot(ui, pane.nr, self.reset);
        });

        Default::default()
    }

    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 1.5
    }
}

/// Persistent state tracking.
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    control_panel: bool,
    continuous: bool,
    tile_tree: Option<egui_tiles::Tree<Pane>>,
}

pub struct EnvApp {
    state: State,
    frame_history: FrameHistory,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                frame_history: Default::default(),
            }
        } else {
            Self {
                state: Default::default(),
                frame_history: Default::default(),
            }
        }
    }
}

impl EnvApp {
    fn create_tree(&mut self) {
        if self.state.tile_tree.is_some() {
            return;
        }

        let mut next_view_nr = 0;
        let mut gen_pane = || {
            let pane = Pane { nr: next_view_nr };
            next_view_nr += 1;
            pane
        };

        let mut tiles = egui_tiles::Tiles::default();

        let children = (0..6).map(|_| tiles.insert_pane(gen_pane())).collect();
        let root = tiles.insert_grid_tile(children);
        let tree = egui_tiles::Tree::new("root_tree", root, tiles);

        self.state.tile_tree = Some(tree);
    }

    fn reset_tree(&mut self) {
        self.state.tile_tree = None;
        self.create_tree();
    }
}

impl eframe::App for EnvApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        if self.state.continuous {
            ctx.request_repaint();
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut behavior = TreeBehavior::default();
        self.create_tree();

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("🏠 Home").clicked() {
                    ui.open_url(OpenUrl::same_tab("/"));
                };
                ui.separator();
                ui.toggle_value(&mut self.state.control_panel, "🖥 Control Panel");
                ui.separator();
            });
        });

        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            egui::Panel::left("control_panel")
                .resizable(false)
                .max_size(250.0)
                .min_size(250.0)
                .show_animated_inside(ui, self.state.control_panel, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        ui.vertical_centered(|ui| {
                            ui.heading("💻 Control Panel");
                        });
                        ui.separator();
                        if ui.button("Reset Tiles").clicked() {
                            self.reset_tree();
                            behavior.reset_plot();
                        };
                        ui.separator();
                        self.frame_history.ui(ui);
                        ui.checkbox(&mut self.state.continuous, "Run Mode - Continuous");
                        ui.separator();
                    });
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        powered_by_egui_and_eframe(ui);
                    });
                });

            if let Some(tree) = &mut self.state.tile_tree {
                tree.ui(&mut behavior, ui);
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        Hyperlink::from_label_and_url("egui", "https://github.com/emilk/egui")
            .open_in_new_tab(true)
            .ui(ui);
        ui.label(" and ");
        Hyperlink::from_label_and_url(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        )
        .open_in_new_tab(true)
        .ui(ui);
        ui.label(".");
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        Hyperlink::from_label_and_url("EnvTrackerNode", "https://github.com/J-Pai/EnvTrackerNode")
            .open_in_new_tab(true)
            .ui(ui);
        ui.label("  ");
        egui::warn_if_debug_build(ui);
    });
}
