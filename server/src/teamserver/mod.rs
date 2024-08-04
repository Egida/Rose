use std::ops::Deref;
use std::{collections::HashMap, sync::Arc};
use std::net::SocketAddr;

use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

use futures::stream::StreamExt;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::{Attack, ProfileConfig};

#[derive(Debug)]
enum ClientError {
    Invalidusername,
    InvalidPassword,
}

#[derive(Serialize, Deserialize)]
enum Methods {
    Auth,
    ServerInfo
}

#[derive(Serialize, Deserialize)]
struct ResponseJson<T> {
    method: Methods,
    parameters: HashMap<String, T>
}

fn auth(profile: Arc<ProfileConfig>, username: String, password: String) -> Result<(), ClientError> {
    let username_config = profile.deref()
        .users[username]
        .clone();

    if username_config.is_null() {
        return Err(ClientError::Invalidusername) 
    }

    if username_config.to_string().eq(&password) {
        Ok(())
    } else {
        Err(ClientError::InvalidPassword)
    }
}

async fn handle_connection(
        stream: TcpStream, 
        addr: SocketAddr,
        clients_autheticated: Arc<Mutex<Vec<SocketAddr>>>,
        profile: Arc<ProfileConfig>,
        _attacks: Arc<Mutex<Vec<Attack>>>
    ) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();

    let (_writer, mut reader) = ws_stream.split();

    while let Some(msg) = reader.next().await {
        let profile_clone = Arc::clone(&profile);
        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                let msgjson: ResponseJson<Value>= serde_json::from_str(&data).unwrap();

                match msgjson.method {
                    Methods::Auth => {
                        let username = msgjson.parameters.get("username").unwrap().to_string();
                        let password = msgjson.parameters.get("password").unwrap().to_string();

                        match auth(profile_clone, username, password) {
                            Ok(_) => clients_autheticated.lock().await.push(addr),
                            Err(e) => eprintln!("ERROR: {:?}", e)
                        }

                    },
                    Methods::ServerInfo => (),
                }
            },
            Message::Binary(_data) => (),
            Message::Ping(_data) => (),
            Message::Pong(_data) => (),
            Message::Frame(_data) => (),
            Message::Close(_data) => (),
        }
    }
}

pub async fn run(server_addr: &str, profile: ProfileConfig, attacks: Arc<Mutex<Vec<Attack>>>) {
    let clients_autheticated: Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(Vec::new()));
    let socket = TcpListener::bind(server_addr).await;
    let listener = socket.unwrap();

    println!("Teamserver Listening on: {}", server_addr);

    let profile_arc = Arc::new(profile);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let attacks_clone = Arc::clone(&attacks);
        let profile_arc_clone = Arc::clone(&profile_arc);
        let clients_autheticated_clone = Arc::clone(&clients_autheticated);

        tokio::spawn(handle_connection(
                stream, 
                client_addr,
                clients_autheticated_clone,
                profile_arc_clone,
                attacks_clone)
            );
    }
}
