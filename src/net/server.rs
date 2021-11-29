use tokio::net::TcpListener;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::net::server::session::BUFFER_SIZE;

pub struct Server {
    listener: TcpListener,
    sessions: HashMap<Uuid, (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>)>,
}
mod session;
mod server;
