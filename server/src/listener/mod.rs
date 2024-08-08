use std::sync::Arc;

use actix_web::{web, App, HttpServer};

use crate::MAShared;

mod routers;

pub async fn run(server_addr: &str, shared: Arc<MAShared>) {
   println!("Webserver listening: {}", server_addr);

   let shared_webdata = web::Data::new(shared);

   HttpServer::new(move || {
       App::new()
          .app_data(shared_webdata.clone())
         .route("/reg", web::post().to(routers::agent_register))
         .route("/target", web::get().to(routers::get_target))
   })
   .disable_signals()
   .bind(server_addr)
      .unwrap()
      .run()
      .await
      .unwrap();
}
