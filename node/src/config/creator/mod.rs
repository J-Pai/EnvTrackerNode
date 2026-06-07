//! TUI for creating new server configs.

use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use appcui::prelude::*;

use crate::config::ServerConfig;
use crate::error::NodeError;
use api::ApiServerUi;
use frontend::FrontendServerUi;
use node::NodeUi;

mod api;
mod frontend;
mod node;

pub(super) struct Creator {
    app: Option<App>,
    config: Rc<Cell<ServerConfig>>,
}

impl Creator {
    pub(super) fn new(config: ServerConfig) -> Result<Self, appcui::system::Error> {
        let mut app = App::new().single_window().build()?;
        let config = Rc::new(Cell::new(config));
        let window = CreatorWindow::new(config.clone());
        app.add_window(window);
        let app = Self {
            app: Some(app),
            config,
        };
        Ok(app)
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
        let config = self.config.as_ref().take();

        let config_text =
            toml::to_string_pretty(&config).expect("Could not convert config to toml.");
        fs::write(&path, config_text).expect("Failed to write config file.");

        config
    }
}

#[Window(events = ButtonEvents)]
struct CreatorWindow {
    config: Rc<Cell<ServerConfig>>,
    api_server: Option<ApiServerUi>,
    frontend_server: Option<FrontendServerUi>,
    node_server: Option<NodeUi>,
}

impl CreatorWindow {
    fn new(config: Rc<Cell<ServerConfig>>) -> Self {
        let mut tabs =
            tab!("d:f, tabs: ['API Server', 'Frontend Server', 'Node'], tw: 32, type: OnTop");

        let mut window = Self {
            config: config,
            base: window!("'ServerConfig',dock:fill"),
            api_server: Some(ApiServerUi::new(&mut tabs, 0)),
            frontend_server: Some(FrontendServerUi::new(&mut tabs, 1)),
            node_server: Some(NodeUi::new(&mut tabs, 2)),
        };
        window.add(tabs);
        let config = window.config.as_ref().take();

        let mut api_server = window.api_server.take().unwrap();
        api_server.restore_config(&mut window, config.get_api_config());
        window.api_server.replace(api_server);

        let mut frontend_server = window.frontend_server.take().unwrap();
        frontend_server.restore_config(&mut window, config.get_frontend_config());
        window.frontend_server.replace(frontend_server);

        let mut node_server = window.node_server.take().unwrap();
        node_server.restore_config(&mut window, config.get_node_config());
        window.node_server.replace(node_server);

        window.config.as_ref().replace(config);
        window
    }
}

impl ButtonEvents for CreatorWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if let Some(mut api_server) = self.api_server.take() {
            let mut server_config = self.config.take();

            match api_server.button_handler(self, &mut server_config, handle) {
                Ok(event) => {
                    self.config.replace(server_config);
                    self.api_server.replace(api_server);
                    return event;
                }
                Err(_) => {},
            }

            self.config.replace(server_config);
            self.api_server.replace(api_server);
        }

        if let Some(mut frontend_server) = self.frontend_server.take() {
            let mut server_config = self.config.take();

            match frontend_server.button_handler(self, &mut server_config, handle) {
                Ok(event) => {
                    self.config.replace(server_config);
                    self.frontend_server.replace(frontend_server);
                    return event;
                }
                Err(_) => {},
            }

            self.config.replace(server_config);
            self.frontend_server.replace(frontend_server);
        }

        if let Some(mut node_server) = self.node_server.take() {
            let mut server_config = self.config.take();

            match node_server.button_handler(self, &mut server_config, handle) {
                Ok(event) => {
                    self.config.replace(server_config);
                    self.node_server.replace(node_server);
                    return event;
                }
                Err(_) => {},
            }

            self.config.replace(server_config);
            self.node_server.replace(node_server);
        }

        EventProcessStatus::Ignored
    }
}
