use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use tokio::sync::broadcast;
use actix_files as fs;

type Message = tokio_tungstenite::tungstenite::Message;


use actix_web::{App, HttpServer, HttpResponse, web};


mod pixels;
use pixels::*;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page_task = tokio::spawn(async {
        let page_port = 8889;
        let folder_for_page = "frontend";
        HttpServer::new(move || {
                App::new()
                    .service(fs::Files::new("/", folder_for_page).index_file("index.html"))
                    .default_service(web::to(|| HttpResponse::NotFound())) 
            })
            .bind(("0.0.0.0", page_port)).expect("Bind error")
            .run()
            .await.unwrap();});
    
    let address = "0.0.0.0:8888";
    let (tx, _) = broadcast::channel::<Pixel>(64);
    let tx = Arc::new(tx);
    let listener = TcpListener::bind(address).await?;
    println!("WebSocket started on {}", address);

    let pixels = Arc::new(Mutex::new(ChunkOfPixels::new()));
    let rx = tx.subscribe();

    let pixels_clone = Arc::clone(&pixels);

    let save_pixels_task = tokio::spawn(async move{
        save_pixels_process(rx, pixels_clone).await
    });
    
    while let Ok((stream, _)) = listener.accept().await {
        let tx_clone = Arc::clone(&tx);
        let rx = tx.subscribe();
        
        let pixels_clone = Arc::clone(&pixels);
        tokio::spawn(async move {
            if let Err(e) = handle_ws_connection(stream, tx_clone, rx, pixels_clone).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
    
    
    tokio::select! {
        _ = save_pixels_task => {}
        _ = page_task => {}
    }
    Ok(())
}

async fn save_pixels_process(mut rx: broadcast::Receiver<Pixel>, pixels: Arc<Mutex<ChunkOfPixels>>){
    while let Ok(msg) = rx.recv().await {
        let mut pixels_data = pixels.lock().await;
        pixels_data.add(msg);
        drop(pixels_data);
    }
}


async fn handle_ws_connection(
    raw_stream: TcpStream,
    tx: Arc<broadcast::Sender<Pixel>>,
    rx: broadcast::Receiver<Pixel>,
    pixels: Arc<Mutex<ChunkOfPixels>>
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = raw_stream.peer_addr()?;

    let ws_stream = accept_async(raw_stream).await?;
    let (mut write, read) = ws_stream.split();

    println!("New connection: {}", addr);
    let pixels_data = pixels.lock().await;
    println!("{}", pixels_data.to_json());
    write.send(Message::Text(pixels_data.to_json())).await.unwrap();
    
    drop(pixels_data);

    let tx_send = tx.clone();
    let send_task = tokio::spawn(async move {
        resending_processing(addr, tx_send, read).await;
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

async fn resending_processing(
    addr: SocketAddr,
    tx_send: Arc<broadcast::Sender<Pixel>>,
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
        if let Err(e) = tx_send.send(Pixel::from_json(&msg).unwrap()) {
            eprintln!("Sending error: {}", e);
        }
    }
}

async fn process_inbox_data(
    mut rx: broadcast::Receiver<Pixel>,
    mut write: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>){
    while let Ok(msg) = rx.recv().await {
        if let Err(_e) = write.send(Message::Text(msg.to_json())).await
        {
            eprintln!("Sending to client error");
            break;
        }
    }
}

