#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use url::Url;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let base = document
            .get_element_by_id("base")
            .expect("Failed to find api")
            .dyn_into::<web_sys::HtmlBaseElement>()
            .expect("api is not a HtmlLinkElement")
            .href();
        let _base = Url::parse(&base).unwrap();

        let api_uri = document
            .get_element_by_id("api")
            .expect("Failed to find api")
            .dyn_into::<web_sys::HtmlLinkElement>()
            .expect("api is not a HtmlLinkElement")
            .href();
        let api_uri = Url::parse(&api_uri).unwrap();

        let kasa_api_uri = document
            .get_element_by_id("kasa_api")
            .expect("Failed to find kasa_api")
            .dyn_into::<web_sys::HtmlLinkElement>()
            .expect("kasas_api is not a HtmlLinkElement")
            .href();
        let kasa_api_uri = Url::parse(&kasa_api_uri).unwrap();
        let kasa_api_uri_path = kasa_api_uri.path().to_string();

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| Ok(Box::new(site::EnvApp::new(cc, api_uri, kasa_api_uri_path)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
