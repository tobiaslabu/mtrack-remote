#[cfg(feature = "server")]
use dioxus::logger::tracing::warn;
use dioxus::{logger::tracing::debug, prelude::*};

use gloo_timers::future::TimeoutFuture;

#[cfg(feature = "server")]
use crate::backend::config::Config;

#[cfg(feature = "server")]
use crate::backend::server::OscStateMachine;

#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::{
    backend::osc::MtrackState,
    components::{self},
};

enum UpdateMessage {}

/// Mtrack component that controls mtrack via the Dioxus fullstack API.
#[component]
pub fn Mtrack() -> Element {
    let client_state: Signal<Option<ClientState>> = use_signal(|| None);
    let mut client_state_move = client_state;
    let _update_routine = use_coroutine(move |_rx: UnboundedReceiver<UpdateMessage>| async move {
        debug!("Starting coroutine");
        client_state_move.set(None);

        loop {
            client_state_move.set(get_state().await.ok());
            let timeout_ms = 500;
            TimeoutFuture::new(timeout_ms).await;
        }
    });

    let client_state_view = match client_state.read().as_ref() {
        Some(state) => {
            let mtrack_state = state.mtrack_state.clone().unwrap_or(MtrackState::default());
            rsx!(
                div {
                    components::Transport {
                        is_playing: mtrack_state.is_playing,
                        elapsed: mtrack_state.time_elapsed,
                    }
                    components::Playlist {
                        songs: mtrack_state.setlist,
                        current_song: mtrack_state.song,
                    }
                }
            )
        }
        None => {
            rsx!(
                {"No info from server"}
            )
        }
    }?;
    rsx! {
        div { id: "mtrack",
            header { "mtrack remote" }
            {client_state_view}
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientState {
    is_connected: bool,
    mtrack_state: Option<MtrackState>,
}

/// Get app state
#[server]
async fn get_state() -> Result<ClientState, ServerFnError> {
    let FromContext(state_machine_option): FromContext<Arc<RwLock<Option<OscStateMachine>>>> =
        extract().await?;
    let mut state_machine_option = state_machine_option.write().await;

    let FromContext(config): FromContext<Arc<RwLock<Config>>> = extract().await?;

    let mut is_connected = false;
    let mut mtrack_state = None;
    *state_machine_option = match state_machine_option.take() {
        Some(mut state_machine) => {
            state_machine = state_machine.ensure_connection(*config.read().await).await;
            is_connected = state_machine.is_connected().await;

            mtrack_state = state_machine.get_mtrack_data().ok();
            Some(state_machine)
        }
        None => {
            warn!("Creating new OSC state machine");
            Some(OscStateMachine::new())
        }
    };

    debug!("Returning client state...");
    Ok(ClientState {
        is_connected,
        mtrack_state,
    })
}
