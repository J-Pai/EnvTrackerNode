use egui::{Hyperlink, OpenUrl, Widget};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EnvApp {}

impl Default for EnvApp {
    fn default() -> Self {
        Self {}
    }
}

impl EnvApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
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

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("🏠 Home").clicked() {
                    ui.open_url(OpenUrl::same_tab("/"));
                };
                ui.separator();
                if ui.button("🏠 Graphs").clicked() {
                    ui.open_url(OpenUrl::same_tab("/"));
                };
                ui.separator();
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        Hyperlink::from_label_and_url("EnvTrackerNode", "https://github.com/J-Pai/EnvTrackerNode")
            .open_in_new_tab(true)
            .ui(ui);
        ui.label(" - ");
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
}
