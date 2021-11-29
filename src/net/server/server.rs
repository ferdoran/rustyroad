use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::net::server::session::{BUFFER_SIZE, Session};

pub struct Server {
    listener: TcpListener,
    sessions: HashMap<Uuid, (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>)>
}

pub enum ServerSignal {
    Started,
    NewConnection(String),
    Shutdown(String)
}

impl Server {
    pub async fn new(addr: &str) -> Server {
        let listener = TcpListener::bind(addr).await.unwrap();
        return Server { listener, sessions: HashMap::new() };
    }

    pub async fn start(mut self) -> Receiver<ServerSignal> {
        let (disconnected_session_sender, mut disconnected_session_receiver) = mpsc::channel::<Uuid>(32);
        let (server_signal_sender, server_signal_receiver) = mpsc::channel::<ServerSignal>(2);
        tokio::spawn(async move {
            match server_signal_sender.send(ServerSignal::Started).await {
                Ok(()) => {},
                Err(err) => eprintln!("failed to send msg: {}", err)
            };
            loop {
               tokio::select! {
                   conn_result = self.listener.accept() => {
                       match conn_result {
                           Ok((stream, _)) => {
                               let sid = Uuid::new_v4();
                               let mut session = Session::new(sid, disconnected_session_sender.clone());
                               server_signal_sender.send(ServerSignal::NewConnection(format!("new connection from: {}", stream.peer_addr().unwrap().to_string())));
                               let (out_channel_sender, in_channel_receiver) = session.start(stream).await;
                               self.sessions.insert(sid, (out_channel_sender, in_channel_receiver));
                           }
                           Err(err) => {
                               let _ = server_signal_sender.send(ServerSignal::Shutdown(err.to_string()));
                               drop(server_signal_sender);
                               return;
                           }
                       }
                   },
                   dced_result = disconnected_session_receiver.recv() => {
                       match dced_result {
                           Some(sid) => {
                               println!("session {} closed", sid.to_string());
                               self.sessions.remove(&sid);
                           },
                           None => {}
                       }
                   }
               }
            }
        });

        return server_signal_receiver;
    }
}
