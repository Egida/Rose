mod listener;
mod teamserver;

const TEAMSERVER_ADDR: &str = "127.0.0.1:5555";
const LISTENER_ADDR: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let teamserver_task = tokio::task::spawn( async { teamserver::run(TEAMSERVER_ADDR).await } );
    let listener_task = tokio::task::spawn( async { listener::run(LISTENER_ADDR).await } );

    // Joining tasks

    teamserver_task.await.unwrap();
    listener_task.await.unwrap();

    Ok(())
}
