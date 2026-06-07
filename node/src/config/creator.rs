//! TUI for creating new server configs.

use std::cell::Cell;
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
        if let Some(api_server) = &self.api_server
            && handle == api_server.save_button
        {
            let api_server = self.api_server.take().unwrap();
            let mut server_config = self.config.take();

            server_config.api_server = api_server.generate_config(self);

            self.config.replace(server_config);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if let Some(frontend_server) = &self.frontend_server
            && handle == frontend_server.save_button
        {
            let frontend_server = self.frontend_server.take().unwrap();
            let mut server_config = self.config.take();

            server_config.frontend_server = frontend_server.generate_config(self);

            self.config.replace(server_config);
            self.frontend_server.replace(frontend_server);
            return EventProcessStatus::Processed;
        }

        if let Some(node_server) = &self.node_server
            && handle == node_server.save_button
        {
            let node_server = self.node_server.take().unwrap();
            let mut server_config = self.config.take();

            server_config.node = node_server.generate_config(self);

            self.config.replace(server_config);
            self.node_server.replace(node_server);
            return EventProcessStatus::Processed;
        }

        if let Some(api_server) = &self.api_server
            && handle == api_server.add_node_button
        {
            let mut api_server = self.api_server.take().unwrap();

            let data = NodeDatasource(
                self.control(api_server.node_editor_panel.name)
                    .unwrap()
                    .text()
                    .to_string(),
                Ip(self
                    .control(api_server.node_editor_panel.ip)
                    .unwrap()
                    .text()
                    .to_string()),
                PollingSchedule(
                    self.control(api_server.node_editor_panel.polling_schedule)
                        .unwrap()
                        .text()
                        .to_string(),
                ),
            );

            api_server.add_node(self, data);
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

        if let Some(api_server) = &self.api_server
            && handle == api_server.update_node_button
        {
            let mut api_server = self.api_server.take().unwrap();

            let data = NodeDatasource(
                self.control(api_server.node_editor_panel.name)
                    .unwrap()
                    .text()
                    .to_string(),
                Ip(self
                    .control(api_server.node_editor_panel.ip)
                    .unwrap()
                    .text()
                    .to_string()),
                PollingSchedule(
                    self.control(api_server.node_editor_panel.polling_schedule)
                        .unwrap()
                        .text()
                        .to_string(),
                ),
            );

            api_server.update_nodes(self, data);
            self.api_server.replace(api_server);
            return EventProcessStatus::Processed;
        }

        if let Some(node_server) = &self.node_server
            && handle == node_server.add_node_button
        {
            let mut node_server = self.node_server.take().unwrap();
            let data = if let Some(node_class) =
                self.control(node_server.node_editor_panel.dropdown.unwrap())
                && let Some(node_class) = node_class.selected_item()
            {
                match node_class {
                    NodeClass::KasaDevice(_, _, _) => NodeClass::KasaDevice(
                        self.control(node_server.node_editor_panel.name)
                            .unwrap()
                            .text()
                            .to_string(),
                        KasaDeviceConfig {
                            ip: Ip(self
                                .control(node_server.node_editor_panel.ip)
                                .unwrap()
                                .text()
                                .to_string()),
                            username: self
                                .control(node_server.node_editor_panel.username)
                                .unwrap()
                                .text()
                                .to_string(),
                            password: self
                                .control(node_server.node_editor_panel.password)
                                .unwrap()
                                .text()
                                .to_string(),
                        },
                        PollingSchedule(
                            self.control(node_server.node_editor_panel.polling_schedule)
                                .unwrap()
                                .text()
                                .to_string(),
                        ),
                    ),
                    NodeClass::Unknown => NodeClass::default(),
                }
            } else {
                NodeClass::default()
            };

            node_server.add_node(self, data);
            self.node_server.replace(node_server);
            return EventProcessStatus::Processed;
        }

        if let Some(node_server) = &self.node_server
            && handle == node_server.remove_nodes_button
        {
            let mut node_server = self.node_server.take().unwrap();
            node_server.remove_nodes(self);
            self.node_server.replace(node_server);
            return EventProcessStatus::Processed;
        }

        EventProcessStatus::Ignored
    }
}

