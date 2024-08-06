use std::hash::RandomState;
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
    InvalidJsonDatatype,
    NotFoundJsonParameter,
    Invalidusername,
    InvalidPassword,
}

#[derive(Serialize, Deserialize)]
enum Method {
    Auth,
    Jobs
}

#[derive(Serialize, Deserialize)]
struct RequestJson<T> {
    method: Method,
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
        profile: Arc<ProfileConfig>,
        _attacks: Arc<Mutex<Vec<Attack>>>
    ) {
    println!("New connection: {}", addr);
    let mut authed = false;
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();

    let (mut writer, mut reader) = ws_stream.split();

    while let Some(msg) = reader.next().await {
        let profile_clone = Arc::clone(&profile);
        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                /*
                JSON Structure
                    {
                        method: Method
                        parameters: Vec<String>
                    }
                */

                fn find_parameter(parameters: HashMap<String, Value, RandomState>, parameter_name: &str) -> Result<String, ErrorResponseJson> { 
                    match parameters.get(parameter_name) {
                        Some(r) => {
                            if let Some(s) = r.as_str() {
                                Ok(s.to_string())
                            } else {
                                Err(ErrorResponseJson { 
                                    error: ClientError::InvalidJsonDatatype, 
                                    message: "Invalid JSON datatype".to_string() 
                                })
                            }
                        },
                        None => {
                            Err(ErrorResponseJson {error: ClientError::NotFoundJsonParameter, message: format!("'{}' paramenter not found", parameter_name)})
                        }
                    }
                }

                let msgjson: RequestJson<Value> = match serde_json::from_str(&data) {
                    Ok(r) => r,
                    Err(e) => {
                        let error  = serde_json::to_string(
                            &ErrorResponseJson {error: ClientError::InvalidJsonFormat, message: e.to_string()}
                        ).unwrap();

                        writer.send(Message::Text(error)).await.unwrap();
                        break
                    }
                };

                match msgjson.method {
                    Method::Auth => {
                        let username = match find_parameter(msgjson.parameters.to_owned(), "username") {
                            Ok(r) => r,
                            Err(e) => {
                                writer.send(Message::Text(serde_json::to_string(&e).unwrap())).await.unwrap();
                                break;
                            }
                        };
                        let password = match find_parameter(msgjson.parameters, "password") {
                            Ok(r) => r,
                            Err(e) => {
                                writer.send(Message::Text(serde_json::to_string(&e).unwrap())).await.unwrap();
                                break;
                            }
                        };

                        match auth(profile_clone, username, password) {
                            Ok(_) => authed = true,
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
                    Method::Jobs => {
                        if !authed {
                            ()
                        }
                    },
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
    let socket = TcpListener::bind(server_addr).await;
    let listener = socket.unwrap();

    println!("Teamserver listening on: {}", server_addr);

    let profile_arc = Arc::new(profile);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let attacks_clone = Arc::clone(&attacks);
        let profile_arc_clone = Arc::clone(&profile_arc);

        tokio::spawn(handle_connection(
                stream, 
                client_addr,
                profile_arc_clone,
                attacks_clone)
            );
    }
}
