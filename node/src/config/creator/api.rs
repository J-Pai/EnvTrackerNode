//! Handles the configuration of the API server.

use std::collections::HashMap;

use appcui::prelude::*;

use crate::config::ApiServerConfig;
use crate::config::Ip;
use crate::config::KasaDeviceConfig;
use crate::config::NodeClass;
use crate::config::PollingConfig;
use crate::config::ServerConfig;
use crate::config::creator::CreatorWindow;

pub(super) struct NodeConfigUi {
    checkbox: Option<Handle<CheckBox>>,
    add: Option<Handle<Button>>,
    update: Option<Handle<Button>>,
    dropdown: Option<Handle<DropDownList<NodeClass>>>,
    panel: Handle<Panel>,
    name: Handle<TextField>,
    ip: Handle<TextField>,
    polling_schedule: Handle<TextField>,
    polling_endpoint: Handle<TextField>,
    height: u16,
}

impl NodeConfigUi {
    const NODE_HEIGHT: u16 = 14;

    pub(super) fn new(
        data: NodeClass,
        panel: &mut Panel,
        editor: bool,
        id: String,
        start_y: u16,
        y_multiplier: u16,
    ) -> Self {
        let smaller = if editor { 0 } else { 4 };
        let height = Self::NODE_HEIGHT - smaller;

        let info = "(Kasa Device)";

        let mut node_editor_panel = Panel::new(
            &format!("Node Config{} - {}", id, info),
            LayoutBuilder::new()
                .x(0)
                .y((Self::NODE_HEIGHT - smaller) * y_multiplier + start_y)
                .width(1.0)
                .height(height)
                .build(),
        );

        let (id, device_config, schedule) = match data {
            NodeClass::KasaDevice(id, device_config, schedule) => (id, device_config, schedule),
            NodeClass::Unknown => (
                // Unknown is currently an unlikely configuration, as such, defaulting to Kasa
                // Device. Once other node types arrive this will be adjusted.
                String::from("node_name"),
                KasaDeviceConfig::default(),
                PollingConfig::default(),
            ),
        };

        let checkbox = if editor {
            let name_label = label!("'Node Name:', x:0, y:0, w:32");
            node_editor_panel.add(name_label);
            None
        } else {
            let checkbox = checkbox!("'Node Name:', x:0, y:0, w:32");
            Some(node_editor_panel.add(checkbox))
        };
        let mut name = textfield!("caption='node_name', x:32, y:0, w: 32");
        name.set_text(&id);
        name.set_enabled(editor);
        let name = node_editor_panel.add(name);
        node_editor_panel.add(label!("'IP:', x:0, y:2, w: 32"));
        let mut ip = textfield!("caption='0.0.0.0:3000', x:32, y:2, w: 32");
        ip.set_text(&device_config.get_ip());
        ip.set_enabled(editor);
        let ip = node_editor_panel.add(ip);

        node_editor_panel.add(label!("'Polling Schedule:', x:0, y:4, w: 32"));
        let mut polling_schedule = textfield!("caption='0 * * * * *', x:32, y:4, w: 32");
        polling_schedule.set_text(&schedule.schedule);
        polling_schedule.set_enabled(editor);
        let polling_schedule = node_editor_panel.add(polling_schedule);

        node_editor_panel.add(label!("'Polling Endpoint:', x:0, y:6, w: 32"));
        let mut polling_endpoint = textfield!("caption='', x:32, y:6, w: 32");
        let api = if let Some(api) = schedule.get_api().clone() {
            api.clone()
        } else {
            String::new()
        };
        polling_endpoint.set_text(&api);
        polling_endpoint.set_enabled(editor);
        let polling_endpoint = node_editor_panel.add(polling_endpoint);

        let dropdown = if editor {
            let mut db = DropDownList::<NodeClass>::new(
                layout!("x:0, y:8, w:64"),
                dropdownlist::Flags::ShowDescription,
            );
            db.add(NodeClass::KasaDevice(
                String::new(),
                KasaDeviceConfig::default(),
                PollingConfig::default(),
            ));
            db.add(NodeClass::Unknown);
            db.set_index(0);
            Some(node_editor_panel.add(db))
        } else {
            None
        };

        let (add, update) = if editor {
            let add = button!("'Add', x:0, y: 10, w:16");
            let update = button!("'Update', x:32, y: 10, w: 16");
            (
                Some(node_editor_panel.add(add)),
                Some(node_editor_panel.add(update)),
            )
        } else {
            (None, None)
        };

        let node_editor_panel = panel.add(node_editor_panel);

        Self {
            checkbox,
            add,
            update,
            dropdown,
            panel: node_editor_panel,
            name,
            ip,
            polling_schedule,
            polling_endpoint,
            height,
        }
    }
}

pub(super) struct ApiServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    db_field: Handle<TextField>,
    node_editor_panel: NodeConfigUi,
    node_panel: Handle<Panel>,
    update_node_button: Handle<Button>,
    add_node_button: Handle<Button>,
    remove_nodes_button: Handle<Button>,
    load_node_button: Handle<Button>,
    node_index: usize,
    node_configs: HashMap<usize, NodeConfigUi>,
}

impl ApiServerUi {
    const NODE_START_Y: u16 = 2;

