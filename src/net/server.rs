use std::collections::HashMap;

use tokio::sync::mpsc::{Sender};
use uuid::Uuid;

use crate::net::server::session::BUFFER_SIZE;

/// General type definition for an SRO packet
type Packet = [u8; BUFFER_SIZE];
/// A type definition holding a session's channels as triplet. Used for code simplification
type SessionChannels = (Sender<()>, Sender<Packet>);

/// An async TCP server with session management capabilities
pub struct Engine {
    bind_host: &'static str,
    bind_port: u16,
    sessions: HashMap<Uuid, SessionChannels>,
}


/// Defined signals the [Server] sends via returned channel on start ([Server::start])
pub enum ServerSignal {
    Started,
    Shutdown(String),
    NewConnection(Uuid),
    ClosedConnection(Uuid)
}

// type ServerOpt = fn(&mut Engine);

mod session;
mod engine;
mod options;