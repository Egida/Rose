use std::sync::Arc;
use tokio::sync::Mutex;

use actix_web::{web, App, HttpServer};

use crate::Job;

mod routers;

pub async fn run(server_addr: &str, jobs: Arc<Mutex<Vec<Job>>>) {
   println!("Webserver listening: {}", server_addr);

   let jobs_webdata = web::Data::new(jobs);

   HttpServer::new(move || {
       App::new()
          .app_data(jobs_webdata.clone())
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
