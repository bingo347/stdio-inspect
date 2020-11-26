mod config;
mod command;

fn main() {
    let config = config::collect_config();
    println!("{:?}", config)
}
