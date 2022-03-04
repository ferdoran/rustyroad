use std::collections::HashMap;
use std::io::Error;
use prometheus::{register_int_counter, register_int_gauge};

use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::net::server::session::Session;
use crate::{Server, ServerSignal};

impl Server {
    /// Creates a new server instance for given address. Fails if binding is not successful
    pub async fn new(addr: &str) -> Result<Server, Error> {
        let bind_result = TcpListener::bind(addr).await;
        return match bind_result {
            Ok(listener) => Ok(Server { listener, sessions: HashMap::new() }),
            Err(err) => {
                error!("failed to bind to address {}", addr);
                Err(err)
            }
        };
    }

    /// Starts the handling of incoming connections.
    /// Returns a [Receiver] to inform about certain events.
    pub async fn start(mut self) -> Receiver<ServerSignal> {
        let (server_signal_sender, server_signal_receiver) = mpsc::channel::<ServerSignal>(2);
        tokio::spawn(async move {
            let (disconnected_session_sender, mut disconnected_session_receiver) = mpsc::channel::<Uuid>(32);
            handle_signal_result(server_signal_sender.send(ServerSignal::Started).await);
            let sessions_gauge = register_int_gauge!("net_server_sessions", "current amount of sessions").expect("failed to register gauge net_server_sessions");
            let failed_accepts_counter = register_int_counter!("net_server_failed_accepts", "total number of connections which the server could not accept due to an error").expect("failed to register counter net_server_failed_accepts");
            loop {
                select! {
                   // Handle either a connection or a disconnection, whatever occurs first
                   conn_result = self.listener.accept() => {
                       match conn_result {
                           Ok((stream, _)) => {
                               // New client/connection
                               let sid = Uuid::new_v4();
                               let session = Session::new(sid);
                               handle_signal_result(server_signal_sender.send(ServerSignal::NewConnection(sid)).await);
                               // TODO: do something with the channels
                               let (out_channel_sender, in_channel_receiver, _session_interrupt_sender) = session.start(stream, disconnected_session_sender.clone()).await;
                               self.sessions.insert(sid, (out_channel_sender, in_channel_receiver, _session_interrupt_sender));
                           }
                           Err(err) => {
                               // Failed to accept a connection
                               handle_signal_result(server_signal_sender.send(ServerSignal::Shutdown(err.to_string())).await);
                               drop(server_signal_sender);
                               failed_accepts_counter.inc();
                               return;
                           }
                       }
                   },
                   dced_result = disconnected_session_receiver.recv() => {
                       if let Some(sid) = dced_result {
                           handle_signal_result(server_signal_sender.send(ServerSignal::ClosedConnection(sid)).await);
                           self.sessions.remove(&sid);
                       }
                   }
               }
               sessions_gauge.set(self.sessions.len() as i64)
            }
        });

        server_signal_receiver
    }
}

/// Logs the failed signal as warning
fn handle_signal_result(result: Result<(), SendError<ServerSignal>>) {
    if let Err(err) = result {
        warn!("failed to send signal to server signal channel: {}", err);
    }
}
