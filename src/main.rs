use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use tokio::sync::broadcast;

type Message = tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "127.0.0.1:8888";
    let (tx, _) = broadcast::channel::<String>(64);
    let tx = Arc::new(tx);

    let listener = TcpListener::bind(address).await?;
    println!("WebSocket started on {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let tx_clone = Arc::clone(&tx);
        let rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, tx_clone, rx).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    raw_stream: TcpStream,
    tx: Arc<broadcast::Sender<String>>,
    rx: broadcast::Receiver<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = raw_stream.peer_addr()?;
    println!("New connection: {}", addr);

    let ws_stream = accept_async(raw_stream).await?;
    let (write, read) = ws_stream.split();

    let tx_send = tx.clone();
    let send_task = tokio::spawn(async move {
        sending_processing(addr, tx_send, read).await;
    });

    let recv_task = tokio::spawn(async move {
        process_inbox_data(rx, write).await;
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    Ok(())
}

async fn sending_processing(
    addr: SocketAddr,
    tx_send: Arc<broadcast::Sender<String>>,
    mut read: SplitStream<WebSocketStream<tokio::net::TcpStream>>){
    while let Some(result) = read.next().await {
        let msg = match result {
            Ok(msg) if msg.is_text() => msg.to_text().unwrap_or_default().to_string(),
            Ok(msg) if msg.is_close() => {
                println!("Client {} disconnected", addr);
                return;
            }
            Ok(_) => continue,
            Err(e) => {
                eprintln!("Reading error from {}: {}", addr, e);
                return;
            }
        };

        if let Err(e) = tx_send.send(msg) {
            eprintln!("Sending error: {}", e);
        }
    }
}

async fn process_inbox_data(
    mut rx: broadcast::Receiver<String>,
    mut write: SplitSink<WebSocketStream<tokio::net::TcpStream>, tokio_tungstenite::tungstenite::Message>){
    while let Ok(msg) = rx.recv().await {
        if let Err(_e) = write.send(Message::Text(msg)).await
        {
            eprintln!("Sending to client error");
            break;
        }
    }
}

