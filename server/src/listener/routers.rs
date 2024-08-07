use std::sync::Arc;

use actix_web::{web, HttpRequest};

use crate::ProfileConfig;

pub async fn agent_register(request: HttpRequest, profile: web::Data<Arc<ProfileConfig>>) -> String {
    let headers = request.headers();
    let user_agent = headers.get("user-agent").unwrap();

    if profile.agents.allowed.iter().any(|v| v == user_agent) {
        "Welcome to Rose".to_string()
    } else {
        "Hello world".to_string()
    }
}

pub async fn get_target() -> String {
    "Hello world".to_string()
}
