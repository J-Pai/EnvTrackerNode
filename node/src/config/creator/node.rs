//! Handles the configuration of a Node server.

use std::collections::HashMap;

use appcui::prelude::*;
use url::Url;

use crate::config::KasaDeviceConfig;
use crate::config::Node;
use crate::config::NodeClass;
use crate::config::PollingConfig;
use crate::config::ServerConfig;
use crate::config::creator::CreatorWindow;

struct NodeDeviceConfigUi {
    checkbox: Option<Handle<CheckBox>>,
    add: Option<Handle<Button>>,
    update: Option<Handle<Button>>,
    dropdown: Option<Handle<DropDownList<NodeClass>>>,
    panel: Handle<Panel>,
    name: Handle<TextField>,
    uri: Handle<TextField>,
    username: Handle<TextField>,
    password: Handle<TextField>,
    polling_schedule: Handle<TextField>,
    polling_endpoint: Handle<TextField>,
    node_class: NodeClass,
    height: u16,
}

impl NodeDeviceConfigUi {
    const NODE_HEIGHT: u16 = 18;

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

        let smaller = if editor { 0 } else { 4 };
        let height = Self::NODE_HEIGHT - smaller;

        let info = if id.is_empty() {
            "(Kasa Device)"
        } else {
            &format!("{} (Kasa Device)", id)
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
        node_panel.add(label!("'URI:', x:0, y:2, w: 32"));
        let mut uri = textfield!("caption='http://0.0.0.0:3000', x:32, y:2, w: 32");
        uri.set_text(config.get_uri().as_str());
        uri.set_enabled(editor);
        let uri = node_panel.add(uri);

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
        polling_schedule.set_text(&schedule.schedule);
        polling_schedule.set_enabled(editor);
        let polling_schedule = node_panel.add(polling_schedule);

        node_panel.add(label!("'Polling Endpoint:', x:0, y:10, w: 32"));
        let mut polling_endpoint = textfield!("caption='', x:32, y:10, w: 32");
        let api = if let Some(api) = schedule.get_api().clone() {
            api.clone()
        } else {
            String::new()
        };
        polling_endpoint.set_text(&api);
        polling_endpoint.set_enabled(editor);
        let polling_endpoint = node_panel.add(polling_endpoint);

        let dropdown = if editor {
            let mut db = DropDownList::<NodeClass>::new(
                layout!("x:0, y:12, w:64"),
                dropdownlist::Flags::ShowDescription,
            );
            db.add(NodeClass::KasaDevice(
                String::new(),
                Box::default(),
                PollingConfig::default(),
            ));
            db.add(NodeClass::Unknown);
            db.set_index(0);
            Some(node_panel.add(db))
        } else {
            None
        };

        let (add, update) = if editor {
            let add = button!("'Add', x:0, y: 14, w:16");
            let update = button!("'Update', x:32, y: 14, w: 16");
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
            uri,
            username,
            password,
            polling_schedule,
            polling_endpoint,
            node_class: NodeClass::KasaDevice(
                String::new(),
                Default::default(),
                Default::default(),
            ),
            height,
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
    load_node_button: Handle<Button>,
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
        let load_node = node_panel.add(button!("'Load Node', x:32, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);

        Self {
            enable,
            save_button: save,
            node_editor_panel,
            node_panel,
            add_node_button: add_node,
            update_node_button: update_node,
            remove_nodes_button: remove_nodes,
            load_node_button: load_node,
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
            self.node_index += 1;
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
        node_configs.sort_by(|x, y| x.0.cmp(y.0));

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
                if update.is_checked()
                    && let NodeClass::KasaDevice(id, device_config, schedule) = &data
                {
                    let name = window.control_mut(config.name).unwrap();
                    name.set_text(id);
                    let uri = window.control_mut(config.uri).unwrap();
                    uri.set_text(device_config.get_uri().as_str());
                    let username = window.control_mut(config.username).unwrap();
                    username.set_text(&device_config.get_username());
                    let password = window.control_mut(config.password).unwrap();
                    password.set_text(&device_config.get_password());
                    let polling_schedule = window.control_mut(config.polling_schedule).unwrap();
                    polling_schedule.set_text(&schedule.schedule);
                    let polling_endpoint = window.control_mut(config.polling_endpoint).unwrap();
                    let api = if let Some(api) = schedule.get_api().clone() {
                        api.clone()
                    } else {
                        String::new()
                    };
                    polling_endpoint.set_text(&api);
                }
            }
        }
        window.request_update();
    }

    fn load_node(&mut self, window: &mut CreatorWindow) {
        for location in self.node_configs.keys() {
            if let Some(config) = self.node_configs.get(location) {
                let update = window.control(config.checkbox.unwrap()).unwrap();
                if update.is_checked()
                    && let NodeClass::KasaDevice(_, _, _) = &config.node_class
                {
                    let name = window
                        .control(config.name)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_name = window.control_mut(self.node_editor_panel.name).unwrap();
                    editor_name.set_text(&name);

                    let uri = window
                        .control(config.uri)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_uri = window.control_mut(self.node_editor_panel.uri).unwrap();
                    editor_uri.set_text(&uri);

                    let username = window
                        .control(config.username)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_username =
                        window.control_mut(self.node_editor_panel.username).unwrap();
                    editor_username.set_text(&username);

                    let password = window
                        .control_mut(config.password)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_password =
                        window.control_mut(self.node_editor_panel.password).unwrap();
                    editor_password.set_text(&password);

                    let polling_schedule = window
                        .control(config.polling_schedule)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_polling_schedule = window
                        .control_mut(self.node_editor_panel.polling_schedule)
                        .unwrap();
                    editor_polling_schedule.set_text(&polling_schedule);

                    let polling_endpoint = window
                        .control(config.polling_endpoint)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_polling_endpoint = window
                        .control_mut(self.node_editor_panel.polling_endpoint)
                        .unwrap();
                    editor_polling_endpoint.set_text(&polling_endpoint);
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

            for (_, config) in self.node_configs.iter() {
                if let NodeClass::KasaDevice(_, _, _) = config.node_class {
                    if window.control(config.panel).is_some() {
                        let name = if let Some(name) = window.control(config.name) {
                            name.text().to_string()
                        } else {
                            return None;
                        };
                        let uri = if let Some(uri) = window.control(config.uri) {
                            Url::parse(uri.text().trim()).unwrap()
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
                            && let Some(polling_endpoint) = window.control(config.polling_endpoint)
                        {
                            let polling_endpoint = polling_endpoint.text().trim();
                            let endpoint = if polling_endpoint.is_empty() {
                                None
                            } else {
                                Some(polling_endpoint.to_string())
                            };
                            PollingConfig {
                                schedule: polling_schedule.text().to_string(),
                                api: endpoint,
                            }
                        } else {
                            return None;
                        };
                        nodes.push(NodeClass::KasaDevice(
                            name,
                            Box::new(KasaDeviceConfig {
                                uri,
                                username,
                                password,
                                batch_size: None,
                            }),
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
                    NodeClass::KasaDevice(_, _, _) => {
                        let polling_endpoint = window
                            .control(self.node_editor_panel.polling_endpoint)
                            .unwrap();

                        let polling_endpoint = polling_endpoint.text().trim();
                        let endpoint = if polling_endpoint.is_empty() {
                            None
                        } else {
                            Some(polling_endpoint.to_string())
                        };
                        NodeClass::KasaDevice(
                            window
                                .control(self.node_editor_panel.name)
                                .unwrap()
                                .text()
                                .to_string(),
                            Box::new(KasaDeviceConfig {
                                uri: Url::parse(
                                    window.control(self.node_editor_panel.uri).unwrap().text(),
                                )
                                .unwrap(),
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
                                batch_size: None,
                            }),
                            PollingConfig {
                                schedule: window
                                    .control(self.node_editor_panel.polling_schedule)
                                    .unwrap()
                                    .text()
                                    .to_string(),
                                api: endpoint,
                            },
                        )
                    }
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
                    NodeClass::KasaDevice(_, _, _) => {
                        let polling_endpoint = window
                            .control(self.node_editor_panel.polling_endpoint)
                            .unwrap();

                        let polling_endpoint = polling_endpoint.text().trim();
                        let endpoint = if polling_endpoint.is_empty() {
                            None
                        } else {
                            Some(polling_endpoint.to_string())
                        };
                        NodeClass::KasaDevice(
                            window
                                .control(self.node_editor_panel.name)
                                .unwrap()
                                .text()
                                .to_string(),
                            Box::new(KasaDeviceConfig {
                                uri: Url::parse(
                                    window
                                        .control(self.node_editor_panel.uri)
                                        .unwrap()
                                        .text()
                                        .trim(),
                                )
                                .unwrap(),
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
                                batch_size: None,
                            }),
                            PollingConfig {
                                schedule: window
                                    .control(self.node_editor_panel.polling_schedule)
                                    .unwrap()
                                    .text()
                                    .to_string(),
                                api: endpoint,
                            },
                        )
                    }
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

        if handle == self.load_node_button {
            self.load_node(window);
            return Ok(EventProcessStatus::Processed);
        }

        Err(handle)
    }
}