struct NodeConfigUi {
    checkbox: Option<Handle<CheckBox>>,
    add: Option<Handle<Button>>,
    update: Option<Handle<Button>>,
    panel: Handle<Panel>,
    name: Handle<TextField>,
    ip: Handle<TextField>,
    polling_schedule: Handle<TextField>,
    height: u16,
}

impl NodeConfigUi {
    const NODE_HEIGHT: u16 = 10;

    fn new(
        data: NodeDatasource,
        panel: &mut Panel,
        editor: bool,
        id: String,
        start_y: u16,
        y_multiplier: u16,
    ) -> Self {
        let smaller = if editor { 0 } else { 2 };
        let height = Self::NODE_HEIGHT - smaller;

        let mut node_editor_panel = Panel::new(
            &format!("Node Config{}", id),
            LayoutBuilder::new()
                .x(0)
                .y((Self::NODE_HEIGHT - smaller) * y_multiplier + start_y)
                .width(1.0)
                .height(height)
                .build(),
        );

        let checkbox = if editor {
            let name_label = label!("'Node Name:', x:0, y:0, w:32");
            node_editor_panel.add(name_label);
            None
        } else {
            let checkbox = checkbox!("'Node Name:', x:0, y:0, w:32");
            Some(node_editor_panel.add(checkbox))
        };
        let mut name = textfield!("caption='node_name', x:32, y:0, w: 32");
        name.set_text(&data.0);
        name.set_enabled(editor);
        let name = node_editor_panel.add(name);
        node_editor_panel.add(label!("'IP:', x:0, y:2, w: 32"));
        let mut ip = textfield!("caption='0.0.0.0:3000', x:32, y:2, w: 32");
        ip.set_text(&data.1.0);
        ip.set_enabled(editor);
        let ip = node_editor_panel.add(ip);
        node_editor_panel.add(label!("'Polling Schedule:', x:0, y:4, w: 32"));
        let mut polling_schedule = textfield!("caption='0 * * * * *', x:32, y:4, w: 32");
        polling_schedule.set_text(&data.2.0);
        polling_schedule.set_enabled(editor);
        let polling_schedule = node_editor_panel.add(polling_schedule);
        let (add, update) = if editor {
            let add = button!("'Add', x:0, y: 6, w:16");
            let update = button!("'Update', x:32, y: 6, w: 16");
            (
                Some(node_editor_panel.add(add)),
                Some(node_editor_panel.add(update)),
            )
        } else {
            (None, None)
        };

        let node_editor_panel = panel.add(node_editor_panel);

        Self {
            checkbox: checkbox,
            add,
            update,
            panel: node_editor_panel,
            name,
            ip,
            polling_schedule,
            height,
        }
    }
}

struct ApiServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    db_field: Handle<TextField>,
    node_editor_panel: NodeConfigUi,
    node_panel: Handle<Panel>,
    update_node_button: Handle<Button>,
    add_node_button: Handle<Button>,
    remove_nodes_button: Handle<Button>,
    node_index: usize,
    node_configs: HashMap<usize, NodeConfigUi>,
}

impl ApiServerUi {
    const NODE_START_Y: u16 = 2;

    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        let label = label!("'Database Path:', x:1, y:2, w: 14");
        form_panel.add(label);
        let db = form_panel.add(textfield!("caption='sqlite.db', x:32, y:2, w:32"));

        let mut node_editor_panel = NodeConfigUi::new(
            NodeDatasource::default(),
            &mut form_panel,
            true,
            String::new(),
            4,
            0,
        );

        tabs.add(index, form_panel);

        let mut node_panel = Panel::new("", layout!("x:50%, y:0, w: 50%, h: 100%"));
        let remove_nodes = node_panel.add(button!("'Remove Nodes', x:1, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);
        let add_node = node_editor_panel.add.take().unwrap();
        let update_node = node_editor_panel.update.take().unwrap();

        Self {
            enable,
            save_button: save,
            db_field: db,
            node_editor_panel: node_editor_panel,
            node_panel: node_panel,
            update_node_button: update_node,
            add_node_button: add_node,
            remove_nodes_button: remove_nodes,
            node_index: 0,
            node_configs: HashMap::new(),
        }
    }

    fn add_node(&mut self, window: &mut CreatorWindow, data: NodeDatasource) {
        let index = self.node_configs.len() as u16;
        let node_panel = window.control_mut(self.node_panel).unwrap();
        self.node_configs.insert(
            self.node_index,
            NodeConfigUi::new(
                data,
                node_panel,
                false,
                format!(" {}", self.node_index),
                2,
                index,
            ),
        );
        self.node_index = self.node_index + 1;
        window.request_update();
    }

    fn remove_nodes(&mut self, window: &mut CreatorWindow) {
        let mut removal_index: Vec<usize> = vec![];

        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let remove = window.control(config.checkbox.unwrap()).unwrap();
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
            let repositioned = config.height * index as u16 + Self::NODE_START_Y;
            {
                let config = window.control_mut(config.panel).unwrap();
                config.set_position(0, repositioned as i32);
            }
        }

        window.request_update();
    }

