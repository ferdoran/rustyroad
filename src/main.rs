use std::convert::Infallible;
use std::net::SocketAddr;
use env_logger::{Target, WriteStyle};
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::LevelFilter;
use prometheus::{Encoder, TextEncoder};
use net::server::{Server, ServerSignal};

mod net;
#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder
        .filter_module("rustyroad", LevelFilter::Trace)
        .target(Target::Stdout)
        .write_style(WriteStyle::Always)
        .init();
    let address = ":8080";
    let server = Server::new(address).await.unwrap();
    let mut server_signal_receiver = server.start().await;
    // TODO: add a hook to handle system signals e.g. for graceful shutdown

    tokio::spawn(serve_metrics());

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

async fn serve_metrics() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(prometheus_metrics))
    });

    let server = hyper::Server::bind(&addr).serve(make_svc);
    info!("started http server on {}", addr);
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn prometheus_metrics(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        // Provide prometheus metrics.
        (&Method::GET, "/metrics") => {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();

            // Gather the metrics.
            let metric_families = prometheus::gather();
            // Encode them to send.
            encoder.encode(&metric_families, &mut buffer).unwrap();

            let output = String::from_utf8(buffer.clone()).unwrap();
            Ok(Response::new(Body::from(output)))
        }
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}