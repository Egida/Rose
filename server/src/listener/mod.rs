use std::sync::Arc;
use tokio::sync::Mutex;

use actix_web::{web, App, HttpServer};

use crate::Attack;

mod routers;

pub async fn run(server_addr: &str, attacks: Arc<Mutex<Vec<Attack>>>) {
   println!("Webserver listening: {}", server_addr);

   let attacks_webdata = web::Data::new(attacks);

   HttpServer::new(move || {
       App::new()
          .app_data(attacks_webdata.clone())
         .route("/reg", web::get().to(routers::agent_register))
         .route("/target", web::get().to(routers::get_target))
   })
   .disable_signals()
   .bind(server_addr)
      .unwrap()
      .run()
      .await
      .unwrap()
}
