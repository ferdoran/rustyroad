use tokio::net::TcpListener;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::net::server::session::BUFFER_SIZE;

/// A type definition holding a session's channels as triplet. Used for code simplification
type Packet = [u8; BUFFER_SIZE];
type SessionChannels = (Sender<()>, Sender<Packet>, Receiver<Packet>);

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