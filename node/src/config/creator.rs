//! TUI for creating new server configs.

use std::fs;
use std::path::PathBuf;

use appcui::prelude::*;

use crate::config::ApiServerConfig;
use crate::config::FrontendServerConfig;
use crate::config::Ip;
use crate::config::KasaDeviceConfig;
use crate::config::Node;
use crate::config::NodeClass;
use crate::config::NodeDatasource;
use crate::config::PollingSchedule;
use crate::config::ServerConfig;
use crate::error::NodeError;

pub(super) struct Creator {
    app: Option<App>,
    config: ServerConfig,
}

impl Creator {
    pub(super) fn new(config: ServerConfig) -> Result<Self, appcui::system::Error> {
        let mut app = App::new().single_window().build()?;
        let window = CreatorWindow::new();
        app.add_window(window);
        Ok(Self {
            app: Some(app),
            config,
        })
    }

    /// Launches the creator TUI session.
    pub(super) fn create(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let app = self
            .app
            .take()
            .ok_or(NodeError::new("Creator already used."))?;
        app.run();
        Ok(self)
    }

    pub(super) fn write(self, path: PathBuf) -> ServerConfig {
        let config_text =
            toml::to_string_pretty(&self.config).expect("Could not convert config to toml.");
        fs::write(&path, config_text).expect("Failed to write config file.");

        self.config
    }
}

#[Window(events = ButtonEvents)]
pub(super) struct CreatorWindow {}

impl CreatorWindow {
    fn new() -> Self {
        let mut window = Self {
            base: window!("'ServerConfig',dock:fill"),
        };

        let mut tabs =
            tab!("d:f, tabs: ['API Server', 'Frontend Server', 'Node'], tw: 32, type: OnTop");
        window = window.api_server(0, &mut tabs);
        window.add(tabs);

        window
    }

    fn api_server(self, tab_index: u32, tab: &mut Tab) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        form_panel.add(enable);
        let label = label!("'Database Path:', x:1, y:2, w: 14");
        form_panel.add(label);
        let db = textfield!("caption='sqlite.db', x:16, y:2, w:32");
        form_panel.add(db);

        tab.add(tab_index, form_panel);

        let mut node_panel = Panel::new("", layout!("x:50%, y:0, w: 50%, h: 100%"));

        let add_node = button!("'Add Node', x:1, y:0, w:16");
        node_panel.add(add_node);

        tab.add(tab_index, node_panel);

        self
    }
}

impl ButtonEvents for CreatorWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        tracing::debug!("Pressed! {:?}", handle);
        EventProcessStatus::Ignored
    }
}
