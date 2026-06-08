//! Handles the configuration of a Node server.

use std::collections::HashMap;

use appcui::prelude::*;

use crate::config::Ip;
use crate::config::KasaDeviceConfig;
use crate::config::Node;
use crate::config::NodeClass;
use crate::config::PollingSchedule;
use crate::config::ServerConfig;
use crate::config::creator::CreatorWindow;

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
        name.set_text(if id.is_empty() { "node_name" } else { &id });
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

pub(super) struct NodeUi {
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

    pub(super) fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);

        let mut node_editor_panel = NodeDeviceConfigUi::new(
            NodeClass::KasaDevice(String::new(), Default::default(), Default::default()),
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

    fn update_nodes(&mut self, window: &mut CreatorWindow, data: NodeClass) {
        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let update = window.control(config.checkbox.unwrap()).unwrap();
                if update.is_checked() {
                    if let NodeClass::KasaDevice(id, device_config, schedule) = &data {
                        let name = window.control_mut(config.name).unwrap();
                        name.set_text(&id);
                        let ip = window.control_mut(config.ip).unwrap();
                        ip.set_text(&device_config.get_ip());
                        let username = window.control_mut(config.username).unwrap();
                        username.set_text(&device_config.get_username());
                        let password = window.control_mut(config.password).unwrap();
                        password.set_text(&device_config.get_password());
                        let polling_schedule = window.control_mut(config.polling_schedule).unwrap();
                        polling_schedule.set_text(&schedule.0);
                    }
                }
            }
        }
        window.request_update();
    }

    pub(super) fn generate_config(&self, window: &mut CreatorWindow) -> Option<Node> {
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

    pub(super) fn restore_config(&mut self, window: &mut CreatorWindow, config: Option<Node>) {
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

    pub(super) fn button_handler(
        &mut self,
        window: &mut CreatorWindow,
        server_config: &mut ServerConfig,
        handle: Handle<Button>,
    ) -> Result<EventProcessStatus, Handle<Button>> {
        if handle == self.save_button {
            server_config.node = self.generate_config(window);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.add_node_button {
            let data = if let Some(node_class) =
                window.control(self.node_editor_panel.dropdown.unwrap())
                && let Some(node_class) = node_class.selected_item()
            {
                match node_class {
                    NodeClass::KasaDevice(_, _, _) => NodeClass::KasaDevice(
                        window
                            .control(self.node_editor_panel.name)
                            .unwrap()
                            .text()
                            .to_string(),
                        KasaDeviceConfig {
                            ip: Ip(window
                                .control(self.node_editor_panel.ip)
                                .unwrap()
                                .text()
                                .to_string()),
                            username: window
                                .control(self.node_editor_panel.username)
                                .unwrap()
                                .text()
                                .to_string(),
                            password: window
                                .control(self.node_editor_panel.password)
                                .unwrap()
                                .text()
                                .to_string(),
                        },
                        PollingSchedule(
                            window
                                .control(self.node_editor_panel.polling_schedule)
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

            self.add_node(window, data);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.update_node_button {
            let data = if let Some(node_class) =
                window.control(self.node_editor_panel.dropdown.unwrap())
                && let Some(node_class) = node_class.selected_item()
            {
                match node_class {
                    NodeClass::KasaDevice(_, _, _) => NodeClass::KasaDevice(
                        window
                            .control(self.node_editor_panel.name)
                            .unwrap()
                            .text()
                            .to_string(),
                        KasaDeviceConfig {
                            ip: Ip(window
                                .control(self.node_editor_panel.ip)
                                .unwrap()
                                .text()
                                .to_string()),
                            username: window
                                .control(self.node_editor_panel.username)
                                .unwrap()
                                .text()
                                .to_string(),
                            password: window
                                .control(self.node_editor_panel.password)
                                .unwrap()
                                .text()
                                .to_string(),
                        },
                        PollingSchedule(
                            window
                                .control(self.node_editor_panel.polling_schedule)
                                .unwrap()
                                .text()
                                .to_string(),
                        ),
                    ),
                    NodeClass::Unknown => NodeClass::default(),
                }
            } else {
                unreachable!()
            };

            self.update_nodes(window, data);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.remove_nodes_button {
            self.remove_nodes(window);
            return Ok(EventProcessStatus::Processed);
        }

        Err(handle)
    }
}
