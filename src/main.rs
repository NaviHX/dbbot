mod config;
mod bot;
mod workerpool;

use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Config Path Needed");
        return ();
    }

    let cfg = config::Config::from(&args[1]);
    
    let mut bt = bot::Bot::from_config(cfg);
    bt.start();
}