    fn update_nodes(&mut self, window: &mut CreatorWindow, data: NodeDatasource) {
        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let update = window.control(config.checkbox.unwrap()).unwrap();
                if update.is_checked() {
                    let name = window.control_mut(config.name).unwrap();
                    name.set_text(&data.0);
                    let ip = window.control_mut(config.ip).unwrap();
                    ip.set_text(&data.1.0);
                    let polling_schedule = window.control_mut(config.polling_schedule).unwrap();
                    polling_schedule.set_text(&data.2.0);
                }
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

    fn restore_config(&mut self, window: &mut CreatorWindow, config: Option<ApiServerConfig>) {
        let config = if let Some(config) = config {
            config
        } else {
            return;
        };

        if let Some(db) = window.control_mut(self.db_field) {
            db.set_text(&config.get_db());
        }

        for node in config.nodes {
            self.add_node(window, node);
        }

        if let Some(enable) = window.control_mut(self.enable) {
            enable.set_checked(true);
        }
    }
}

struct FrontendServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    ip_field: Handle<TextField>,
    base_field: Handle<TextField>,
}

impl FrontendServerUi {
    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);

        form_panel.add(label!("'API Server IP:', x:0, y:2, w: 32"));
        let ip = textfield!("caption='0.0.0.0:3000', x:32, y:2, w: 32");
        let ip = form_panel.add(ip);
        form_panel.add(label!("'Base:', x:0, y:4, w: 32"));
        let base = textfield!("caption='', x:32, y:4, w: 32");
        let base = form_panel.add(base);

        tabs.add(index, form_panel);

        Self {
            enable,
            save_button: save,
            ip_field: ip,
            base_field: base,
        }
    }

    fn generate_config(&self, window: &mut CreatorWindow) -> Option<FrontendServerConfig> {
        if let Some(enabled) = window.control(self.enable)
            && enabled.is_checked()
        {
            let api_server_ip = if let Some(ip) = window.control(self.ip_field) {
                Ip(ip.text().to_string())
            } else {
                return None;
            };

            let base = if let Some(base) = window.control(self.base_field) {
                let base = base.text().trim();

                if base.is_empty() {
                    None
                } else {
                    Some(base.to_string())
                }
            } else {
                return None;
            };

            return Some(FrontendServerConfig {
                api_server_ip,
                base,
            });
        }

        None
    }

    fn restore_config(&mut self, window: &mut CreatorWindow, config: Option<FrontendServerConfig>) {
        let config = if let Some(config) = config {
            config
        } else {
            return;
        };

        if let Some(ip) = window.control_mut(self.ip_field) {
            ip.set_text(&config.get_api_server_ip().0);
        }

        if let Some(base) = window.control_mut(self.base_field)
            && let Some(base_str) = config.get_base()
        {
            base.set_text(&base_str);
        }

        if let Some(enable) = window.control_mut(self.enable) {
            enable.set_checked(true);
        }
    }
}

impl DropDownListType for NodeClass {
    fn name(&self) -> &str {
        match self {
            NodeClass::KasaDevice(_, _, _) => "Kasa Device",
            NodeClass::Unknown => "Unknown",
        }
    }

    fn description(&self) -> &str {
        match self {
            NodeClass::KasaDevice(_, _, _) => "Communicate with a Kasa Device",
            NodeClass::Unknown => "What is this?",
        }
    }

    fn symbol(&self) -> &str {
        ""
    }
}

