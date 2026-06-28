//! UI element for main panel.

use egui::Color32;
use egui::Response;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_tiles::SimplificationOptions;

pub(super) type TileTree = egui_tiles::Tree<Tile>;

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
pub(super) struct Tile {
    nr: i32,
}

impl Tile {
    pub(super) fn new(nr: i32) -> Self {
        Self { nr }
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

impl egui_tiles::Behavior<Tile> for TileBehavior {
    fn tab_title_for_pane(&mut self, pane: &Tile) -> egui::WidgetText {
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
        tile: &mut Tile,
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
            BorrowPointsExample::default().show_plot(ui, tile.nr, self.reset);
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
