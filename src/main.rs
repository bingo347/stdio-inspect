mod command;
mod config;

use config::AppConfig::Run;

fn main() {
    match config::collect_config() {
        Run { command, .. } => {
            command.run();
        }
        _ => todo!(),
    }
}
