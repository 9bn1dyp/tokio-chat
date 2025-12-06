use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::broadcast::{self, Receiver, Sender};

async fn client_read_loop(
    mut rx: Receiver<String>,
    mut socket_write: OwnedWriteHalf,
) -> anyhow::Result<()> {
    loop {
        let mut bytes = rx.recv().await?;
        bytes.push_str("\r\n");

        if let Err(e) = socket_write.write_all(bytes.as_bytes()).await {
            eprintln!("Failed reading chat {e}")
        }
    }
}

async fn client_write_loop(
    tx: Sender<String>,
    mut socket_read: OwnedReadHalf,
) -> anyhow::Result<()> {
    loop {
        let mut buf = [0u8; 1024];
        let n = socket_read.read(&mut buf).await?;

        let msg = String::from_utf8_lossy(&buf[..n]).to_string();

        if let Err(e) = tx.send(msg) {
            eprintln!("Failed sending message {e}")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    // broadcast
    let (tx, _) = broadcast::channel::<String>(16);
    loop {
        // accept and split
        let (socket, _) = listener.accept().await?;
        let split = socket.into_split();

        // clone sender
        let tx_clone = tx.clone();
        // create receiver from tx
        let rx = tx.subscribe();

        // read is reading chat (broadcast output), requires write socket
        // needs a receiver
        tokio::spawn(client_read_loop(rx, split.1));

        // write is writing to receivers (broadcast input), requires read socket
        tokio::spawn(client_write_loop(tx_clone, split.0));
    }
}
