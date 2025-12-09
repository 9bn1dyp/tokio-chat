use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

// Req for serde
#[derive(Deserialize, Serialize)]
struct User(String);

// To pass into broadcast we need the Clone trait
impl Clone for User {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Req for serde
#[derive(Deserialize, Serialize)]
struct Message {
    username: User,
    message: String,
}

// To pass into broadcast we need the Clone trait
impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            username: self.username.clone(),
            message: self.message.clone(),
        }
    }
}

async fn client_read_loop(
    mut rx: Receiver<Message>,
    mut writer: FramedWrite<OwnedWriteHalf, LinesCodec>,
) -> anyhow::Result<()> {
    loop {
        // get Message type from broadcast
        let message_struct = rx.recv().await?;

        // convert to json
        let message_struct = serde_json::to_string(&message_struct)?;

        if let Err(e) = writer.send(message_struct).await {
            eprintln!("Failed reading chat {e}")
        }
    }
}

async fn client_write_loop(
    tx: Sender<Message>,
    mut reader: FramedRead<OwnedReadHalf, LinesCodec>,
) -> anyhow::Result<()> {
    // iterate over FrameRead as it impl Stream
    while let Some(Ok(msg)) = reader.next().await {
        let message_struct: Message = serde_json::from_str(&msg)?;
        if let Err(e) = tx.send(message_struct) {
            eprintln!("Failed sending message {e}");
        }
    }
    Ok(())
}

async fn run(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    // broadcast
    let (tx, _) = broadcast::channel::<Message>(64);
    loop {
        // accept and split
        let (socket, _) = listener.accept().await?;
        let (read_half, write_half) = socket.into_split();

        //frames use \n delimeter from LinesCodec, (cant use pretty print json)
        let reader = FramedRead::new(read_half, LinesCodec::new());
        let writer = FramedWrite::new(write_half, LinesCodec::new());

        // clone sender
        let tx_clone = tx.clone();
        // create receiver from tx
        let rx = tx.subscribe();

        // read is reading chat (broadcast output), requires write socket
        // needs a receiver
        tokio::spawn(client_read_loop(rx, writer));

        // write is writing to receivers (broadcast input), requires read socket
        tokio::spawn(client_write_loop(tx_clone, reader));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run("127.0.0.1:8080").await
}
