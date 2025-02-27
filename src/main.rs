use log::{error, info};

mod executor;
mod net;
mod utils;
mod messages;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .env()
        .with_utc_timestamps()
        .init()
        .unwrap();

    info!("MetalX Agent - Launching");
    let host_id = match utils::get_machine_id() {
        Ok(id) => id,
        Err(err) => {
            error!("Failed to get machine id: {}", err);
            if cfg!(debug_assertions) {
                eprintln!(
                    "[Debug] Failed to get machine id, use 'cafecafecafecafecafecafecafecafe' instead",
                );
                "cafecafecafecafecafecafecafecafe".to_string()
            } else {
                std::process::exit(1);
            }
        }
    };
    let ws_url = std::env::var("WS_URL").unwrap_or("ws://localhost:8080/ws".to_string());
    loop {
        if let Err(err) = net::agent_main(ws_url.clone(), host_id.clone()).await {
            error!("Agent failed: {}", err);
        }
    }
}
