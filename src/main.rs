mod command;
mod config;

use config::AppConfig::Run;

#[tokio::main]
async fn main() {
    match config::collect_config() {
        Run { command, .. } => {
            let code = command.run().await;
            std::process::exit(code);
        }
        _ => todo!(),
    }
}
