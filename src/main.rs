use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use env_logger::{Target, WriteStyle};
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::LevelFilter;
use prometheus::{Encoder, TextEncoder};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use net::server::{Engine, ServerSignal};
use crate::pk2::archive::Archive;
use crate::pk2::entry::EntryType;
use crate::pk2::errors::Error;

mod net;
mod blowfish;
mod pk2;

#[macro_use]
extern crate log;
extern crate core;


#[tokio::main]
async fn main() {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder
        .filter_module("rustyroad", LevelFilter::Trace)
        .target(Target::Stdout)
        .write_style(WriteStyle::Always)
        .init();
    let file = String::from("/Users/rmu/workspaces/private/go-sro-agent-server/Data.pk2");
    let mut archive = Archive::open(Path::new(&file)).unwrap();
    match archive.index() {
        Err(err) => {
            let msg = match err {
                Error::InvalidHeader(msg) => String::from(msg),
                Error::IO(er) => er.to_string(),
                Error::InvalidBlock(msg) => String::from(msg)
            };
            panic!("failed to index archive: {}", msg);
        }
        Ok(dir) => dir.print_entries()
    };

    let server = Engine::new(Vec::new()).await;
    let (mut server_signal_receiver, packet_receiver) = server.start().await.unwrap();
    tokio::spawn(serve_metrics());
    tokio::spawn(async move {
        // TODO: build a packet stream that does following in order
        //  1) decrypt packet
        //  2) verify checksum
        //  3) unwrap massive packet
        //  4) handle packet
        // TODO: remove this example
        let mut receiver_stream = ReceiverStream::new(packet_receiver);
        while let Some((uuid, data)) = receiver_stream.next().await {
            debug!("session {} received: {:02X?}", uuid.to_string(), data);
        }
    });
    loop {
        // blocks the main process to handle server signals
        if let Some(signal) = server_signal_receiver.recv().await {
            match signal {
                ServerSignal::Shutdown(msg) => {
                    info!("shutting down server: {}", msg);
                    return;
                }
                ServerSignal::NewConnection(msg) => debug!("new session: {}", msg.to_string()),
                ServerSignal::ClosedConnection(msg) => debug!("closed session: {}", msg),
                ServerSignal::Started => {}
            }
        }
    }
    // TODO: add a hook to handle system signals e.g. for graceful shutdown
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