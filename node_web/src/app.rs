use std::collections::HashMap;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use thaw::Theme;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    use thaw::ssr::SSRMountStyleProvider;

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    // injects a stylesheet into the document <head>
                    // id=leptos means cargo-leptos will hot-reload this stylesheet
                    <Stylesheet id="leptos" href="/pkg/node_web.css" />
                    // sets the document title
                    <Title text="Web Node" />
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
    use thaw::ConfigProvider;

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let brand_color = HashMap::from([
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
    ]);

    let theme = RwSignal::new(Theme::custom_dark(&brand_color));

    view! {
        <ConfigProvider theme>
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    // content for this welcome page
                    <Route
                        path=StaticSegment("")
                        view=move || {
                            view! {
                                <HomePage theme=theme.clone() brand_colors=brand_color.clone() />
                            }
                        }
                    />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(theme: RwSignal<Theme>, brand_colors: HashMap<i32, &'static str>) -> impl IntoView {
    use leptos_chartistry::*;
    use leptos_meta::*;
    use thaw::*;

    /// Renders the home page of your application.
    #[component]
    fn PowerConsumptionGraph() -> impl IntoView {
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

        pub fn load_data() -> Signal<Vec<MyData>> {
            Signal::derive(move || {
                vec![
                    MyData::new(0.0, 1.0, 0.0),
                    MyData::new(1.0, 3.0, 1.0),
                    MyData::new(2.0, 5.0, 2.5),
                    MyData::new(3.0, 5.5, 3.0),
                    MyData::new(4.0, 5.0, 3.0),
                    MyData::new(5.0, 2.5, 4.0),
                    MyData::new(6.0, 2.25, 9.0),
                    MyData::new(7.0, 3.0, 5.0),
                    MyData::new(8.0, 7.0, 3.5),
                    MyData::new(9.0, 8.5, 3.2),
                    MyData::new(10.0, 10.0, 3.0),
                ]
            })
        }

        let series = Series::new(|data: &MyData| data.x)
            .line(
                Line::new(|data: &MyData| data.y1)
                    .with_name("data1")
                    .with_marker(MarkerShape::Diamond)
                    .with_interpolation(Step::Horizontal),
            )
            .line(
                Line::new(|data: &MyData| if data.x < 6.0 { data.y2 } else { -data.y2 })
                    .with_name("data2")
                    .with_marker(MarkerShape::Triangle)
                    .with_interpolation(Step::Horizontal),
            )
            .with_y_range(-20.0, 20.0)
            .with_x_range(0.0, 10.0);

        view! {
            <Space justify=SpaceJustify::FlexStart gap=SpaceGap::Size(0) attr:style="height: 25vh">
                <Style>
                    "
                        ._chartistry {
                            background: var(--colorNeutralBackground1Pressed);
                            padding: 0px 10px 0px 10px;
                            width: calc(100% - 20px) !important;
                        }
                        ._chartistry > svg >
                        g._chartistry_axis_marker,
                        g._chartistry_grid_line_x,
                        g._chartistry_grid_line_y {
                            stroke: var(--colorNeutralForeground1);
                        }
                        ._chartistry > svg {
                            width: calc(100% - 5px);
                        }
                        aside {
                            color: black;
                        }
                        text {
                            fill: var(--colorNeutralForeground1);
                        }
                    "
                </Style>

                <Chart
                    aspect_ratio=AspectRatio::from_env()
                    series=series
                    data=load_data()
                    top=RotatedLabel::start("Power Consumption (Watts)")
                    inner=[
                        AxisMarker::left_edge().into_inner(),
                        AxisMarker::bottom_edge().into_inner(),
                        XGuideLine::over_data().into_inner(),
                    ]
                />
            </Space>
        }
    }

    #[component]
    fn PowerConsumptionDataCard() -> impl IntoView {
        view! {
            <Grid cols=4 attr:style="background: var(--colorNeutralBackground1Pressed); width: 100%; height: 100px">
                <GridItem column=4>
                    <Space align=SpaceAlign::Center attr:style="height: 100%;">
                        "content"
                    </Space>
                </GridItem>
            </Grid>
        }
    }

    #[component]
    fn DevicePowerControlCard(name: RwSignal<&'static str>) -> impl IntoView {
        view! {
            <Grid cols=6 attr:style="background: var(--colorNeutralBackground1Pressed); width: 400px; height: 130px">
                <GridItem>
                    <Space align=SpaceAlign::Center justify=SpaceJustify::Center attr:style="height: 100%;">
                        <div style="height: max-content; width: max-content;">
                            <Checkbox size=CheckboxSize::Large checked=true />
                        </div>
                    </Space>
                </GridItem>
                <GridItem column=4>
                    <Space align=SpaceAlign::Center attr:style="height: 100%; font-family: monospace !important">
                        <div style="height: max-content; width: max-content;">
                            <Space vertical=true justify=SpaceJustify::Start align=SpaceAlign::Start>
                                <b>{name} " / YYYY:MM::DD HH:SS"</b>
                                <Grid cols=2 attr:style="text-align: start; width: 350px;">
                                    <GridItem>
                                        "Current Usage"
                                    </GridItem>
                                    <GridItem>
                                        "106.7 W"
                                    </GridItem>
                                    <GridItem>
                                        "Today's Usage"
                                    </GridItem>
                                    <GridItem>
                                        "106.7 kWh"
                                    </GridItem>
                                    <GridItem>
                                        "Total Usage"
                                    </GridItem>
                                    <GridItem>
                                        "106.7 kWh"
                                    </GridItem>
                                </Grid>
                            </Space>
                        </div>
                    </Space>
                </GridItem>
                <GridItem attr:style="place-items: center;">
                    <Space align=SpaceAlign::Center justify=SpaceJustify::Center attr:style="height: 100%;">
                        <div style="height: max-content; width: max-content;">
                            <Button
                                icon=icondata::FaPowerOffSolid
                                appearance=ButtonAppearance::Subtle
                                shape=ButtonShape::Circular
                                attr:style="color: green"
                            ></Button>
                        </div>
                    </Space>
                </GridItem>
            </Grid>
        }
    }

    let toggle_theme_icon = RwSignal::new(icondata::BsSun);
    let on_click_toggle_theme = move |_| {
        if theme.get().name == "dark" {
            theme.set(Theme::custom_light(&brand_colors));
            toggle_theme_icon.set(icondata::BsMoonStars);
        } else {
            theme.set(Theme::custom_dark(&brand_colors));
            toggle_theme_icon.set(icondata::BsSun);
        }
    };

    view! {
        <Layout position=LayoutPosition::Absolute>
            <LayoutHeader>
                <Grid cols=2 x_gap=0 y_gap=0>
                    <GridItem>
                        <Flex justify=FlexJustify::Start>
                            <h2 style="margin-left: 20px;">"Web Node"</h2>
                        </Flex>
                    </GridItem>
                    <GridItem>
                        <Layout attr:style="margin: 10px;">
                            <Flex justify=FlexJustify::End>
                                <Button
                                    icon=toggle_theme_icon
                                    on:click=on_click_toggle_theme
                                    appearance=ButtonAppearance::Primary
                                ></Button>
                            </Flex>
                        </Layout>
                    </GridItem>
                </Grid>
            </LayoutHeader>
            <PowerConsumptionGraph />
            <Layout attr:style="margin: 10px;">
                <Grid cols=1 x_gap=8 y_gap=8>
                    <GridItem>
                        <PowerConsumptionDataCard />
                    </GridItem>
                    <GridItem>
                        <Flex attr:style="flex-wrap: wrap;">
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device1")/></GridItem>
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device2")/></GridItem>
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device3")/></GridItem>
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device4")/></GridItem>
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device5")/></GridItem>
                            <GridItem><DevicePowerControlCard name=RwSignal::new("Device6")/></GridItem>
                        </Flex>
                    </GridItem>
                </Grid>
            </Layout>
        </Layout>
    }
}
