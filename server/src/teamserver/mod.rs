use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();
}

pub async fn run(server_addr: &str) {
    let socket = TcpListener::bind(server_addr).await;
    let listener = socket.unwrap();

    println!("Listening on: {}", server_addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, client_addr));
    }
}
