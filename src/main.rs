use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::logger::tracing::debug;
use mtrack_remote::route::Route;

const FAVICON: Asset = asset!("./assets/favicon_512x512.ico");
const TAILWIND_CSS: Asset = asset!("./assets/tailwind.css");

#[cfg(feature = "server")]
fn main() {
    use std::sync::Arc;

    use dioxus::logger::tracing::warn;
    use mtrack_remote::backend::config::Config;
    use tokio::sync::RwLock;

    debug!("Starting server");
    let builder = dioxus::LaunchBuilder::server();
    let config = match Config::read_config() {
        Ok(config) => config,
        Err(err) => {
            warn!("Could not read config, creating default config. {err}");
            Config::new()
        }
    };

    builder
        .with_context(server_only!(Arc::new(RwLock::new(Some(
            mtrack_remote::backend::server::OscStateMachine::new()
        )))))
        .with_context(Arc::new(RwLock::new(config)))
        .launch(App);
}

#[cfg(feature = "web")]
fn main() {
    use dioxus::logger::tracing::debug;

    debug!("Starting web client");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}

#[cfg(test)]
mod main_test {
    #[cfg(feature = "server")]
    #[tokio::test]
    async fn test_app_init() {
        use std::sync::Arc;

        use tokio::sync::RwLock;

        use crate::App;

        let builder = dioxus::LaunchBuilder::server();

        builder
            .with_context(Arc::new(RwLock::new(Some(
                mtrack_remote::backend::server::OscStateMachine::new(),
            ))))
            .with_context(Arc::new(RwLock::new(
                mtrack_remote::backend::config::Config::new(),
            )))
            .launch(App);
    }
}
