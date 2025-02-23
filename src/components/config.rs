use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use dioxus::{
    logger::tracing::{debug, error, warn},
    prelude::*,
};

#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use crate::backend::server::OscStateMachine;

#[cfg(feature = "server")]
use tokio::sync::RwLock;

use crate::backend::config::{Config, DEFAULT_LISTEN_PORT, DEFAULT_MTRACK_PORT};
use crate::components::{NumberInput, TextInput};

enum OptionResource<T: 'static> {
    SomeResource(Resource<T>),
    NoResource(T),
}

struct ConfigResetArguments {
    mtrack_host_edit: Signal<String>,
    mtrack_port_edit: Signal<u16>,
    listen_port_edit: Signal<u16>,
    used_config: OptionResource<Option<Config>>,
}

fn try_read_used_config(used_config: OptionResource<Option<Config>>) -> Option<Config> {
    match used_config {
        OptionResource::SomeResource(used_config_resource) => {
            let used_config_resource_peek = used_config_resource.try_peek();
            match used_config_resource_peek.as_deref() {
                Ok(Some(Some(config))) => Some(*config),
                _ => None,
            }
        }
        OptionResource::NoResource(used_config) => used_config,
    }
}

fn reset_config(mut config_editors: ConfigResetArguments) {
    debug!("Resetting config");
    let used_config = try_read_used_config(config_editors.used_config);
    match used_config {
        Some(config) => {
            let config_ip = config.mtrack_addr.ip().to_string();
            config_editors.mtrack_host_edit.set(config_ip);
            let config_mtrack_port = config.mtrack_addr.port();
            config_editors.mtrack_port_edit.set(config_mtrack_port);
            let config_listen_port = config.listen_port;
            config_editors.listen_port_edit.set(config_listen_port);
        }
        None => warn!("Current server config is not set! Cannot reset config signals."),
    };
}

/// Config component that allows changing application settings.
#[component]
pub fn ConfigComponent() -> Element {
    let mtrack_host_edit = use_signal(|| "0.0.0.0".to_string());
    let mtrack_port_edit = use_signal(|| 0);
    let listen_port_edit = use_signal(|| 0);
    let mut used_config = use_resource(move || async move {
        let server_config = match get_config().await {
            Ok(server_config) => Some(server_config),
            Err(e) => {
                error!("Could not get server config! {e}");
                None
            }
        };

        use_effect(move || {
            let config_editors = ConfigResetArguments {
                mtrack_host_edit,
                mtrack_port_edit,
                listen_port_edit,
                used_config: OptionResource::NoResource(server_config),
            };
            reset_config(config_editors);
        });
        server_config
    });

    let mtrack_host_element = rsx!(
        TextInput { value: mtrack_host_edit, default_value: "127.0.0.1" }
    );

    let mtrack_port_element = rsx!(
        NumberInput { value: mtrack_port_edit, default_value: DEFAULT_MTRACK_PORT }
    );

    let listen_port_element = rsx!(
        NumberInput { value: listen_port_edit, default_value: DEFAULT_LISTEN_PORT }
    );

    let edit_config_memo = use_memo(move || {
        debug!("Edit config memo!");
        let mtrack_host = mtrack_host_edit.read().to_string();
        let mtrack_host_addr = match IpAddr::from_str(&mtrack_host) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Could not parse IP addr {mtrack_host}! {e}");
                return None;
            }
        };

        let mtrack_port = *mtrack_port_edit.read();

        let mtrack_addr = SocketAddr::new(mtrack_host_addr, mtrack_port);
        let listen_port = match listen_port_edit.try_read() {
            Ok(port) => *port,
            Err(err) => {
                error!("Could not read listen_port_edit! {err}");
                0
            }
        };

        Some(Config {
            mtrack_addr,
            listen_port,
        })
    });

    let is_unchanged = use_memo(move || {
        let used_config_read = match used_config.read().as_ref() {
            Some(Some(config)) => *config,
            Some(None) | None => return true,
        };

        let edit_config_memo_read = match edit_config_memo() {
            Some(config) => config,
            None => return true,
        };
        used_config_read == edit_config_memo_read
    });

    rsx! {
        div { id: "config", class: "flex flex-col w-full",
            header { "Configuration" }
            div { class: "flex flex-col w-full",
                div { class: "flex flex-row w-full",
                    label { class: "basis-1/3", "mtrack host" }
                    div { class: "basis-1/3" }
                    div { class: "basis-1/3", {mtrack_host_element} }
                }
                div { class: "flex flex-row  w-full",
                    label { class: "basis-1/3", "mtrack port" }
                    div { class: "basis-1/3" }
                    div { class: "basis-1/3", {mtrack_port_element} }
                }
                div { class: "flex flex-row w-full",
                    label { class: "basis-1/3", "listen port" }
                    div { class: "basis-1/3" }
                    div { class: "basis-1/3", {listen_port_element} }
                }
                div { class: "flex flex-row w-full",
                    button {
                        class: "basis-1/2",
                        disabled: is_unchanged,
                        onclick: move |_event| async move {
                            let config_editors = ConfigResetArguments {
                                mtrack_host_edit,
                                mtrack_port_edit,
                                listen_port_edit,
                                used_config: OptionResource::SomeResource(used_config),
                            };
                            reset_config(config_editors);
                        },
                        "Reset"
                    }
                    button {
                        class: "basis-1/2",
                        disabled: is_unchanged,
                        onclick: move |_event| async move {
                            debug!("Save changes");
                            let edited_config = match *edit_config_memo.peek() {
                                Some(edited_config) => edited_config,
                                None => {
                                    error!("Could not read config to set..");
                                    return;
                                }
                            };
                            let _new_config = match set_config(edited_config).await {
                                Ok(new_config) => new_config,
                                Err(e) => {
                                    error!("Could not set config! {e}");
                                    return;
                                }
                            };
                            used_config.restart();
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

/// Get server configuration
#[server]
async fn get_config() -> Result<Config, ServerFnError> {
    debug!("Getting config...");
    let FromContext(config): FromContext<Arc<RwLock<Config>>> = extract().await?;
    let config = config.read().await;

    Ok(*config)
}

#[cfg(feature = "server")]
async fn set_config_value(config: Arc<RwLock<Config>>, new_config: Config) {
    let mut writeable_config = config.write().await;
    *writeable_config = new_config;
    match new_config.write_config() {
        Ok(_ok) => debug!("Saved config."),
        Err(err) => warn!("Failed saving config! {err}"),
    };
}

#[server(SetNewConfig)]
async fn set_config(new_config: Config) -> Result<Config, ServerFnError> {
    let FromContext(config): FromContext<Arc<RwLock<Config>>> = extract().await?;
    set_config_value(config.clone(), new_config).await;
    let FromContext(state_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let mut writable_state_option = state_option.write().await;
    *writable_state_option = match writable_state_option.take() {
        Some(state_machine) => {
            let disconnected_state_machine = state_machine.disconnect().await;
            Some(
                disconnected_state_machine
                    .ensure_connection(*config.read().await)
                    .await,
            )
        }
        None => {
            warn!("Could not take state option! Creating new state object!");
            let new_osc_state_machine = OscStateMachine::new();
            Some(new_osc_state_machine)
        }
    };
    Ok(new_config)
}
