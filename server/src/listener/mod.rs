use std::sync::Arc;

use axum::{routing::{get, post}, Router};

use crate::MAShared;

mod routers;

pub async fn run(server_addr: &str, shared: Arc<MAShared>) {
   println!("Webserver listening: {}", server_addr);

   let app = Router::new()
      .with_state(shared)
      .route("/reg", post(routers::agent_register))
      .route("/target", get(routers::get_target))
      .route("/ping", get(routers::ping));
}
