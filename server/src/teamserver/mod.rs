use std::ops::Deref;
use std::{collections::HashMap, sync::Arc};
use std::net::SocketAddr;

use futures::SinkExt;
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

use futures::stream::StreamExt;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::{Attack, ProfileConfig};

#[derive(Debug, Serialize)]
enum ClientError {
    InvalidJsonFormat,
    InvalidJsonData,
    Invalidusername,
    InvalidPassword,
}

#[derive(Serialize, Deserialize)]
enum Methods {
    Auth,
    ServerInfo
}

#[derive(Serialize, Deserialize)]
struct RequestJson<T> {
    method: Methods,
    parameters: HashMap<String, T>
}

#[derive(Serialize)]
struct ErrorResponseJson {
    error: ClientError,
    message: String
}

#[derive(Serialize)]
struct ResponseJson<T> {
    data: T
}

fn auth(profile: Arc<ProfileConfig>, username: String, password: String) -> Result<(), ClientError> {
    let username_config = profile.deref()
        .users[username]
        .clone();

    if username_config.is_null() {
        return Err(ClientError::Invalidusername) 
    }

    if username_config.as_str().unwrap().eq(&password) {
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
    println!("New connection: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();

    let (mut writer, mut reader) = ws_stream.split();

    while let Some(msg) = reader.next().await {
        let profile_clone = Arc::clone(&profile);
        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                let msgjson: RequestJson<Value> = match serde_json::from_str(&data) {
                    Ok(r) => r,
                    Err(_e) => {
                        let error  = serde_json::to_string(
                            &ErrorResponseJson {error: ClientError::InvalidJsonFormat, message: "Invalid JSON format".to_string()}
                        ).unwrap();

                        writer.send(Message::Text(error)).await.unwrap();
                        break
                    }
                };

                match msgjson.method {
                    Methods::Auth => {
                        let username = match msgjson.parameters.get("username") {
                            Some(r) => r.as_str().unwrap().to_string(),
                            None => {
                                let error = serde_json::to_string(
                                    &ErrorResponseJson {error: ClientError::InvalidJsonData, message: "Invalid JSON data".to_string()}
                                ).unwrap();

                                writer.send(Message::Text(error)).await.unwrap();
                                break;
                            },
                        };

                        let password = match msgjson.parameters.get("password") {
                            Some(r) => r.as_str().unwrap().to_string(),
                            None => {
                                let error = serde_json::to_string(
                                    &ErrorResponseJson {error: ClientError::InvalidJsonData, message: "Invalid JSON data".to_string()}
                                ).unwrap();

                                writer.send(Message::Text(error)).await.unwrap();
                                break;
                            }
                        };

                        match auth(profile_clone, username, password) {
                            Ok(_) => clients_autheticated.lock().await.push(addr),
                            Err(e) => {
                                let message = match e {
                                    ClientError::Invalidusername => "Invalid username".to_string(),
                                    ClientError::InvalidPassword => "Invalid password".to_string(),
                                    _ => panic!("Case not covered")
                                };

                                let error = serde_json::to_string(
                                    &ErrorResponseJson {error: e, message: message.to_string()}
                                );

                                writer.send(Message::Text(error.unwrap())).await.unwrap();
                                break;
                            }
                        }

                        writer.send(Message::Text(
                                serde_json::to_string(&ResponseJson {data: "ok".to_string()}).unwrap())
                            ).await.unwrap();
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

    println!("Teamserver listening on: {}", server_addr);

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
