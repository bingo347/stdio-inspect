mod command;
mod communicator;
mod config;
mod udp;

use config::AppConfig::Run;
use tokio::sync::broadcast::channel;

#[tokio::main]
async fn main() {
    match config::collect_config() {
        Run { command, udp, .. } => {
            let (tx, rx) = channel(32);
            let mut rx = Some(rx);
            macro_rules! get_rx {
                () => {
                    rx.take().unwrap_or_else(|| tx.subscribe())
                };
            }

            if let Some(addr) = udp {
                udp::sender::start_sender(addr, get_rx!()).await;
            }

            let code = command.run(tx).await;
            std::process::exit(code);
        }
        _ => todo!(),
    }
}
