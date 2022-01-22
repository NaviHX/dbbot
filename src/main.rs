mod config;
mod bot;
mod workerpool;

fn main() {
    println!("Hello, world!");

    // let cfg = config::Config::from();
    let cfg = config::Config::from("config.json");
    
    let mut bt = bot::Bot::from_config(cfg);
    bt.start();
}
