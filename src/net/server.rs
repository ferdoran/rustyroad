use std::collections::HashMap;

use tokio::net::TcpListener;
use tokio::sync::mpsc::{Sender};
use uuid::Uuid;

use crate::net::server::session::BUFFER_SIZE;

/// General type definition for an SRO packet
type Packet = [u8; BUFFER_SIZE];
/// A type definition holding a session's channels as triplet. Used for code simplification
type SessionChannels = (Sender<()>, Sender<Packet>);

/// An async TCP server with session management capabilities
pub struct Engine {
    listener: TcpListener,
    sessions: HashMap<Uuid, SessionChannels>,
}


/// Defined signals the [Server] sends via returned channel on start ([Server::start])
pub enum ServerSignal {
    Started,
    Shutdown(String),
    NewConnection(Uuid),
    ClosedConnection(Uuid)
}

mod session;
mod engine;