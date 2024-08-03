use actix_web::{web, App, HttpServer};

async fn hello() -> String {
    "Hello world".to_string()
}

pub async fn run(server_addr: &str) {
   println!("Webserver listening: {}", server_addr);

   HttpServer::new(|| {
       App::new()
           .route("/", web::get().to(hello))
   })
   .disable_signals()
   .bind(server_addr)
      .unwrap()
      .run()
      .await
      .unwrap()
}
