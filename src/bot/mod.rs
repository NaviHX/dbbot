use crate::workerpool::WorkerPool;
mod command;

// use mysql::Pool;
use mysql::*;
use mysql::prelude::*;
use tungstenite::connect;
use url::Url;
use serde_json::Value;

use crate::config::Config;

type Websocket = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub struct Bot {
    id: String,
    verify_key: String,
    workers: WorkerPool,
    // db_connection_pool: Pool,
    mirai_connection: Websocket,
}

impl Bot {
    pub fn from_config(cfg: Config) -> Bot {
        let ws = format!("ws://localhost:8080/all?verifyKey={}&qq={}",cfg.verify_key,cfg.id);
        let (ws, _) = connect(Url::parse(&ws).unwrap()).unwrap();
        
        Bot {
            id: cfg.id,
            verify_key: cfg.verify_key,
            workers: WorkerPool::new(cfg.worker_amount),
            // db_connection_pool: Pool::new(mysql::Opts::from_url(&cfg.dbUrl).unwrap()).unwrap(),
            mirai_connection: ws,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = match self.mirai_connection.read_message() {
                Ok(s) => match s.into_text() {
                    Ok(s) => s,
                    Err(_) => { eprintln!("message corrupted"); continue },
                }
                Err(_) => { eprintln!("recv error"); break },
            };

            self.workers.execute(move || {
                println!("RECV: {}",msg);
            });

            // let msg: Value = match serde_json::from_str(&msg) {
                // Ok(v) => v,
                // Err(_) => { eprintln!("parse error"); continue },
            // };
        }    
    }
}