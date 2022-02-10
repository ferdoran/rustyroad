use env_logger::{Target, WriteStyle};
use log::LevelFilter;
use net::server::{Server, ServerSignal};

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

    let address = "127.0.0.1:8080";
    let server = Server::new(address).await.unwrap();
    let mut server_signal_receiver = server.start().await;
    // TODO: add a hook to handle system signals e.g. for graceful shutdown
    loop {
        // blocks the main process to handle server signals
        match server_signal_receiver.recv().await {
            Some(signal) => {
                match signal {
                    ServerSignal::Shutdown(msg) => {
                        info!("shutting down server: {}", msg);
                        return;
                    },
                    ServerSignal::Started => info!("server started listening on {}", address),
                    ServerSignal::NewConnection(msg) => info!("new session: {}", msg.to_string()),
                    ServerSignal::ClosedConnection(msg) => info!("closed session: {}", msg),
                }
            }
            None => {}
        }
    }
}