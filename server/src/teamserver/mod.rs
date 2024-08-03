use std::{collections::HashMap, sync::Arc};
use std::net::SocketAddr;
use tokio::sync::Mutex;

use futures::stream::StreamExt;

use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::Attack;

#[derive(Serialize, Deserialize)]
enum Methods {
    AUTH,
    SERVERINFO
}

#[derive(Serialize, Deserialize)]
struct ResponseJson<T> {
    method: Methods,
    parameters: HashMap<String, T>
}

async fn handle_connection(stream: TcpStream, _addr: SocketAddr, _attacks: Arc<Mutex<Vec<Attack>>>) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();

    let (_writer, mut reader) = ws_stream.split();

    while let Some(msg) = reader.next().await {
        let msg = msg.unwrap();

        match msg {
            Message::Text(_data) => {
                let msgjson: ResponseJson<Value>= serde_json::from_str(&_data).unwrap();

                match msgjson.method {
                    Methods::AUTH => (),
                    Methods::SERVERINFO => (),
                }
            },
            Message::Binary(_data) => (),
            Message::Ping(_data) => (),
            Message::Pong(_data) => (),
            Message::Close(_data) => (),
            Message::Frame(_data) => (),
        }
    }
}

pub async fn run(server_addr: &str, attacks: Arc<Mutex<Vec<Attack>>>) {
    let socket = TcpListener::bind(server_addr).await;
    let listener = socket.unwrap();

    println!("Listening on: {}", server_addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let attacks_clone = Arc::clone(&attacks);
        tokio::spawn(handle_connection(stream, client_addr, attacks_clone));
    }
}
