mod command;
mod communicator;
mod config;

use config::AppConfig::Run;
use tokio::sync::broadcast::channel;

#[tokio::main]
async fn main() {
    match config::collect_config() {
        Run { command, .. } => {
            let (tx, _rx) = channel(32);
            let code = command.run(tx).await;
            std::process::exit(code);
        }
        _ => todo!(),
    }
}
