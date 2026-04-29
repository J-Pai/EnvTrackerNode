use egui::{Hyperlink, OpenUrl, Widget};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct State {
    control_panel: bool,
    graph: bool,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EnvApp {
    state: State,
}

impl Default for EnvApp {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for EnvApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
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
                .max_size(200.0)
                .min_size(200.0)
                .show_animated_inside(ui, self.state.control_panel, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        ui.vertical_centered(|ui| {
                            ui.heading("💻 Control Panel");
                        });
                        ui.separator();
                        ui.vertical_centered_justified(|ui| {
                            for i in 0..6 {
                                ui.add_space(4.0);
                                ui.toggle_value(&mut self.state.graph, format!("Graph {}", i));
                            }
                        });
                        ui.separator();
                    });
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(4.0);
                        powered_by_egui_and_eframe(ui);
                    });
                });
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
