use egui::Frame;
use egui::OpenUrl;
use fps::FrameHistory;
use tile::TileBehavior;

use crate::app::kasa::Kasa;
use crate::app::tile::Pane;

mod control_panel;
mod fps;
mod kasa;
mod tile;

/// Persistent state tracking.
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    control_panel: bool,
    continuous: bool,
    tiles: Option<tile::Tile>,
}

pub struct EnvApp {
    state: State,
    frame_history: FrameHistory,
    tile_behavior: TileBehavior,
    kasa: Kasa,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        _api_endpoint: String,
        kasa_api_endpoint: String,
    ) -> Self {
        let mut app = if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                frame_history: Default::default(),
                tile_behavior: TileBehavior::default(),
                kasa: Kasa::new(&kasa_api_endpoint),
            }
        } else {
            Self {
                state: Default::default(),
                frame_history: Default::default(),
                tile_behavior: TileBehavior::default(),
                kasa: Kasa::new(&kasa_api_endpoint),
            }
        };

        app.create_tree();

        app
    }
}

impl EnvApp {
    fn create_tree(&mut self) {
        if self.state.tiles.is_some() {
            return;
        }

        let mut next_view_nr = 0;
        let mut gen_pane = || {
            let pane = Pane::new(next_view_nr);
            next_view_nr += 1;
            pane
        };

        let mut tiles = egui_tiles::Tiles::default();

        let children = (0..6).map(|_| tiles.insert_pane(gen_pane())).collect();
        let root = tiles.insert_grid_tile(children);
        let tile = egui_tiles::Tree::new("root_tree", root, tiles);

        self.state.tiles = Some(tile);
    }

    fn reset_tree(&mut self) {
        self.state.tiles = None;
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
        self.kasa.logic();
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

        egui::CentralPanel::no_frame()
            .frame(Frame {
                fill: ui.theme().default_style().visuals.panel_fill,
                ..Frame::default()
            })
            .show(ui, |ui| {
                self.control_panel_ui(ui);
            });
    }
}
