use thiserror::Error;

#[cfg(feature = "server")]
use dioxus::logger::tracing::{debug, error};

#[cfg(feature = "server")]
use super::{config::Config, osc::OscConnection};

#[cfg(feature = "server")]
use super::osc::MtrackState;

#[derive(Debug, Error)]
pub enum OscStateMachineError {
    #[error("Could not connect to OSC endpoint! {0}")]
    CouldNotConnect(String),
    #[error("Could not disconnect properly! {0}")]
    CouldNotDisconnect(String),
    #[error("Not connected yet!")]
    NotConnected,
    #[error("OSC error! {0}")]
    Osc(String),
}

#[cfg(feature = "server")]
#[derive(Debug)]
pub enum State {
    Disconnected,
    Connected(OscConnection),
}

#[cfg(feature = "server")]
impl State {}

#[cfg(feature = "server")]
#[derive(Debug)]
pub struct OscStateMachine {
    pub state: State,
}

#[derive(Debug)]
pub struct Unconfigured {}

#[cfg(feature = "server")]
impl Default for OscStateMachine {
    fn default() -> Self {
        Self {
            state: State::Disconnected,
        }
    }
}

#[cfg(feature = "server")]
impl OscStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn is_connected(&self) -> bool {
        match &self.state {
            State::Disconnected => false,
            State::Connected(_) => true,
        }
    }

    pub async fn disconnect(self) -> Self {
        match self.state {
            State::Disconnected => self,
            State::Connected(mut osc_connection) => {
                match osc_connection.disconnect().await {
                    Ok(_) => debug!("Disconnected properly."),
                    Err(err) => error!("Failed to disconnect properly! {err}"),
                };
                Self {
                    state: State::Disconnected,
                }
            }
        }
    }

    pub async fn ensure_connection(mut self, config: Config) -> Self {
        if !self.is_connected().await {
            let mut osc_connection = OscConnection::new();
            match osc_connection.init_socket(config).await {
                Ok(_) => {
                    self.state = State::Connected(osc_connection);
                }
                Err(error) => {
                    error!("Could not connect OSC! {error}");
                }
            }
        } else {
            debug!("Already connected...");
        }
        self
    }

    pub fn get_mtrack_data(&self) -> Result<MtrackState, OscStateMachineError> {
        match &self.state {
            State::Disconnected => Err(OscStateMachineError::NotConnected),
            State::Connected(osc_connection) => match osc_connection.get_state() {
                Ok(mtrack) => Ok(mtrack),
                Err(err) => Err(OscStateMachineError::Osc(err.to_string())),
            },
        }
    }

    pub async fn play(&self) -> Result<(), OscStateMachineError> {
        match &self.state {
            State::Disconnected => Err(OscStateMachineError::NotConnected),
            State::Connected(osc_connection) => match osc_connection.play().await {
                Ok(result) => {
                    debug!("Asked OSC routine to play song");
                    Ok(result)
                }
                Err(err) => {
                    error!("Could not request playing song! {err:?}");
                    Err(OscStateMachineError::Osc(err.to_string()))
                }
            },
        }
    }

    pub async fn stop(&self) -> Result<(), OscStateMachineError> {
        debug!("Stop..");
        match &self.state {
            State::Disconnected => Err(OscStateMachineError::NotConnected),
            State::Connected(osc_connection) => match osc_connection.stop().await {
                Ok(result) => {
                    debug!("Asked OSC routine to stop song");
                    Ok(result)
                }
                Err(err) => {
                    error!("Could not request playing song! {err:?}");
                    Err(OscStateMachineError::Osc(err.to_string()))
                }
            },
        }
    }

    pub async fn next(&self) -> Result<(), OscStateMachineError> {
        debug!("Next..");
        match &self.state {
            State::Disconnected => Err(OscStateMachineError::NotConnected),
            State::Connected(osc_connection) => match osc_connection.next().await {
                Ok(result) => {
                    debug!("Asked OSC routine to skip to next song");
                    Ok(result)
                }
                Err(err) => {
                    error!("Could not request skipping to next song! {err:?}");
                    Err(OscStateMachineError::Osc(err.to_string()))
                }
            },
        }
    }

    pub async fn prev(&self) -> Result<(), OscStateMachineError> {
        debug!("Prev..");
        match &self.state {
            State::Disconnected => Err(OscStateMachineError::NotConnected),
            State::Connected(osc_connection) => match osc_connection.prev().await {
                Ok(result) => {
                    debug!("Asked OSC routine to skip to prev song");
                    Ok(result)
                }
                Err(err) => {
                    error!("Could not request skipping to prev song! {err:?}");
                    Err(OscStateMachineError::Osc(err.to_string()))
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum ServerMessage {
    GetSetlist,
    GetSong,
    Disconnect,
    Play,
    Stop,
    Next,
    Prev,
}

#[cfg(test)]
pub mod tests {
    #[cfg(feature = "server")]
    use super::OscStateMachine;
    #[cfg(feature = "server")]
    use crate::backend::config::Config;

    #[cfg(feature = "server")]
    #[test]
    fn create_state_machine() {
        let s = OscStateMachine::new();
        let config = Config::default();
        println!("State: {s:?}");
        println!("Config: {config:?}");
    }
}
