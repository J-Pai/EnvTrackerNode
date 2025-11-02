use std::collections::HashMap;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    use thaw::ssr::SSRMountStyleProvider;
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
                <body>
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    use thaw::{ConfigProvider, Theme};

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let theme = RwSignal::new(Theme::custom_dark(&HashMap::from([
        (10, "#030303"),
        (20, "#171717"),
        (30, "#252525"),
        (40, "#313131"),
        (50, "#3D3D3D"),
        (60, "#494949"),
        (70, "#565656"),
        (80, "#636363"),
        (90, "#717171"),
        (100, "#7F7F7F"),
        (110, "#8D8D8D"),
        (120, "#9B9B9B"),
        (130, "#AAAAAA"),
        (140, "#B9B9B9"),
        (150, "#C8C8C8"),
        (160, "#D7D7D7"),
    ])));

    view! {
        <ConfigProvider theme>
            // injects a stylesheet into the document <head>
            // id=leptos means cargo-leptos will hot-reload this stylesheet
            <Stylesheet id="leptos" href="/pkg/node_web.css" />

            // sets the document title
            <Title text="Welcome to Leptos" />

            // content for this welcome page
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    use leptos_chartistry::*;
    use leptos_meta::Style;
    use thaw::*;

    #[derive(Debug)]
    pub struct MyData {
        x: f64,
        y1: f64,
        y2: f64,
    }

    impl MyData {
        fn new(x: f64, y1: f64, y2: f64) -> Self {
            Self { x, y1, y2 }
        }
    }

    pub fn load_data(count: RwSignal<i32>) -> Signal<Vec<MyData>> {
        Signal::derive(move || {
            let val = count.get() as f64 + 1.0;

            vec![
                MyData::new(0.0, 1.0 * val, 0.0 * val),
                MyData::new(1.0, 3.0 * val, 1.0 * val),
                MyData::new(2.0, 5.0 * val, 2.5 * val),
                MyData::new(3.0, 5.5 * val, 3.0 * val),
                MyData::new(4.0, 5.0 * val, 3.0 * val),
                MyData::new(5.0, 2.5 * val, 4.0 * val),
                MyData::new(6.0, 2.25 * val, 9.0 * val),
                MyData::new(7.0, 3.0 * val, 5.0 * val),
                MyData::new(8.0, 7.0 * val, 3.5 * val),
                MyData::new(9.0, 8.5 * val, 3.2 * val),
                MyData::new(10.0, 10.0 * val, 3.0 * val),
            ]
        })
    }

    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    let series = Series::new(|data: &MyData| data.x)
        .bar(Bar::new(|data: &MyData| data.y1).with_name("data1"))
        .bar(
            Bar::new(|data: &MyData| if data.x < 6.0 { data.y2 } else { -data.y2 })
                .with_name("data2"),
        )
        .with_y_range(-1000.0, 1000.0)
        .with_x_range(0.0, 10.0);

    view! {
        <Layout position=LayoutPosition::Absolute>
            <LayoutHeader attr:style="padding: 20px; font-size: 36px">"Web Node"</LayoutHeader>
            <Layout attr:style="padding: 10px;">
                <Button on:click=on_click appearance=ButtonAppearance::Primary>
                    "Click Me: "
                    {count}
                </Button>
            </Layout>
            <Layout>
                <Layout>
                    <Style>
                        "
                            div._chartistry {
                                background: var(--colorNeutralForegroundOnBrand);
                                padding: 20px;
                            }
                        
                            aside {
                                color: black;
                            }
                        "
                    </Style>
                    <Chart
                        attr:style="display: inline-block;"

                        aspect_ratio=AspectRatio::from_outer_height(800.0, 2.0)
                        series=series
                        data=load_data(count)

                        top=RotatedLabel::middle("Hello, hydration!")
                        left=TickLabels::aligned_floats()
                        bottom=TickLabels::aligned_floats()
                        right=Legend::middle()
                        inner=[
                            AxisMarker::left_edge().into_inner(),
                            AxisMarker::bottom_edge().into_inner(),
                            XGridLine::default().into_inner(),
                            YGridLine::default().into_inner(),
                            YGuideLine::over_mouse().into_inner(),
                            XGuideLine::over_data().into_inner(),
                        ]
                        tooltip=Tooltip::left_cursor().show_x_ticks(false)
                    />
                </Layout>
                <Layout attr:style="padding: 10px;">
                    <Space justify=SpaceJustify::Center>
                        <Button>"Secondary"</Button>
                        <Button appearance=ButtonAppearance::Primary>"Primary"</Button>
                        <Button appearance=ButtonAppearance::Subtle>"Subtle"</Button>
                        <Button appearance=ButtonAppearance::Transparent>"Transparent"</Button>
                    </Space>
                </Layout>
            </Layout>
        </Layout>
    }
}
