//! UI element for main panel.

use std::collections::HashMap;

use egui_tiles::SimplificationOptions;
use egui_tiles::TileId;
use egui_tiles::Tiles;

use crate::app::EnvWidget;
use crate::app::Kasa;

#[derive(Clone, Default, Eq, PartialEq, Hash, serde::Deserialize, serde::Serialize)]
pub(super) struct PaneId(pub(super) String);

/// Tile/Pane rendering.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub(super) struct Pane {
    id: PaneId,
    alias: String,
}

impl Pane {
    pub(super) fn new(id: PaneId, alias: String) -> Self {
        Self { id, alias }
    }

    pub(super) fn ui(
        &self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        widget: Option<&mut dyn EnvWidget>,
    ) -> egui_tiles::UiResponse {
        let Some(widget) = widget else {
            let dragged = ui
                .allocate_rect(ui.max_rect(), egui::Sense::click_and_drag())
                .on_hover_cursor(egui::CursorIcon::Grab)
                .dragged();
            if dragged {
                return egui_tiles::UiResponse::DragStarted;
            } else {
                return egui_tiles::UiResponse::None;
            }
        };

        widget.ui(ui, &self.id, &self.alias)
    }
}

pub(super) struct TileBehavior {
    kasa_widgets: Option<HashMap<PaneId, Kasa>>,
}

impl Default for TileBehavior {
    fn default() -> Self {
        Self { kasa_widgets: None }
    }
}

impl TileBehavior {
    pub(super) fn register_kasa_widgets(&mut self, widget: HashMap<PaneId, Kasa>) {
        self.kasa_widgets.replace(widget);
    }

    pub(super) fn kasa_widgets_registered(&self) -> bool {
        self.kasa_widgets.is_some()
    }
}

impl egui_tiles::Behavior<Pane> for TileBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.alias.into()
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
        tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        if let Some(kasa_widgets) = &mut self.kasa_widgets {
            if let Some(widget) = kasa_widgets.get_mut(&view.id) {
                return view.ui(ui, tile_id, Some(widget));
            }
        }
        view.ui(ui, tile_id, None)
    }

    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 1.0
    }
}
