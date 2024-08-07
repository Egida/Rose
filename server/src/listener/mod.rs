use std::sync::Arc;
use tokio::sync::Mutex;

use actix_web::{web, App, HttpServer};

use crate::{Job, ProfileConfig};

mod routers;

pub async fn run(server_addr: &str, profile: Arc<ProfileConfig>, jobs: Arc<Mutex<Vec<Job>>>) {
   println!("Webserver listening: {}", server_addr);

   let jobs_webdata = web::Data::new(jobs);
   let profile_webdata = web::Data::new(profile);

   HttpServer::new(move || {
       App::new()
          .app_data(jobs_webdata.clone())
          .app_data(profile_webdata.clone())
         .route("/reg", web::post().to(routers::agent_register))
         .route("/target", web::get().to(routers::get_target))
   })
   .disable_signals()
   .bind(server_addr)
      .unwrap()
      .run()
      .await
      .unwrap()
}
