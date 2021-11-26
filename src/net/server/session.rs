use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use uuid::Uuid;

const BUFFER_SIZE: usize = 4096;

pub struct Session {
    pub id: Uuid,
    inc_handle: Option<JoinHandle<()>>,
    out_handle: Option<JoinHandle<()>>,
}

// impl From<TcpStream> for Session {
//     fn from(stream: &TcpStream) -> Self {
//         return Session {
//             stream,
//             id: Uuid::new_v4(),
//             inc_handle: Option::None,
//             out_handle: Option::None
//         }
//     }
// }

impl Session {
    pub fn new() -> Session {
        return Session {
            id: Uuid::new_v4(),
            inc_handle: Option::None,
            out_handle: Option::None,
        };
    }

    pub async fn start(mut self, stream: TcpStream) -> Result<(Sender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>), String> {
        println!("Client {} started", self.id);
        let (incoming_sender, incoming_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(32);
        let (mut read_half, mut write_half) = tokio::io::split(stream);
        let session_id = self.id.to_string();
        // read incoming packets and send to channel
        let inc_handle = Option::Some(tokio::spawn(async move {
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match read_half.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Client {} failed to read from socket; err = {:?}", session_id, e);
                        return;
                    }
                };

                println!("Client {} read {} bytes", session_id, n);

                incoming_sender.send(buf).await;
            }
        }));

        let (outgoing_sender, mut outgoing_receiver) = mpsc::channel::<[u8; BUFFER_SIZE]>(32);
        // receive outgoing packets and write into stream
        let out_handle = Option::Some(tokio::spawn(async move {
            while let Some(data) = outgoing_receiver.recv().await {
                let _ = write_half.write(&data);
            }
        }));

        self.inc_handle = inc_handle;
        self.out_handle = out_handle;
        return Ok((outgoing_sender, incoming_receiver));
    }

    pub fn stop(mut self) {
        self.inc_handle.unwrap().abort();
        self.out_handle.unwrap().abort();

        self.inc_handle = Option::None;
        self.out_handle = Option::None;
    }
}

