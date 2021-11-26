use crate::net::server::session::Session;
use tokio::net::{TcpListener};
use std::collections::HashMap;
use uuid::Uuid;
use futures::executor::block_on;

pub struct Server {
    listener: TcpListener,
    sessions: HashMap<Uuid, *const Session>,
}

impl Server {
    pub async fn new(addr: &str) -> Server {
        let listener = TcpListener::bind(addr).await.unwrap();
        return Server { listener, sessions: HashMap::new() };
    }

    pub fn start(mut self) {
        println!("started listening on {}", self.listener.local_addr().unwrap().to_string());
        loop {
           block_on(self.listener.accept())
                .map(|(s, _)| {
                    let session = Session::new();
                    self.sessions.insert(session.id, &session);
                    session.start(s);
                });
        }
    }
}