use std::collections::HashMap;
use std::collections::HashSet;

use egui::Frame;
use egui::OpenUrl;
use egui_async::Bind;
use egui_tiles::Tree;
use fps::FrameHistory;
use tile::TileBehavior;

use crate::app::kasa::Kasa;
use crate::app::kasa::KasaDeviceChildAlias;
use crate::app::kasa::KasaDeviceChildId;
use crate::app::tile::Pane;
use crate::app::tile::PaneId;

mod control_panel;
mod fps;
mod kasa;
mod tile;

/// Persistent state tracking.
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct State {
    tiles: egui_tiles::Tree<Pane>,
    pane_ids: HashSet<PaneId>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tiles: EnvApp::new_tree(),
            pane_ids: HashSet::new(),
        }
    }
}

#[derive(Default)]
pub struct EnvApp {
    state: State,
    continuous: bool,
    control_panel: bool,
    frame_history: FrameHistory,
    tile_behavior: TileBehavior,
    kasa_api_endpoint: String,
    kasa_device_ids: Bind<Vec<(KasaDeviceChildId, KasaDeviceChildAlias)>, String>,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        api_endpoint: &str,
        kasa_api_endpoint: &str,
    ) -> Self {
        let kasa_api_endpoint = format!("{api_endpoint}/{kasa_api_endpoint}");
        let mut app = if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                kasa_api_endpoint,
                continuous: true,
                ..Self::default()
            }
        } else {
            Self {
                kasa_api_endpoint,
                continuous: true,
                ..Self::default()
            }
        };

        Kasa::request_device_ids(&mut app.kasa_device_ids, &app.kasa_api_endpoint);

        app
    }
}

impl EnvApp {
    fn new_tree() -> Tree<Pane> {
        let mut tiles = egui_tiles::Tiles::default();
        let root = tiles.insert_grid_tile(vec![]);
        egui_tiles::Tree::new("root_tree", root, tiles)
    }

    fn reset_tiles(&mut self) {
        self.state.tiles = Self::new_tree();
        self.state.pane_ids.clear();
        self.tile_behavior = TileBehavior::default();
        Kasa::request_device_ids(&mut self.kasa_device_ids, &self.kasa_api_endpoint);
    }

    fn reset_plots(&mut self) {
        self.tile_behavior.reset_plots();
    }

    fn load_tiles(&mut self) {
        if !self.tile_behavior.kasa_widgets_registered() {
            let mut widgets = HashMap::<PaneId, Kasa>::new();
            let devices = Kasa::read_device_ids(&mut self.kasa_device_ids);

            if !devices.is_empty() {
                for (id, alias) in devices {
                    let id = PaneId(id.0.clone());

                    if !self.state.pane_ids.contains(&id) {
                        let tile_id = self
                            .state
                            .tiles
                            .tiles
                            .insert_pane(Pane::new(id.clone(), alias.0));

                        if let Some(root_tile_id) = self.state.tiles.root() {
                            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Grid(
                                root,
                            ))) = self.state.tiles.tiles.get_mut(root_tile_id)
                            {
                                root.add_child(tile_id);
                            }
                        } else {
                            self.state.tiles.root =
                                Some(self.state.tiles.tiles.insert_grid_tile(vec![tile_id]));
                        }
                    }

                    widgets.insert(id.clone(), Kasa::new(&self.kasa_api_endpoint));
                    self.state.pane_ids.insert(id);
                }

                self.tile_behavior.register_kasa_widgets(widgets);
            }
        }
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

        if self.continuous {
            ctx.request_repaint();
        }

        self.load_tiles();
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
                ui.toggle_value(&mut self.control_panel, "🖥 Control Panel");
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
                self.state.tiles.ui(&mut self.tile_behavior, ui);
            });
    }
}

trait EnvWidget {
    fn ui(&mut self, ui: &mut egui::Ui, id: &PaneId, alias: &str) -> egui_tiles::UiResponse;
}
