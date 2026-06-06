//! TUI for creating new server configs.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

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
    config: Rc<ServerConfig>,
}

impl Creator {
    pub(super) fn new(config: ServerConfig) -> Result<Self, appcui::system::Error> {
        let mut app = App::new().single_window().build()?;
        let config = Rc::new(config);
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
        let config_text = toml::to_string_pretty(self.config.as_ref())
            .expect("Could not convert config to toml.");
        fs::write(&path, config_text).expect("Failed to write config file.");

        self.config.as_ref().clone()
    }
}

#[Window(events = ButtonEvents)]
struct CreatorWindow {
    config: Rc<ServerConfig>,
    api_server: Option<ApiServerUi>,
    frontend_server: Option<FrontendServerUi>,
    node_server: Option<NodeUi>,
}

impl CreatorWindow {
    fn new(config: Rc<ServerConfig>) -> Self {
        let mut tabs =
            tab!("d:f, tabs: ['API Server', 'Frontend Server', 'Node'], tw: 32, type: OnTop");

        let mut window = Self {
            config,
            base: window!("'ServerConfig',dock:fill"),
            api_server: Some(ApiServerUi::new(&mut tabs, 0)),
            frontend_server: Some(FrontendServerUi::new(&mut tabs, 1)),
            node_server: Some(NodeUi::new(&mut tabs, 2)),
        };

        window.add(tabs);

        window
    }
}

impl ButtonEvents for CreatorWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.api_server.as_ref().unwrap().add_node_button {
            let mut api_server = self.api_server.take().unwrap();
            api_server.add_node(self);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if handle == self.api_server.as_ref().unwrap().save_button {
            let mut api_server = self.api_server.take().unwrap();
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        EventProcessStatus::Ignored
    }
}

struct ApiServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    db_field: Handle<TextField>,
    node_panel: Handle<Panel>,
    add_node_button: Handle<Button>,
    node_configs: Vec<(Handle<Button>, Handle<Panel>)>,
}

impl ApiServerUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        let label = label!("'Database Path:', x:1, y:2, w: 14");
        form_panel.add(label);
        let db = form_panel.add(textfield!("caption='sqlite.db', x:32, y:2, w:32"));
        tabs.add(index, form_panel);

        let mut node_panel = Panel::new("", layout!("x:50%, y:0, w: 50%, h: 100%"));
        let add_node = node_panel.add(button!("'Add Node', x:1, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);

        Self {
            enable,
            save_button: save,
            db_field: db,
            node_panel: node_panel,
            add_node_button: add_node,
            node_configs: vec![],
        }
    }

    fn add_node(&mut self, window: &mut CreatorWindow) {
        const NODE_HEIGHT: u16 = 7;
        const NODE_START_Y: u16 = 2;
        let index = self.node_configs.len() as u16;
        let node_panel = window.control_mut(self.node_panel).unwrap();
        let mut panel = Panel::new(
            format!("Node {}", index).as_str(),
            layout!("x: 0, y: 2, w: 100%, h: 5"),
        );
        panel.set_position(0, (NODE_HEIGHT * index + NODE_START_Y) as i32);
        panel.set_size(node_panel.size().width as u16 - 2, NODE_HEIGHT);
        panel.add(label!("'Node Name:', x:0, y:0, w: 32"));
        panel.add(textfield!("caption='node_name', x:32, y:0, w: 32"));
        panel.add(label!("'IP:', x:0, y:1, w: 32"));
        panel.add(textfield!("caption='0.0.0.0:3000', x:32, y:1, w: 32"));
        panel.add(label!("'Polling Schedule:', x:0, y:2, w: 32"));
        panel.add(textfield!("caption='0 * * * * *', x:32, y:2, w: 32"));
        let button = button!("Remove, x:0, y:3, w:10");
        self.node_configs
            .push((panel.add(button), node_panel.add(panel)));
        window.request_update();
    }
}

struct FrontendServerUi {
    enable: Handle<CheckBox>,
}

impl FrontendServerUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        tabs.add(index, form_panel);

        Self { enable }
    }
}

struct NodeUi {
    enable: Handle<CheckBox>,
}

impl NodeUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        tabs.add(index, form_panel);

        Self { enable }
    }
}
