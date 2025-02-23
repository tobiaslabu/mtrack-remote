#[cfg(feature = "server")]
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

#[cfg(feature = "server")]
use dioxus::logger::tracing::{debug, error, info, span, warn, Level};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use thiserror::Error;

#[cfg(feature = "server")]
use rosc::encoder;

#[cfg(feature = "server")]
use rosc::{decoder::MTU, OscError, OscMessage, OscPacket};

#[cfg(feature = "server")]
use tokio::{
    net::UdpSocket,
    select,
    sync::{
        mpsc::{Receiver, Sender},
        RwLock,
    },
    task::JoinHandle,
};

#[cfg(feature = "server")]
use crate::backend::server::ServerMessage;

#[cfg(feature = "server")]
use super::config::Config;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MtrackState {
    pub is_playing: bool,
    pub time_elapsed: String,
    pub song: String,
    pub setlist: Vec<String>,
}

impl MtrackState {
    pub fn set_is_playing(&mut self, is_playing: String) {
        self.is_playing = matches!(is_playing.as_str(), "Playing");
    }

    pub fn set_setlist(&mut self, setlist: Vec<String>) {
        self.setlist = setlist;
    }

    pub fn set_current_song(&mut self, song: String) {
        self.song = song;
    }

    pub fn set_time_elapsed(&mut self, time_elapsed: String) {
        self.time_elapsed = time_elapsed;
    }
}

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct OscConnection {
    socket: Arc<RwLock<Option<UdpSocket>>>,
    mtrack: Arc<RwLock<MtrackState>>,
    osc_tx: Option<Sender<ServerMessage>>,
    task_handle: Arc<RwLock<Option<JoinHandle<Result<(), OscTransportError>>>>>,
}

#[cfg(feature = "server")]
#[derive(Debug, Error)]
pub enum OscTransportError {
    #[error("Could not receive from socket! {0}")]
    Receive(std::io::Error),
    #[error("Could not decode OSC packet! {0}")]
    Decode(OscError),
    #[error("Socket has not been initialized")]
    NotInitialized,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Could not send OSC message! {0}")]
    Send(String),
    #[error("Could not lock mtrack state! {0}")]
    LockMtrackError(String),
    #[error("Could not join task! {0}")]
    JoinError(tokio::task::JoinError),
}

#[cfg(feature = "server")]
enum OscRequests {
    GetSetlist,
    GetSong,
    Play,
    Stop,
    Next,
    Prev,
}

#[cfg(feature = "server")]
fn get_udp_buf(request: OscRequests) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let message = match request {
        OscRequests::GetSetlist => OscMessage {
            addr: "/mtrack/playlist".to_string(),
            args: vec![],
        },
        OscRequests::GetSong => OscMessage {
            addr: "/mtrack/song".to_string(),
            args: vec![],
        },
        OscRequests::Play => OscMessage {
            addr: "/mtrack/play".to_string(),
            args: vec![],
        },
        OscRequests::Stop => OscMessage {
            addr: "/mtrack/stop".to_string(),
            args: vec![],
        },
        OscRequests::Next => OscMessage {
            addr: "/mtrack/next".to_string(),
            args: vec![],
        },
        OscRequests::Prev => OscMessage {
            addr: "/mtrack/prev".to_string(),
            args: vec![],
        },
    };
    let packet = OscPacket::Message(message);
    match encoder::encode_into(&packet, &mut buf) {
        Ok(_num_bytes) => buf,
        Err(err) => {
            error!("Could not encode osc packet! {err}");
            buf
        }
    }
}

#[cfg(feature = "server")]
impl Default for OscConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
impl OscConnection {
    pub fn new() -> Self {
        debug!("Initializing OscConnection");
        let socket: Arc<RwLock<Option<UdpSocket>>> = Arc::new(RwLock::new(None));
        let osc_tx = None;
        let mtrack_state = MtrackState::default();
        let mtrack = Arc::new(RwLock::new(mtrack_state));
        let task_handle = Arc::new(RwLock::new(None));
        Self {
            socket,
            task_handle,
            mtrack,
            osc_tx,
        }
    }

