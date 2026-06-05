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

pub(super) struct Creator(App);

impl Creator {
    pub(super) fn new() -> Result<Self, appcui::system::Error> {
        let mut app = App::new().single_window().build()?;
        let window = window!("'ServerConfig',dock:fill");
        app.add_window(window);
        Ok(Self(app))
    }

    /// Launches the creator TUI session.
    pub(super) fn create(self, path: PathBuf) -> ServerConfig {
        self.0.run();
        let config = ServerConfig {
            api_server: Some(ApiServerConfig {
                nodes: vec![NodeDatasource(
                    "kasa-power-strip".to_string(),
                    Ip("0.0.0.0:3000".to_string()),
                    PollingSchedule("0 * * * * *".to_string()),
                )],
                db: "sqlite.db".to_string(),
            }),
            frontend_server: Some(FrontendServerConfig {
                api_server_ip: Ip("0.0.0.0:3000".to_string()),
                base: None,
            }),
            node: Some(Node {
                nodes: vec![NodeClass::KasaDevice(
                    "kasa-power-strip".to_string(),
                    KasaDeviceConfig {
                        ip: Ip("0.0.0.0".to_string()),
                        username: "user".to_string(),
                        password: "password".to_string(),
                    },
                    PollingSchedule("*/1 * * * * *".to_string()),
                )],
            }),
        };

        let config_text =
            toml::to_string_pretty(&config).expect("Could not convert config to toml.");
        fs::write(&path, config_text).expect("Failed to write config file.");

        config
    }
}
