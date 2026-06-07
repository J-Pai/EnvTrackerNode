//! TUI for creating new server configs.

use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use appcui::prelude::*;
use notify::poll;

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

        window
    }
}

impl ButtonEvents for CreatorWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if let Some(api_server) = &self.api_server
            && handle == api_server.save_button
        {
            let api_server = self.api_server.take().unwrap();
            let server_config = self.config.clone();
            let mut server_config = server_config.take();

            server_config.api_server = api_server.generate_config(self);

            self.config.replace(server_config);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if let Some(frontend_server) = &self.frontend_server
            && handle == frontend_server.save_button
        {
            return EventProcessStatus::Processed;
        }

        if let Some(node_server) = &self.node_server
            && handle == node_server.save_button
        {
            return EventProcessStatus::Processed;
        }

        if let Some(api_server) = &self.api_server
            && handle == api_server.add_node_button
        {
            let mut api_server = self.api_server.take().unwrap();
            api_server.add_node(self);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if let Some(api_server) = &self.api_server
            && handle == api_server.save_button
        {
            let api_server = self.api_server.take().unwrap();
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if let Some(api_server) = &self.api_server
            && handle == api_server.remove_nodes_button
        {
            let mut api_server = self.api_server.take().unwrap();
            api_server.remove_nodes(self);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        EventProcessStatus::Ignored
    }
}

struct NodeConfigUi {
    checkbox: Handle<CheckBox>,
    panel: Handle<Panel>,
    name: Handle<TextField>,
    ip: Handle<TextField>,
    polling_schedule: Handle<TextField>,
}

const NODE_HEIGHT: u16 = 7;
const NODE_START_Y: u16 = 2;

struct ApiServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    db_field: Handle<TextField>,
    node_panel: Handle<Panel>,
    add_node_button: Handle<Button>,
    remove_nodes_button: Handle<Button>,
    node_index: usize,
    node_configs: HashMap<usize, NodeConfigUi>,
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
        let remove_nodes = node_panel.add(button!("'Remove Nodes', x:32, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);

        Self {
            enable,
            save_button: save,
            db_field: db,
            node_panel: node_panel,
            add_node_button: add_node,
            remove_nodes_button: remove_nodes,
            node_index: 0,
            node_configs: HashMap::new(),
        }
    }

    fn add_node(&mut self, window: &mut CreatorWindow) {
        let index = self.node_configs.len() as u16;
        let node_panel = window.control_mut(self.node_panel).unwrap();
        let mut panel = Panel::new(
            format!("Node Config {}", self.node_index).as_str(),
            LayoutBuilder::new()
                .x(0)
                .y(NODE_HEIGHT * index + NODE_START_Y)
                .width(1.0)
                .height(NODE_HEIGHT)
                .build(),
        );
        let checkbox = checkbox!("'Node Name:', x:0, y:0, w:32");
        let checkbox = panel.add(checkbox);
        let name = panel.add(textfield!("caption='node_name', x:32, y:0, w: 32"));
        panel.add(label!("'IP:', x:0, y:2, w: 32"));
        let ip = panel.add(textfield!("caption='0.0.0.0:3000', x:32, y:2, w: 32"));
        panel.add(label!("'Polling Schedule:', x:0, y:4, w: 32"));
        let polling_schedule = panel.add(textfield!("caption='0 * * * * *', x:32, y:4, w: 32"));
        self.node_configs.insert(
            self.node_index,
            NodeConfigUi {
                checkbox,
                panel: node_panel.add(panel),
                name,
                ip,
                polling_schedule,
            },
        );
        self.node_index = self.node_index + 1;
        window.request_update();
    }

    fn remove_nodes(&mut self, window: &mut CreatorWindow) {
        let mut removal_index: Vec<usize> = vec![];

        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let remove = window.control(config.checkbox).unwrap();
                if remove.is_checked() {
                    let config = window.control_mut(config.panel).unwrap();
                    config.set_visible(false);
                    removal_index.push(*location);
                }
            }
        }

        for index in removal_index {
            self.node_configs.remove(&index);
        }

        let mut node_configs: Vec<(&usize, &NodeConfigUi)> = self.node_configs.iter().collect();
        node_configs.sort_by(|x, y| x.0.cmp(&y.0));

        for (index, (_, config)) in node_configs.iter().enumerate() {
            let repositioned = NODE_HEIGHT * index as u16 + NODE_START_Y;
            {
                let config = window.control_mut(config.panel).unwrap();
                config.set_position(0, repositioned as i32);
            }
        }

        window.request_update();
    }

    fn generate_config(&self, window: &mut CreatorWindow) -> Option<ApiServerConfig> {
        if let Some(enabled) = window.control(self.enable)
            && enabled.is_checked()
        {
            let db = if let Some(db) = window.control(self.db_field) {
                db.text().to_string()
            } else {
                return None;
            };

            let mut nodes: Vec<NodeDatasource> = vec![];

            for (_, (_, config)) in self.node_configs.iter().enumerate() {
                if let Some(_) = window.control(config.panel) {
                    let name = if let Some(name) = window.control(config.name) {
                        name.text().to_string()
                    } else {
                        return None;
                    };
                    let ip = if let Some(ip) = window.control(config.ip) {
                        Ip(ip.text().to_string())
                    } else {
                        return None;
                    };
                    let polling_schedule =
                        if let Some(polling_schedule) = window.control(config.polling_schedule) {
                            PollingSchedule(polling_schedule.text().to_string())
                        } else {
                            return None;
                        };
                    nodes.push(NodeDatasource(name, ip, polling_schedule));
                } else {
                    return None;
                };
            }

            return Some(ApiServerConfig { db, nodes });
        }

        None
    }
}

struct FrontendServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
}

impl FrontendServerUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        tabs.add(index, form_panel);

        Self {
            enable,
            save_button: save,
        }
    }
}

struct NodeUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
}

impl NodeUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        tabs.add(index, form_panel);

        Self {
            enable,
            save_button: save,
        }
    }
}
