use env_logger::{Target, WriteStyle};
use log::LevelFilter;
use rustyroad::net::server::{Server, ServerSignal};
use crate::net::server::Server;

mod net;

#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .filter_module("rustyroad", LevelFilter::Trace)
        .target(Target::Stdout)
        .write_style(WriteStyle::Always)
        .init();

    let server = Server::new("127.0.0.1:8080").await;
    let mut server_signal_receiver = server.start().await;
    loop {
        match server_signal_receiver.recv().await {
            Some(signal) => {
                match signal {
                    ServerSignal::Shutdown(msg) => {
                        eprintln!("shutting down server: {}", msg);
                        return;
                    },
                    ServerSignal::Started => info!("server started"),
                    ServerSignal::NewConnection(msg) => info!("new session: {}", msg.to_string()),
                    ServerSignal::ClosedConnection(msg) => info!("closed session: {}", msg),
                }
            }
            None => {}
        }
    }
}