use egui::OpenUrl;
use tile::Tile;
use tile::TileBehavior;
use fps::FrameHistory;

mod control_panel;
mod fps;
mod tile;

#[allow(clippy::allow_attributes)]
#[allow(unused)]
#[macro_export]
macro_rules! console_log {
    ($expr:expr) => {
        web_sys::console::log_1(&web_sys::wasm_bindgen::JsValue::from_str($expr.as_str()));
    };
}

/// Persistent state tracking.
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    control_panel: bool,
    continuous: bool,
    tile_tree: Option<tile::TileTree>,
}

pub struct EnvApp {
    state: State,
    frame_history: FrameHistory,
    tile_behavior: TileBehavior,
    api_endpoint: String,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, api_endpoint: String) -> Self {
        let mut app = if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                frame_history: Default::default(),
                tile_behavior: TileBehavior::default(),
                api_endpoint,
            }
        } else {
            Self {
                state: Default::default(),
                frame_history: Default::default(),
                tile_behavior: TileBehavior::default(),
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
            let pane = Tile::new(next_view_nr);
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
