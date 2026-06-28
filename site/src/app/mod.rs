use egui::Color32;
use egui::OpenUrl;
use egui::Response;
use egui::util::History;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_tiles::SimplificationOptions;

mod control_panel;

#[allow(clippy::allow_attributes)]
#[allow(unused)]
#[macro_export]
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
        let mut plot = Plot::new(format!("plot{nr}"))
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
        fn clear_color(visuals: &egui::Visuals) -> Color32 {
            // Give the area behind the floating windows a different color, because it looks better:
            let color = egui::lerp(
                egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
                0.0,
            );
            egui::Color32::from(color)
        }

        egui::CentralPanel::no_frame().show(ui, |ui| {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, clear_color(ui.visuals()));
            ui.separator();
            ui.add_space(10.0);
            BorrowPointsExample::default().show_plot(ui, pane.nr, self.reset);
            if self.reset {
                self.reset = false;
            }
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
    tree_behavior: TreeBehavior,
    api_endpoint: String,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, api_endpoint: String) -> Self {
        let mut app = if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                frame_history: Default::default(),
                tree_behavior: TreeBehavior::default(),
                api_endpoint,
            }
        } else {
            Self {
                state: Default::default(),
                frame_history: Default::default(),
                tree_behavior: TreeBehavior::default(),
                api_endpoint,
            }
        };

        app.create_tree();

        app
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

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        if self.state.continuous {
            ctx.request_repaint();
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("🏠 Home").clicked() {
                    ui.open_url(OpenUrl::same_tab("/"));
                }
                ui.separator();
                ui.toggle_value(&mut self.state.control_panel, "🖥 Control Panel");
                ui.separator();
            });
        });

        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.control_panel_ui(ui);
        });
    }
}
