//! Parses the config.toml file for system configuration.
//! - Also provides generator tool.

use std::fs;
use std::path::PathBuf;

/// Base configuration structure.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) api_server: Option<ApiServerConfig>,
    pub(crate) frontend_server: Option<FrontendServerConfig>,
    pub(crate) node: Option<Node>,
}

impl ServerConfig {
    pub(crate) fn new(path: PathBuf) -> Self {
        if let Ok(config_text) = fs::read_to_string(&path)
            && let Ok(config) = toml::from_str(&config_text)
        {
            return ServerConfig::from(config);
        }

        println!(
            "[ServerConfig] Error obtaining config file: {}",
            path.to_string_lossy()
        );

        let config = Self {
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

    pub(crate) fn get_node_config(&self) -> Option<Node> {
        self.node.clone()
    }

    pub(crate) fn get_frontend_config(&self) -> Option<FrontendServerConfig> {
        self.frontend_server.clone()
    }

    pub(crate) fn get_api_config(&self) -> Option<ApiServerConfig> {
        self.api_server.clone()
    }
}

/// IP address + port.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Ip(String);

/// Polling schedule using a cron-like string.
///
///   * * * * * * <command to execute>
/// # | | | | | |
/// # | | | | | day of the week (0–6) (Sunday to Saturday;
/// # | | | | month (1–12)             7 is also Sunday on some systems)
/// # | | | day of the month (1–31)
/// # | | hour (0–23)
/// # | minute (0–59)
/// # seconds (0-59)
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PollingSchedule(String);

impl ToString for PollingSchedule {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Node datasource configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct NodeDatasource(String, Ip, PollingSchedule);

/// API and Database server configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiServerConfig {
    /// List of nodes to poll and its polling schedule.
    nodes: Vec<NodeDatasource>,
    /// Path to database file (SQLite).
    db: String,
}

impl ApiServerConfig {
    pub(crate) fn get_db(&self) -> String {
        self.db.clone()
    }
}

// Frontend configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct FrontendServerConfig {
    /// API server for data.
    api_server_ip: Ip,
    /// Offset base URL.
    base: Option<String>,
}

impl FrontendServerConfig {
    pub(crate) fn get_base(&self) -> Option<String> {
        self.base.clone()
    }
}

// Kasa device configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct KasaDeviceConfig {
    ip: Ip,
    username: String,
    password: String,
}

impl KasaDeviceConfig {
    pub(crate) fn get_ip(&self) -> String {
        self.ip.0.clone()
    }

    pub(crate) fn get_username(&self) -> String {
        self.username.clone()
    }

    pub(crate) fn get_password(&self) -> String {
        self.password.clone()
    }
}

/// Supported node types.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum NodeClass {
    /// Kasa Device.
    /// Specific to the HS300 model for now.
    KasaDevice(String, KasaDeviceConfig, PollingSchedule),
}

/// Node configuration.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct Node {
    // List of IoT devices to interact with and their polling schedules.
    nodes: Vec<NodeClass>,
}

impl Node {
    pub(crate) fn get_nodes(&self) -> Vec<NodeClass> {
        self.nodes.clone()
    }
}
