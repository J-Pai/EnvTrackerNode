//! UI element for main panel.

use egui::Color32;
use egui::Frame;
use egui::Margin;
use egui_tiles::SimplificationOptions;
use egui_tiles::TileId;
use egui_tiles::Tiles;

use crate::app::kasa::BorrowPointsExample;

pub(super) type Tile = egui_tiles::Tree<Pane>;

/// Tile/Pane rendering.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub(super) struct Pane {
    nr: i32,
}

impl Pane {
    pub(super) fn new(nr: i32) -> Self {
        Self { nr }
    }
}

impl Pane {
    pub(super) fn ui(&self, ui: &mut egui::Ui) -> egui_tiles::UiResponse {
        let color = match ui.theme() {
            egui::Theme::Dark => egui::epaint::Hsva::new(0.0, 0.0, 0.025, 1.0),
            egui::Theme::Light => egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0),
        };
        egui::Panel::left(format!("data_panel{}", self.nr))
            .frame(Frame {
                fill: Color32::from(color),
                inner_margin: Margin::same(8),
                ..Frame::default()
            })
            .min_size(200.0)
            .max_size(200.0)
            .show(ui, |ui| {
                ui.label(format!("Pane {}", self.nr));
                ui.separator();
                // let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);
                // ui.painter().rect_filled(ui.max_rect(), 0.0, color);
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
                    .show(ui, |ui| {
                        // let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);
                        // ui.painter().rect_filled(ui.max_rect(), 0.0, color);
                        BorrowPointsExample::default().show_plot(ui, self.nr, false)
                    });
            });

        let dragged = ui
            .allocate_rect(ui.max_rect(), egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged();
        if dragged {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub(super) struct TileBehavior {
    reset: bool,
}

impl TileBehavior {
    pub(super) fn reset_plot(&mut self) {
        self.reset = true;
    }
}

impl egui_tiles::Behavior<Pane> for TileBehavior {
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

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        2.0
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        if self.reset {
            self.reset = false;
        }
        view.ui(ui)
    }

    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 1.0
    }
}
