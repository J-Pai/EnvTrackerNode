//! Parses the config.toml file for system configuration.
//! - Also provides generator tool.

use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[cfg(feature = "tui")]
mod creator;

/// Base configuration structure.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) api_server: Option<ApiServerConfig>,
    pub(crate) frontend_server: Option<FrontendServerConfig>,
    pub(crate) node: Option<Node>,
}

impl ServerConfig {
    pub(crate) fn new(path: PathBuf, edit_config: bool) -> Self {
        #[allow(unused_variables)]
        let config = if let Ok(config_text) = fs::read_to_string(&path)
            && let Ok(config) = toml::from_str(&config_text)
        {
            if !edit_config {
                return ServerConfig::from(config);
            }

            config
        } else {
            println!(
                "[ServerConfig] Error obtaining config file: {}",
                path.to_string_lossy()
            );

            ServerConfig::default()
        };

        #[cfg(feature = "tui")]
        crate::config::creator::Creator::new(config).unwrap().create().unwrap().write(path);

        exit(0);
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

impl Default for Ip {
    fn default() -> Self {
        Self(String::from("0.0.0.0:3000"))
    }
}

/// Polling configuration. How often and where to ping.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PollingConfig {
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
    schedule: String,
    /// API endpoint to use for polling operation.
    api: Option<String>,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            schedule: String::from("0 * * * * *"),
            api: None,
        }
    }
}

impl ToString for PollingConfig {
    fn to_string(&self) -> String {
        self.schedule.clone()
    }
}

impl PollingConfig {
    pub(crate) fn get_api(&self) -> Option<String> {
        self.api.clone()
    }

    pub(crate) fn get_schedule(&self) -> String {
        self.schedule.clone()
    }
}

/// API and Database server configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiServerConfig {
    /// List of nodes to poll and its polling schedule.
    nodes: Vec<NodeClass>,
    /// Path to database file (SQLite).
    db: String,
}

impl ApiServerConfig {
    pub(crate) fn get_db(&self) -> String {
        self.db.clone()
    }

    pub(crate) fn get_nodes(&self) -> &Vec<NodeClass> {
        &self.nodes
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
    pub(crate) fn get_api_server_ip(&self) -> Ip {
        self.api_server_ip.clone()
    }

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

impl Default for KasaDeviceConfig {
    fn default() -> Self {
        Self {
            ip: Ip(String::from("0.0.0.0:3001")),
            username: String::from("username@email.com"),
            password: String::from("somepassword"),
        }
    }
}

/// Supported node types.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) enum NodeClass {
    /// Kasa Device.
    /// Specific to the HS300 model for now.
    KasaDevice(String, KasaDeviceConfig, PollingConfig),
    /// Unknown device type.
    #[default]
    Unknown,
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
