use dioxus::{
    logger::tracing::{debug, warn},
    prelude::*,
};
use dioxus_free_icons::icons::ld_icons::{LdCirclePlay, LdSkipBack, LdSkipForward};
use dioxus_free_icons::Icon;

#[cfg(feature = "server")]
use std::future::Future;

#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use tokio::sync::RwLock;

#[cfg(feature = "server")]
use crate::backend::server::{OscStateMachine, OscStateMachineError};

#[component]
fn Prev() -> Element {
    let icon = rsx!(Icon {
        class: "center",
        width: 24,
        height: 24,
        icon: LdSkipBack,
    })?;
    rsx!(
        button {
            class: "transport-button basis-1/4",
            onclick: move |_event| async move {
                debug!("Next");
                let _next_result = prev().await;
            },
            {icon}
        }
    )
}

#[component]
fn Play() -> Element {
    rsx!(
        button {
            class: "play-button basis-1/4",
            onclick: move |_event| async move {
                debug!("Play");
                match play().await {
                    Ok(_r) => debug!("OK"),
                    Err(err) => warn!("Error requesting play! {err}"),
                };
            },
            Icon {
                class: "center",
                width: 24,
                height: 24,
                icon: LdCirclePlay,
            }
        }
    )
}

#[component]
fn Stop() -> Element {
    rsx!(
        button {
            class: "stop-button basis-1/4",
            onclick: move |_event| async move {
                debug!("Stop");
                let _stop_result = stop().await;
            },
            "#"
        }
    )
}

#[component]
fn Next() -> Element {
    rsx!(
        button {
            class: "transport-button basis-1/4",
            onclick: move |_event| async move {
                debug!("Next");
                let _next_result = next().await;
            },
            Icon {
                class: "center",
                width: 24,
                height: 24,
                icon: LdSkipForward,
            }
        }
    )
}

#[component]
pub fn Transport(is_playing: bool, elapsed: String) -> Element {
    let play_or_stop = match is_playing {
        true => rsx!(Stop {}),
        false => rsx!(Play {}),
    };
    rsx!(
        div { class: "flex flex-row",
            Prev {}
            {play_or_stop}
            Next {}
            div { class: "time_elapsed basis-1/4", {elapsed} }
        }
    )
}

#[cfg(feature = "server")]
async fn run_osc_command<'a, T>(
    state_machine: &'a OscStateMachine,
    osc_fun: impl FnOnce(&'a OscStateMachine) -> T,
) -> Result<(), ServerFnError>
where
    T: Future<Output = Result<(), OscStateMachineError>>,
{
    if !state_machine.is_connected().await {
        return Err(ServerFnError::ServerError(
            "OSC state machine is not connected!".to_string(),
        ));
    }

    match osc_fun(state_machine).await {
        Ok(_result) => Ok(()),
        Err(err) => Err(ServerFnError::Response(err.to_string())),
    }
}

#[server(StartPlayback)]
async fn play() -> Result<(), ServerFnError> {
    let FromContext(state_machine_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let state_machine_option = state_machine_option.read().await;
    match state_machine_option.as_ref() {
        Some(state_machine) => run_osc_command(state_machine, OscStateMachine::play).await,
        None => Err(ServerFnError::ServerError(
            "OSC state machine is None!".to_string(),
        )),
    }
}

#[server(StopPlayback)]
async fn stop() -> Result<(), ServerFnError> {
    let FromContext(state_machine_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let state_machine_option = state_machine_option.read().await;

    match state_machine_option.as_ref() {
        Some(state_machine) => run_osc_command(state_machine, OscStateMachine::stop).await,
        None => Err(ServerFnError::ServerError(
            "OSC state machine is None!".to_string(),
        )),
    }
}

#[server(NextSong)]
async fn next() -> Result<(), ServerFnError> {
    let FromContext(state_machine_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let state_machine_option = state_machine_option.read().await;

    match state_machine_option.as_ref() {
        Some(state_machine) => run_osc_command(state_machine, OscStateMachine::next).await,
        None => Err(ServerFnError::ServerError(
            "OSC state machine is None!".to_string(),
        )),
    }
}

#[server(PrevSong)]
async fn prev() -> Result<(), ServerFnError> {
    let FromContext(state_machine_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let state_machine_option = state_machine_option.read().await;

    match state_machine_option.as_ref() {
        Some(state_machine) => run_osc_command(state_machine, OscStateMachine::prev).await,
        None => Err(ServerFnError::ServerError(
            "OSC state machine is None!".to_string(),
        )),
    }
}
