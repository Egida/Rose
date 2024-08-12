use std::{net::SocketAddr, sync::Arc};

use axum::{routing::{get, post}, Router};
use tokio::net::TcpListener;

use crate::MAShared;

mod routers;

pub async fn run(server_addr: &str, shared: Arc<MAShared>) {
   println!("Webserver listening: {}", server_addr);

   let app = Router::new()
      .route("/reg", post(routers::agent_register))
      .route("/target", get(routers::get_target))
      .with_state(shared);

   let listener = TcpListener::bind(server_addr).await.unwrap();
   axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
