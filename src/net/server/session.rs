use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub const BUFFER_SIZE: usize = 4096;
const INCOMING_CHANNEL_SIZE: usize = 32;
const OUTGOING_CHANNEL_SIZE: usize = 32;

pub struct Session {
    pub id: Uuid,
    dc_sender: Sender<Uuid>,
    inc_handle: Option<JoinHandle<()>>,
    out_handle: Option<JoinHandle<()>>
}

impl Session {
    pub fn new(id: Uuid, dc_sender: Sender<Uuid>) -> Session {
        return Session {
            id,
            dc_sender,
            inc_handle: Option::None,
            out_handle: Option::None,
        };
    }

    pub async fn start(&mut self, stream: TcpStream) -> (Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>) {
        let (read_half, write_half) = tokio::io::split(stream);
        let incoming_receiver = self.handle_incoming_data(read_half);
        let outgoing_sender = self.handle_outgoing_data(write_half);

        return (outgoing_sender, incoming_receiver);
    }

    fn handle_outgoing_data(&mut self, mut write_half: WriteHalf<TcpStream>) -> Sender<[u8; BUFFER_SIZE]> {
        let (outgoing_sender, mut outgoing_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(OUTGOING_CHANNEL_SIZE);
        let sid = self.id.clone();
        let dc_sender = self.dc_sender.clone();
        self.out_handle = Option::Some(tokio::spawn(async move {
            loop {
                match outgoing_receiver.recv().await {
                    Some(data) => {
                        match write_half.write_all(&data).await {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("closing session {}: failed to write buffer: {}", sid, e);
                                break;
                            }
                        };
                    },
                    None => {
                        // stop handling outgoing data when outgoing channel is closed
                        break;
                    }
                };
            }
            outgoing_receiver.close();
            dc_sender.send(sid).await;
            drop(outgoing_receiver);
            drop(write_half);
            drop(dc_sender);
        }));

        return outgoing_sender;
    }

    fn handle_incoming_data(&mut self, mut read_half: ReadHalf<TcpStream>) -> Receiver<[u8; BUFFER_SIZE]> {
        let (incoming_sender, incoming_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(INCOMING_CHANNEL_SIZE);
        let sid = self.id.clone();
        let dc_sender = self.dc_sender.clone();
        self.inc_handle = Option::Some(tokio::spawn(async move {
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match read_half.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => {
                        println!("client terminated connection");
                        break;
                    },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("session {} failed to read from socket: {:?}", sid, e);
                        break;
                    }
                };

                println!("session {}: read {} bytes", sid, n);
                incoming_sender.send(buf).await;
            }
            dc_sender.send(sid).await;
            drop(incoming_sender);
            drop(read_half);
            drop(dc_sender);
        }));

        return incoming_receiver;
    }

    pub async fn stop(mut self) {
        self.dc_sender.send(self.id).await;
        self.inc_handle.unwrap().abort();
        self.out_handle.unwrap().abort();

        self.inc_handle = Option::None;
        self.out_handle = Option::None;
    }
}

