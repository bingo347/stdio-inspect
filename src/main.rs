mod config;

fn main() {
    let config = config::collect_config();
    println!("{:?}", config)
}
