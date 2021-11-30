use log::{debug, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub const BUFFER_SIZE: usize = 4096;
const INCOMING_CHANNEL_SIZE: usize = 32;
const OUTGOING_CHANNEL_SIZE: usize = 32;

pub struct Session {
    pub id: Uuid
}

impl Session {
    pub fn new(id: Uuid) -> Session {
        return Session {id};
    }

    pub async fn start(self, stream: TcpStream, dc_sender: Sender<Uuid>) -> (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>, Sender<()>) {
        let (interrupt_sender, mut interrupt_receiver) = mpsc::channel::<()>(1);
        let (incoming_sender, incoming_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(INCOMING_CHANNEL_SIZE);
        let (outgoing_sender, mut outgoing_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(OUTGOING_CHANNEL_SIZE);
        let sid = self.id;
        let (mut read_half, mut write_half) = tokio::io::split(stream);
        tokio::spawn(async move {
            loop {
               let mut read_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
               select! {
                   interrupted = interrupt_receiver.recv() => {
                       match interrupted {
                           Some(_) => {
                               debug!("stopping session {}", sid);
                               break;
                           },
                           None => {}
                       }
                   },
                   read_result = read_half.read(&mut read_buf) => {
                       let read_bytes = match read_result { // TODO: maybe add metrics for read_bytes in the future
                           Ok(n) if n == 0 => {
                               debug!("client terminated connection");
                               break;
                           },
                           Ok(n) => {
                               debug!("session {}: {:02X?}", sid, read_buf);
                               incoming_sender.send(read_buf).await;
                               n
                           },
                           Err(e) => {
                               warn!("session {} failed to read from socket: {:?}", sid, e);
                               break;
                           }
                       };
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
                           },
                           // stop handling when either incoming or outgoing channel is closed
                           None => {
                               break;
                           }
                       }
                   }
               }
            }

            dc_sender.send(sid).await;
        });

        return (outgoing_sender, incoming_receiver, interrupt_sender);
    }
}

