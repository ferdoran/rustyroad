use lazy_static::lazy_static;
use log::{debug, warn};
use prometheus::{register_int_counter};
use prometheus::core::{AtomicU64, GenericCounter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender};
use uuid::Uuid;
use crate::net::server::SessionChannels;

pub const BUFFER_SIZE: usize = 4096;
const INCOMING_CHANNEL_SIZE: usize = 32;
const OUTGOING_CHANNEL_SIZE: usize = 32;

lazy_static! {
    static ref RECEIVED_BYTES_COUNTER: GenericCounter<AtomicU64> = register_int_counter!("net_server_received_bytes", "amount of received bytes").expect("failed to register counter net_server_received_bytes");
    static ref SENT_BYTES_COUNTER: GenericCounter<AtomicU64> = register_int_counter!("net_server_sent_bytes", "amount of sent bytes").expect("failed to register counter net_server_sent_bytes");
}

/// An identified TCP client session
pub struct Session {
    pub id: Uuid,
}

impl Session {
    /// Creates a new session given a [Uuid]
    pub fn new(id: Uuid) -> Session {
        Session { id }
    }

    /// Starts handling incoming and outgoing data.
    ///
    /// Returns a:
    /// * [Sender] for sending data back to the client and
    /// * [Receiver] to handle incoming data.
    /// * [Sender] to interrupt or close the session.
    ///
    /// Be aware that it's a multi-producer-single-consumer channel.
    pub async fn start(self, stream: TcpStream, dc_sender: Sender<Uuid>) -> SessionChannels {
        let (interrupt_sender, mut interrupt_receiver) = mpsc::channel::<()>(1);
        let (incoming_sender, incoming_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(INCOMING_CHANNEL_SIZE);
        let (outgoing_sender, mut outgoing_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(OUTGOING_CHANNEL_SIZE);
        let sid = self.id;
        let (mut read_half, mut write_half) = tokio::io::split(stream);
        tokio::spawn(async move {
            loop {
                let mut read_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
                select! {
                   // Handle either an interruption, incoming data, or outgoing data, whatever occurs first
                   interrupted = interrupt_receiver.recv() => {
                       if interrupted.is_some() {
                            debug!("stopping session {}", sid);
                            break;
                       }
                   },
                   read_result = read_half.read(&mut read_buf) => {
                       let _read_bytes = match read_result {
                           Ok(n) if n == 0 => {
                               debug!("client terminated connection");
                               break;
                           },
                           Ok(n) => {
                               debug!("session {}: {:02X?}", sid, read_buf);
                               let send_result = incoming_sender.send(read_buf).await;
                               match send_result {
                                   Err(err) => {
                                       error!("failed to send client {} incoming data ({} bytes) to channel", n, err);
                                       break;
                                   },
                                   Ok(_) => n
                               }
                           },
                           Err(e) => {
                               warn!("session {} failed to read from socket: {:?}", sid, e);
                               break;
                           }
                       };
                       RECEIVED_BYTES_COUNTER.inc_by(_read_bytes as u64);
                   },
                   out_channel_result = outgoing_receiver.recv() => {
                       match out_channel_result {
                           Some(out_data) => {
                               match write_half.write_all(&out_data).await {
                                   Ok(_) => {},
                                   Err(e) => {
                                       warn!("closing session {}: failed to write buffer: {}", sid, e);
                                       outgoing_receiver.close();
                                       break;
                                   }
                               };
                               SENT_BYTES_COUNTER.inc_by(out_data.len() as u64);
                           },
                           // stop handling when either incoming or outgoing channel is closed
                           None => {
                               break;
                           }
                       }
                   }
               }
            }
            if let Err(err) = dc_sender.send(sid).await {
                warn!("failed to send disconnected client signal to channel: {}", err);
            }
        });


        (interrupt_sender, outgoing_sender, incoming_receiver)
    }
}

