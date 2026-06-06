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
        let mut window = window!("'ServerConfig',dock:fill");
        Self::tabs(&mut window);
        app.add_window(window);
        Ok(Self {
            app: Some(app),
            config,
        })
    }

    fn tabs(window: &mut Window) {
        let mut tab = tab!("d:f, tabs: ['API Server', 'Frontend Server', 'Node'], tw: 32, type: OnTop");
        Self::api_server(0, &mut tab);
        window.add(tab);
    }

    fn api_server(tab_index: u32, tab: &mut Tab) {
        let mut panel = panel!("x:0, y:0, w: 100%, h: 100%, type:Border");

        let enable = checkbox!("'Enable Service', x:1, y:1, w:32");

        panel.add(enable);
        let label = label!("'Database Path:', x:1, y:3, w: 14");
        panel.add(label);
        let db = textfield!("caption='sqlite.db', x:16, y:3, w:32");
        panel.add(db);

        tab.add(tab_index, panel);
    }

    /// Launches the creator TUI session.
    pub(super) fn create(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let app = self.app.take().ok_or(NodeError::new("Creator already used."))?;
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
