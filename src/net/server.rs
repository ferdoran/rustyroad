use tokio::net::TcpListener;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::net::server::session::BUFFER_SIZE;

/// An async TCP server with session management capabilities
pub struct Server {
    listener: TcpListener,
    sessions: HashMap<Uuid, (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>, Sender<()>)>,
}

/// Defined signals the [Server] sends via returned channel on start ([Server::start])
pub enum ServerSignal {
    Started,
    Shutdown(String),
    NewConnection(Uuid),
    ClosedConnection(Uuid)
}

mod session;
mod server;
