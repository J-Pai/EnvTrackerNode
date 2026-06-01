use egui::Hyperlink;
use egui::OpenUrl;
use egui::Widget;
use egui::util::History;
use egui_tiles::SimplificationOptions;

/// Persistent state tracking.
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    control_panel: bool,
    tile_tree: Option<egui_tiles::Tree<Pane>>,
}

pub struct EnvApp {
    state: State,
    frame_time: History<f32>,
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        let frame_time = History::new(0..max_len, max_age);

        if let Some(storage) = cc.storage {
            Self {
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
                frame_time,
            }
        } else {
            Self {
                state: Default::default(),
                frame_time,
            }
        }
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Pane {
    nr: i32,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct TreeBehavior {}

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
        if pane.nr == -1 {
            return egui_tiles::UiResponse::None;
        }

        let color = egui::lerp(
            egui::Rgba::from(ui.visuals().panel_fill)
                ..=egui::Rgba::from(ui.visuals().code_bg_color),
            0.5,
        );

        ui.painter().rect_filled(ui.max_rect(), 0.0, color);

        ui.label(format!("The contents of pane {}.", pane.nr));

        Default::default()
    }

    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 1.5
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
                        if ui.button("Reset").clicked() {
                            self.reset_tree();
                        };
                        ui.separator();
                        ui.label("Mean CPU usage: 00.00 ms / frame");
                        ui.separator();
                    });
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        powered_by_egui_and_eframe(ui);
                    });
                });

            let mut behavior = TreeBehavior {};
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