    pub async fn init_socket(&mut self, config: Config) -> Result<(), OscTransportError> {
        info!("Initializing socket");

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.listen_port);
        match UdpSocket::bind(addr).await {
            Ok(s) => {
                debug!("Bound UDP socket");
                let mut writeable_socket = self.socket.write().await;
                *writeable_socket = Some(s);
                debug!("Saved socket");
            }
            Err(e) => {
                error!("Failed to bind socket! {addr} {e}");
                return Err(OscTransportError::IoError(e.to_string()));
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(16);
        let socket_move = self.socket.clone();
        let mtrack = self.mtrack.clone();
        let mtrack_addr = config.mtrack_addr;
        let osc_task = async move {
            let span = span!(Level::DEBUG, "OSC>>");
            let _entered = span.enter();
            debug!("Started backend coroutine");
            loop {
                if socket_move.read().await.is_none() {
                    debug!("Socket is none, quitting loop!");
                    break;
                }

                select! {
                    received_osc_result = OscConnection::read_from_socket(socket_move.clone()) => {
                        match received_osc_result {
                            Ok(osc_packet) => OscConnection::handle_osc_packet(&mtrack, &osc_packet).await,
                            Err(err) => {error!("Error reading OSC packet: {err}"); panic!()}
                        };
                    },
                    message_result = OscConnection::receive_through_channel(&mut rx) => {
                        debug!("Received something through channel");
                        debug!("Received {message_result:?}");
                        let mut socket_write = socket_move.write().await;
                        let osc_request = match message_result {
                            ServerMessage::GetSetlist => OscRequests::GetSetlist,
                            ServerMessage::GetSong => OscRequests::GetSong,
                            ServerMessage::Play => OscRequests::Play,
                            ServerMessage::Stop => OscRequests::Stop,
                            ServerMessage::Next => OscRequests::Next,
                            ServerMessage::Prev => OscRequests::Prev,
                            ServerMessage::Disconnect => {
                                debug!("Received disconnect message.");
                                *socket_write = None;
                                break;
                            },
                        };

                        match socket_write.as_ref() {
                            Some(socket) => {
                                let buf = get_udp_buf(osc_request);
                                match socket.send_to(&buf, mtrack_addr).await {
                                    Ok(ok_result) => {
                                        debug!("Sent UDP message {ok_result}");
                                    },
                                    Err(err) => {
                                        error!("Failed to send through socket! {err}");
                                    },
                                };
                            },
                            None => {
                                error!("Socket has not been initialized, cannot write to it!");
                            },
                        };
                    },
                }
            }
            Ok(())
        };

        let task_handle = tokio::spawn(osc_task);

        self.osc_tx = Some(tx);
        let mut task_handle_write = self.task_handle.write().await;
        *task_handle_write = Some(task_handle);

        debug!("Got join handle");
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), OscTransportError> {
        info!("Disconnecting socket");
        match &self.osc_tx {
            Some(tx) => match tx.send(ServerMessage::Disconnect).await {
                Ok(_send_result) => {
                    let task_handle_clone = self.task_handle.clone();
                    let mut handle_write = task_handle_clone.write().await;
                    match handle_write.take() {
                        Some(handle) => match handle.await {
                            Ok(result) => result,
                            Err(err) => {
                                error!("Could not join task! {err}");
                                Err(OscTransportError::JoinError(err))
                            }
                        },
                        None => Ok(()),
                    }
                }
                Err(err) => {
                    error!("Failed to send disconnect message! {err}");
                    Err(OscTransportError::Send(err.to_string()))
                }
            },
            None => {
                error!("osc_tx is None!");
                Err(OscTransportError::NotInitialized)
            }
        }
    }

    async fn read_from_socket(
        socket: Arc<RwLock<Option<UdpSocket>>>,
    ) -> Result<OscPacket, OscTransportError> {
        let socket = socket.read().await;

        match socket.as_ref() {
            Some(socket) => {
                let mut buf = Vec::with_capacity(MTU);
                let (_bytes_received, _from_address) = match socket.recv_buf_from(&mut buf).await {
                    Ok(ok_result) => ok_result,
                    Err(err) => return Err(OscTransportError::Receive(err)),
                };
                match rosc::decoder::decode_udp(&buf) {
                    Ok((_remainder, osc_packet)) => Ok(osc_packet),
                    Err(err) => Err(OscTransportError::Decode(err)),
                }
            }
            None => {
                warn!("Socket is None, cannot read from it!");
                Err(OscTransportError::NotInitialized)
            }
        }
    }

    async fn receive_through_channel(rx: &mut Receiver<ServerMessage>) -> ServerMessage {
        match rx.recv().await {
            Some(message) => message,
            None => ServerMessage::Disconnect,
        }
    }

    async fn handle_osc_message(state: &Arc<RwLock<MtrackState>>, osc_message: &OscMessage) {
        let mut state_mut = state.write().await;
        match osc_message.addr.as_str() {
            "/mtrack/playlist/current_song" => {
                (*state_mut).set_current_song(
                    osc_message
                        .args
                        .iter()
                        .filter_map(|e| e.clone().string())
                        .collect(),
                );
            }
            "/mtrack/playlist/current" => {
                (*state_mut).set_setlist(
                    osc_message
                        .args
                        .iter()
                        .filter_map(|e| e.clone().string())
                        .flat_map(|e| {
                            e.split("\n")
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                        })
                        .collect(),
                );
            }
            "/mtrack/playlist/current_song/elapsed" => {
                (*state_mut).set_time_elapsed(
                    osc_message
                        .args
                        .iter()
                        .filter_map(|e| e.clone().string())
                        .collect(),
                );
            }
            "/mtrack/status" => {
                (*state_mut).set_is_playing(
                    osc_message
                        .args
                        .iter()
                        .filter_map(|e| e.clone().string())
                        .collect(),
                );
            }
            _ => {
                let addr = &osc_message.addr;
                debug!("Received unknown OSC address {addr}");
                let args = &osc_message.args; //.iter().map(|e| e.).join(", ");
                debug!("args: {args:?}")
            }
        };
    }

    async fn handle_osc_packet(state: &Arc<RwLock<MtrackState>>, osc_packet: &OscPacket) {
        match osc_packet {
            OscPacket::Message(osc_message) => {
                OscConnection::handle_osc_message(state, osc_message).await
            }
            OscPacket::Bundle(osc_bundle) => {
                for osc_packet in osc_bundle.content.iter() {
                    Box::pin(OscConnection::handle_osc_packet(state, osc_packet)).await;
                }
            }
        };
    }

    pub async fn is_connected(&self) -> bool {
        match self.socket.read().await.as_ref() {
            Some(_socket) => true,
            None => false,
        }
    }

    pub fn get_state(&self) -> Result<MtrackState, OscTransportError> {
        match self.mtrack.try_read() {
            Ok(mtrack) => Ok(mtrack.clone()),
            Err(err) => Err(OscTransportError::LockMtrackError(err.to_string())),
        }
    }

    pub async fn fetch_song(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::GetSong).await
    }

    pub async fn fetch_setlist(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::GetSetlist).await
    }

    async fn send_osc_message(&self, message: ServerMessage) -> Result<(), OscTransportError> {
        match &self.osc_tx {
            Some(tx) => match tx.send(message).await {
                Ok(result) => Ok(result),
                Err(err) => Err(OscTransportError::Send(err.to_string())),
            },
            None => Err(OscTransportError::NotInitialized),
        }
    }

    pub async fn play(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::Play).await
    }

    pub async fn stop(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::Stop).await
    }

    pub async fn next(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::Next).await
    }

    pub async fn prev(&self) -> Result<(), OscTransportError> {
        self.send_osc_message(ServerMessage::Prev).await
    }
}

#[cfg(feature = "server")]
impl Drop for OscConnection {
    fn drop(&mut self) {
        debug!("Dropping OSC connection");
        match self.socket.try_read() {
            Ok(socket_read) => {
                if socket_read.is_some() {
                    panic!("Dropping while still connected!")
                }
            }
            Err(err) => {
                panic!("Could not get read lock on socket! {err}")
            }
        };
        {}
    }
}
