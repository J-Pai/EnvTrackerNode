//! UI element for control panel.

use egui::Hyperlink;
use egui::Widget as _;

use crate::EnvApp;

impl EnvApp {
    pub(super) fn control_panel_ui(&mut self, ui: &mut egui::Ui) {
        let mut reset = false;

        egui::Panel::left("control_panel")
            .resizable(false)
            .max_size(250.0)
            .min_size(250.0)
            .show_collapsible(ui, &mut self.control_panel, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("💻 Control Panel");
                    });
                    ui.separator();
                    if ui.button("Reset Tiles").clicked() {
                        reset = true;
                    }
                    ui.separator();
                    self.frame_history.ui(ui);
                    ui.checkbox(&mut self.continuous, "Continuous");
                    ui.separator();
                });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(4.0);
                    Self::app_information(ui);
                });
            });

        if reset {
            self.reset_tiles();
        }
    }

    fn app_information(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            Hyperlink::from_label_and_url(
                "EnvTrackerNode",
                "https://github.com/J-Pai/EnvTrackerNode",
            )
            .open_in_new_tab(true)
            .ui(ui);
            ui.label("  ");
            egui::warn_if_debug_build(ui);
        });
    }
}
