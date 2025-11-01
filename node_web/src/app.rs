use leptos::prelude::*;
use leptos_chartistry::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/node_web.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

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
    Signal::derive(|| {
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

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    let series = Series::new(|data: &MyData| data.x)
        .bar(|data: &MyData| data.y1)
        .bar(|data: &MyData| if data.x < 6.0 { data.y2 } else { -data.y2 })
        .with_y_range(-20.0, 20.0);

    println!("{:?}", load_data().read());

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <Chart
            aspect_ratio=AspectRatio::from_outer_height(300.0, 1.2)
            series=series
            data=load_data()

            top=RotatedLabel::middle("Hello, hydration!")
            left=TickLabels::aligned_floats()
            bottom=Legend::end()
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
    }
}
