//! Handles the configuration of the Frontend server.

use appcui::prelude::*;
use url::Url;

use crate::config::FrontendServerConfig;
use crate::config::ServerConfig;
use crate::config::creator::CreatorWindow;

pub(super) struct FrontendServerUi {
    enable: Handle<CheckBox>,
    save_button: Handle<Button>,
    uri_field: Handle<TextField>,
    kasa_api_field: Handle<TextField>,
    base_field: Handle<TextField>,
}

impl FrontendServerUi {
    pub(super) fn new(tabs: &mut Tab, index: u32) -> Self {
        let mut form_panel = Panel::new("", layout!("x:0, y:0, w: 50%, h: 100%"));

        let enable = checkbox!("'Enable Service', x:1, y:0, w:32");
        let enable = form_panel.add(enable);
        let save = button!("'Save', x:32, y:0, w:32");
        let save = form_panel.add(save);

        form_panel.add(label!("'API Server Base URI:', x:0, y:2, w: 32"));
        let uri = textfield!("caption='http://0.0.0.0:3000', x:32, y:2, w: 32");
        let uri = form_panel.add(uri);
        form_panel.add(label!("'Kasa API Endpoint:', x:0, y:4, w: 32"));
        let kasa_api = textfield!("caption='/kasa', x:32, y:4, w: 32");
        let kasa_api = form_panel.add(kasa_api);
        form_panel.add(label!("'Frontend Base Path:', x:0, y:6, w: 32"));
        let base = textfield!("caption='', x:32, y:6, w: 32");
        let base = form_panel.add(base);

        tabs.add(index, form_panel);

        Self {
            enable,
            save_button: save,
            uri_field: uri,
            kasa_api_field: kasa_api,
            base_field: base,
        }
    }

    fn generate_config(&self, window: &mut CreatorWindow) -> Option<FrontendServerConfig> {
        if let Some(enabled) = window.control(self.enable)
            && enabled.is_checked()
        {
            let api_server_uri = if let Some(ip) = window.control(self.uri_field) {
                Url::parse(ip.text().trim()).unwrap()
            } else {
                return None;
            };

            let kasa_api = if let Some(kasa_api) = window.control(self.kasa_api_field) {
                kasa_api.text().to_string()
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
                api_server_uri,
                kasa_api,
                base,
            });
        }

        None
    }

    pub(super) fn restore_config(
        &mut self,
        window: &mut CreatorWindow,
        config: Option<FrontendServerConfig>,
    ) {
        let config = if let Some(config) = config {
            config
        } else {
            return;
        };

        if let Some(uri) = window.control_mut(self.uri_field) {
            uri.set_text(config.get_api_server_uri().as_str());
        }

        if let Some(kasa_api) = window.control_mut(self.kasa_api_field) {
            kasa_api.set_text(&config.get_kasa_api());
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

    pub(super) fn button_handler(
        &mut self,
        window: &mut CreatorWindow,
        server_config: &mut ServerConfig,
        handle: Handle<Button>,
    ) -> Result<EventProcessStatus, Handle<Button>> {
        if handle == self.save_button {
            server_config.frontend_server = self.generate_config(window);
        }
        Err(handle)
    }
}