struct NodeDeviceConfigUi {
    checkbox: Option<Handle<CheckBox>>,
    add: Option<Handle<Button>>,
    update: Option<Handle<Button>>,
    dropdown: Option<Handle<DropDownList<NodeClass>>>,
    panel: Handle<Panel>,
    name: Handle<TextField>,
    ip: Handle<TextField>,
    username: Handle<TextField>,
    password: Handle<TextField>,
    polling_schedule: Handle<TextField>,
    node_class: NodeClass,
    height: u16,
}

impl NodeDeviceConfigUi {
    const NODE_HEIGHT: u16 = 16;

    fn new(
        data: NodeClass,
        panel: &mut Panel,
        editor: bool,
        key: String,
        start_y: u16,
        y_multiplier: u16,
    ) -> Self {
        let (id, config, schedule) = if let NodeClass::KasaDevice(id, config, schedule) = data {
            (id, config, schedule)
        } else {
            unreachable!();
        };

        let smaller = if editor { 0 } else { 2 };
        let height = Self::NODE_HEIGHT - smaller;

        let info = if id.is_empty() {
            "(Kasa Device)"
        } else {
            &format!("{} (Kasa Device", id)
        };

        let mut node_panel = Panel::new(
            format!("Node Config{} - {}", key, info).as_str(),
            LayoutBuilder::new()
                .x(0)
                .y(height * y_multiplier + start_y)
                .width(1.0)
                .height(height)
                .build(),
        );

        let checkbox = if editor {
            let name_label = label!("'Node Name:', x:0, y:0, w:32");
            node_panel.add(name_label);
            None
        } else {
            let checkbox = checkbox!("'Node Name:', x:0, y:0, w:32");
            Some(node_panel.add(checkbox))
        };
        let mut name = textfield!("caption='node_name', x:32, y:0, w: 32");
        name.set_text(&id);
        name.set_enabled(editor);
        let name = node_panel.add(name);
        node_panel.add(label!("'IP:', x:0, y:2, w: 32"));
        let mut ip = textfield!("caption='0.0.0.0:3000', x:32, y:2, w: 32");
        ip.set_text(&config.get_ip());
        ip.set_enabled(editor);
        let ip = node_panel.add(ip);

        node_panel.add(label!("'Username:', x:0, y:4, w: 32"));
        let mut username = textfield!("caption='username', x:32, y:4, w: 32");
        username.set_text(&config.get_username());
        username.set_enabled(editor);
        let username = node_panel.add(username);

        node_panel.add(label!("'Password:', x:0, y:6, w: 32"));
        let mut password = textfield!("caption='password', x:32, y:6, w: 32");
        password.set_text(&config.get_password());
        password.set_enabled(editor);
        let password = node_panel.add(password);

        node_panel.add(label!("'Polling Schedule:', x:0, y:8, w: 32"));
        let mut polling_schedule = textfield!("caption='0 * * * * *', x:32, y:8, w: 32");
        polling_schedule.set_text(&schedule.0);
        polling_schedule.set_enabled(editor);
        let polling_schedule = node_panel.add(polling_schedule);

        let dropdown = if editor {
            let mut db = DropDownList::<NodeClass>::new(
                layout!("x:0, y:10, w:64"),
                dropdownlist::Flags::ShowDescription,
            );
            db.add(NodeClass::KasaDevice(
                String::new(),
                KasaDeviceConfig::default(),
                PollingSchedule::default(),
            ));
            db.add(NodeClass::Unknown);
            db.set_index(0);
            Some(node_panel.add(db))
        } else {
            None
        };

        let (add, update) = if editor {
            let add = button!("'Add', x:0, y: 12, w:16");
            let update = button!("'Update', x:32, y: 12, w: 16");
            (Some(node_panel.add(add)), Some(node_panel.add(update)))
        } else {
            (None, None)
        };

        NodeDeviceConfigUi {
            checkbox,
            add,
            update,
            dropdown,
            panel: panel.add(node_panel),
            name,
            ip,
            username,
            password,
            polling_schedule,
            node_class: NodeClass::KasaDevice(
                String::new(),
                Default::default(),
                Default::default(),
            ),
            height: height,
        }
    }
}

struct NodeUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    node_editor_panel: NodeDeviceConfigUi,
    node_panel: Handle<Panel>,
    add_node_button: Handle<Button>,
    update_node_button: Handle<Button>,
    remove_nodes_button: Handle<Button>,
    node_index: usize,
    node_configs: HashMap<usize, NodeDeviceConfigUi>,
}

