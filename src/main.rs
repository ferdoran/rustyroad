mod net;

#[tokio::main]
async fn main() {
    let server = crate::net::server::server::Server::new("127.0.0.1:8080").await;
    server.start();
}