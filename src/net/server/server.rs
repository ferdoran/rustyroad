use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
use log::error;

use crate::net::server::session::{BUFFER_SIZE, Session};

pub struct Server {
    listener: TcpListener,
    sessions: HashMap<Uuid, (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>)>
}

pub enum ServerSignal {
    Started,
    NewConnection(Uuid),
    ClosedConnection(Uuid),
    Shutdown(String)
}

impl Server {
    pub async fn new(addr: &str) -> Server {
        let listener = TcpListener::bind(addr).await.unwrap();
        return Server { listener, sessions: HashMap::new() };
    }

    pub async fn start(mut self) -> Receiver<ServerSignal> {
        let (server_signal_sender, server_signal_receiver) = mpsc::channel::<ServerSignal>(2);
        tokio::spawn(async move {
            let (disconnected_session_sender, mut disconnected_session_receiver) = mpsc::channel::<Uuid>(32);
            match server_signal_sender.send(ServerSignal::Started).await {
                Ok(()) => {},
                Err(err) => error!("failed to send msg: {}", err)
            };
            loop {
               tokio::select! {
                   conn_result = self.listener.accept() => {
                       match conn_result {
                           Ok((stream, _)) => {
                               let sid = Uuid::new_v4();
                               let session = Session::new(sid);
                               server_signal_sender.send(ServerSignal::NewConnection(sid)).await;
                               // TODO: do something with the channels
                               let (out_channel_sender, in_channel_receiver, session_interrupt_sender) = session.start(stream, disconnected_session_sender.clone()).await;
                               self.sessions.insert(sid, (out_channel_sender, in_channel_receiver));
                           }
                           Err(err) => {
                               server_signal_sender.send(ServerSignal::Shutdown(err.to_string())).await;
                               drop(server_signal_sender);
                               return;
                           }
                       }
                   },
                   dced_result = disconnected_session_receiver.recv() => {
                       match dced_result {
                           Some(sid) => {
                               server_signal_sender.send(ServerSignal::ClosedConnection(sid)).await;
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