impl NodeUi {
    const NODE_START_Y: u16 = 2;

    fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);

        let mut node_editor_panel = NodeDeviceConfigUi::new(
            NodeClass::KasaDevice(
                String::from("node_name"),
                Default::default(),
                Default::default(),
            ),
            &mut form_panel,
            true,
            String::new(),
            2,
            0,
        );
        let add_node = node_editor_panel.add.take().unwrap();
        let update_node = node_editor_panel.update.take().unwrap();

        tabs.add(index, form_panel);

        let mut node_panel = Panel::new("", layout!("x:50%, y:0, w: 50%, h: 100%"));
        let remove_nodes = node_panel.add(button!("'Remove Nodes', x:1, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);

        Self {
            enable,
            save_button: save,
            node_editor_panel,
            node_panel,
            add_node_button: add_node,
            update_node_button: update_node,
            remove_nodes_button: remove_nodes,
            node_index: 0,
            node_configs: HashMap::new(),
        }
    }

    fn add_node(&mut self, window: &mut CreatorWindow, data: NodeClass) {
        let data = if let NodeClass::Unknown = data
            && let Some(node_class) = window.control(self.node_editor_panel.dropdown.unwrap())
        {
            if let Some(node_class) = node_class.selected_item() {
                match node_class {
                    NodeClass::KasaDevice(_, _, _) => &NodeClass::KasaDevice(
                        String::from("node_name"),
                        Default::default(),
                        Default::default(),
                    ),
                    NodeClass::Unknown => &NodeClass::Unknown,
                }
            } else {
                &NodeClass::Unknown
            }
        } else {
            &data
        };

        if let NodeClass::KasaDevice(_, _, _) = data {
            let index = self.node_configs.len() as u16;
            let node_panel = window.control_mut(self.node_panel).unwrap();

            self.node_configs.insert(
                self.node_index,
                NodeDeviceConfigUi::new(
                    data.clone(),
                    node_panel,
                    false,
                    format!(" {}", self.node_index),
                    Self::NODE_START_Y,
                    index,
                ),
            );
            self.node_index = self.node_index + 1;
            window.request_update();
        }
    }

    fn remove_nodes(&mut self, window: &mut CreatorWindow) {
        let mut removal_index: Vec<usize> = vec![];

        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let remove = window.control(config.checkbox.unwrap()).unwrap();
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

        let mut node_configs: Vec<(&usize, &NodeDeviceConfigUi)> =
            self.node_configs.iter().collect();
        node_configs.sort_by(|x, y| x.0.cmp(&y.0));

        for (index, (_, config)) in node_configs.iter().enumerate() {
            let repositioned = config.height * index as u16 + Self::NODE_START_Y;
            {
                let config = window.control_mut(config.panel).unwrap();
                config.set_position(0, repositioned as i32);
            }
        }

        window.request_update();
    }

    fn generate_config(&self, window: &mut CreatorWindow) -> Option<Node> {
        if let Some(enabled) = window.control(self.enable)
            && enabled.is_checked()
        {
            let mut nodes: Vec<NodeClass> = vec![];

            for (_, (_, config)) in self.node_configs.iter().enumerate() {
                if let NodeClass::KasaDevice(_, _, _) = config.node_class {
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
                        let username = if let Some(username) = window.control(config.username) {
                            username.text().to_string()
                        } else {
                            return None;
                        };
                        let password = if let Some(password) = window.control(config.password) {
                            password.text().to_string()
                        } else {
                            return None;
                        };
                        let polling_schedule = if let Some(polling_schedule) =
                            window.control(config.polling_schedule)
                        {
                            PollingSchedule(polling_schedule.text().to_string())
                        } else {
                            return None;
                        };
                        nodes.push(NodeClass::KasaDevice(
                            name,
                            KasaDeviceConfig {
                                ip,
                                username,
                                password,
                            },
                            polling_schedule,
                        ));
                    } else {
                        return None;
                    };
                }
            }

            return Some(Node { nodes });
        }

        None
    }

    fn restore_config(&mut self, window: &mut CreatorWindow, config: Option<Node>) {
        let config = if let Some(config) = config {
            config
        } else {
            return;
        };

        for node in config.nodes {
            self.add_node(window, node);
        }

        if let Some(enable) = window.control_mut(self.enable) {
            enable.set_checked(true);
        }
    }
}
