//! Parses the config.toml file for system configuration.
//! - Also provides generator tool.

use std::fs;
use std::path::PathBuf;
use std::process::exit;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use url::Url;

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
        #[allow(clippy::result_large_err)]
        let config = fs::read_to_string(&path).map_or_else(
            |e| {
                println!(
                    "[ServerConfig] Error obtaining config file: {}. {}",
                    path.to_string_lossy(),
                    e,
                );
                Err(ServerConfig::default())
            },
            |v| {
                toml::from_str(&v).map_or_else(
                    |e| {
                        println!(
                            "[ServerConfig] Error parsing config file: {}. {}",
                            path.to_string_lossy(),
                            e,
                        );
                        Err(ServerConfig::default())
                    },
                    |v| {
                        if !edit_config {
                            return Ok(v);
                        }
                        Err(v)
                    },
                )
            },
        );

        match config {
            Ok(config) => config,
            #[allow(unused_variables)]
            Err(config) => {
                #[cfg(feature = "tui")]
                crate::config::creator::Creator::new(config)
                    .unwrap()
                    .create()
                    .unwrap()
                    .write(path);

                exit(0);
            }
        }
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

    pub(crate) fn override_frontend_base(&mut self, base: &str) {
        if let Some(frontend_config) = &mut self.frontend_server {
            frontend_config.base = Some(base.to_owned());
        }
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

#[allow(clippy::to_string_trait_impl)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OAuth2Config {
    /// Path to the client secret JSON. This can be obtained directly from
    /// GCP when creating a new client in the OAuth2 setup page.
    client_secret_json: String,
    /// Path to the Google Home client secret JSON. This can be obtained
    /// directly from GCP when creating a new client in the OAuth2 setup page.
    google_home_client_secret_json: Option<String>,
    /// OAuth2 callback/redirect base URI.
    redirect_uri_base: Url,
    /// OAuth2 base64 cookie encryption key.
    ///
    /// Can be generated using the following command:
    ///
    /// ```shell
    /// openssl rand -base64 64
    /// ```
    cookie_secret_key: Option<String>,
}

impl OAuth2Config {
    pub(crate) fn get_client_json(&self) -> PathBuf {
        PathBuf::from(self.client_secret_json.clone())
    }

    pub(crate) fn get_google_home_client_json(&self) -> Option<PathBuf> {
        if let Some(json) = self.google_home_client_secret_json.clone() {
            Some(PathBuf::from(json))
        } else {
            None
        }
    }

    pub(crate) fn get_redirect_uri_base(&self) -> Url {
        self.redirect_uri_base.clone()
    }

    pub(crate) fn get_cookie_secret_key(&self) -> Vec<u8> {
        let Some(cookie_secret_key) = &self.cookie_secret_key else {
            return vec![];
        };

        BASE64_STANDARD
            .decode(cookie_secret_key)
            .unwrap_or_else(|e| {
                tracing::warn!("Invalid cookie secret key: {e}");
                vec![]
            })
    }
}

/// API and Database server configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiServerConfig {
    /// List of nodes to poll and its polling schedule.
    nodes: Vec<NodeClass>,
    /// Path to database file (SQLite).
    db: String,
    /// OAuth2 Configuration.
    oauth2: Option<OAuth2Config>,
}

impl ApiServerConfig {
    pub(crate) fn get_db(&self) -> String {
        self.db.clone()
    }

    pub(crate) fn get_nodes(&self) -> &Vec<NodeClass> {
        &self.nodes
    }

    pub(crate) fn get_oauth2_config(&self) -> Option<OAuth2Config> {
        self.oauth2.clone()
    }
}

// Frontend configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct FrontendServerConfig {
    /// API server for data.
    api_server_uri: Url,
    /// Kasa API endpoint.
    kasa_api: String,
    /// Offset base URL.
    base: Option<String>,
}

impl FrontendServerConfig {
    pub(crate) fn get_api_server_uri(&self) -> Url {
        self.api_server_uri.clone()
    }

    pub(crate) fn get_kasa_api(&self) -> String {
        self.kasa_api.clone()
    }

    pub(crate) fn get_base(&self) -> Option<String> {
        self.base.clone()
    }
}

// Kasa device configuration.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct KasaDeviceConfig {
    uri: Url,
    username: String,
    password: String,
    batch_size: Option<usize>,
}

impl KasaDeviceConfig {
    pub(crate) fn get_uri(&self) -> Url {
        self.uri.clone()
    }

    pub(crate) fn get_username(&self) -> String {
        self.username.clone()
    }

    pub(crate) fn get_password(&self) -> String {
        self.password.clone()
    }

    pub(crate) fn get_batch_size(&self) -> Option<usize> {
        self.batch_size
    }
}

impl Default for KasaDeviceConfig {
    fn default() -> Self {
        Self {
            uri: Url::parse("http://192.168.0.1").unwrap(),
            username: String::from("username@email.com"),
            password: String::from("somepassword"),
            batch_size: None,
        }
    }
}

/// Supported node types.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) enum NodeClass {
    /// Kasa Device.
    /// Specific to the HS300 model for now.
    KasaDevice(String, Box<KasaDeviceConfig>, PollingConfig),
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
