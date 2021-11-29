use crate::net::server::server::ServerSignal;
use crate::net::server::server::Server;

mod net;

#[tokio::main]
async fn main() {
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
                    ServerSignal::Started => println!("server started"),
                    ServerSignal::NewConnection(msg) => println!("{}", msg)
                }
            }
            None => {}
        }
    }
}