    pub(crate) fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);
        let label = label!("'Database Path:', x:1, y:2, w: 14");
        form_panel.add(label);
        let db = form_panel.add(textfield!("caption='sqlite.db', x:32, y:2, w:32"));

        let mut node_editor_panel = NodeConfigUi::new(
            NodeClass::default(),
            &mut form_panel,
            true,
            String::new(),
            4,
            0,
        );

        tabs.add(index, form_panel);

        let mut node_panel = Panel::new("", layout!("x:50%, y:0, w: 50%, h: 100%"));
        let remove_nodes = node_panel.add(button!("'Remove Nodes', x:1, y:0, w:16"));
        let load_node = node_panel.add(button!("'Load Node', x:32, y:0, w:16"));
        let node_panel = tabs.add(index, node_panel);
        let add_node = node_editor_panel.add.take().unwrap();
        let update_node = node_editor_panel.update.take().unwrap();

        Self {
            enable,
            save_button: save,
            db_field: db,
            node_editor_panel,
            node_panel,
            update_node_button: update_node,
            add_node_button: add_node,
            remove_nodes_button: remove_nodes,
            load_node_button: load_node,
            node_index: 0,
            node_configs: HashMap::new(),
        }
    }

    fn add_node(&mut self, window: &mut CreatorWindow, data: NodeClass) {
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
        self.node_index += 1;
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
                    let ip = window.control_mut(config.ip).unwrap();
                    ip.set_text(&device_config.get_ip());
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
                if update.is_checked() {
                    let name = window
                        .control(config.name)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string();
                    let editor_name = window.control_mut(self.node_editor_panel.name).unwrap();
                    editor_name.set_text(&name);

                    let ip = window.control(config.ip).unwrap().text().trim().to_string();
                    let editor_ip = window.control_mut(self.node_editor_panel.ip).unwrap();
                    editor_ip.set_text(&ip);

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

    fn generate_config(&self, window: &mut CreatorWindow) -> Option<ApiServerConfig> {
        if let Some(enabled) = window.control(self.enable)
            && enabled.is_checked()
        {
            let db = if let Some(db) = window.control(self.db_field) {
                db.text().to_string()
            } else {
                return None;
            };

            let mut nodes: Vec<NodeClass> = vec![];

            for (_, config) in self.node_configs.iter() {
                if window.control(config.panel).is_some() {
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

                    // Assuming everything is just a Kasa Device for now.
                    nodes.push(NodeClass::KasaDevice(
                        name,
                        KasaDeviceConfig {
                            ip,
                            username: String::new(),
                            password: String::new(),
                        },
                        polling_schedule,
                    ));
                } else {
                    return None;
                };
            }

            return Some(ApiServerConfig { db, nodes });
        }

        None
    }

    pub(super) fn restore_config(
        &mut self,
        window: &mut CreatorWindow,
        config: Option<ApiServerConfig>,
    ) {
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

    pub(super) fn button_handler(
        &mut self,
        window: &mut CreatorWindow,
        server_config: &mut ServerConfig,
        handle: Handle<Button>,
    ) -> Result<EventProcessStatus, Handle<Button>> {
        if handle == self.save_button {
            server_config.api_server = self.generate_config(window);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.add_node_button {
            let polling_endpoint = window
                .control(self.node_editor_panel.polling_endpoint)
                .unwrap();

            let polling_endpoint = polling_endpoint.text().trim();
            let endpoint = if polling_endpoint.is_empty() {
                None
            } else {
                Some(polling_endpoint.to_string())
            };

            let class = window
                .control(self.node_editor_panel.dropdown.unwrap())
                .unwrap();
            let class = class.selected_item();
            let data = match class.unwrap() {
                NodeClass::KasaDevice(_, _, _) => NodeClass::KasaDevice(
                    window
                        .control(self.node_editor_panel.name)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string(),
                    KasaDeviceConfig {
                        ip: Ip(window
                            .control(self.node_editor_panel.ip)
                            .unwrap()
                            .text()
                            .trim()
                            .to_string()),
                        username: String::new(),
                        password: String::new(),
                    },
                    PollingConfig {
                        schedule: window
                            .control(self.node_editor_panel.polling_schedule)
                            .unwrap()
                            .text()
                            .to_string(),
                        api: endpoint,
                    },
                ),
                NodeClass::Unknown => NodeClass::Unknown,
            };

            self.add_node(window, data);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.remove_nodes_button {
            self.remove_nodes(window);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.update_node_button {
            let polling_endpoint = window
                .control(self.node_editor_panel.polling_endpoint)
                .unwrap();

            let polling_endpoint = polling_endpoint.text().trim();
            let endpoint = if polling_endpoint.is_empty() {
                None
            } else {
                Some(polling_endpoint.to_string())
            };

            let class = window
                .control(self.node_editor_panel.dropdown.unwrap())
                .unwrap();
            let class = class.selected_item();
            let data = match class.unwrap() {
                NodeClass::KasaDevice(_, _, _) => NodeClass::KasaDevice(
                    window
                        .control(self.node_editor_panel.name)
                        .unwrap()
                        .text()
                        .trim()
                        .to_string(),
                    KasaDeviceConfig {
                        ip: Ip(window
                            .control(self.node_editor_panel.ip)
                            .unwrap()
                            .text()
                            .trim()
                            .to_string()),
                        username: String::new(),
                        password: String::new(),
                    },
                    PollingConfig {
                        schedule: window
                            .control(self.node_editor_panel.polling_schedule)
                            .unwrap()
                            .text()
                            .to_string(),
                        api: endpoint,
                    },
                ),
                NodeClass::Unknown => NodeClass::Unknown,
            };

            self.update_nodes(window, data);
            return Ok(EventProcessStatus::Processed);
        }

        if handle == self.load_node_button {
            self.load_node(window);
            return Ok(EventProcessStatus::Processed);
        }

        Err(handle)
    }
}
