use crate::workerpool::WorkerPool;
mod command;

// use mysql::Pool;
use mysql::*;
use mysql::prelude::*;
use tungstenite::connect;
use url::Url;
use serde_json::Value;
use std::collections::HashMap;

use crate::config::Config;
use command::Command;

// struct Response {};

type Websocket = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub struct Bot {
    id: String,
    verify_key: String,
    admin_id: String,
    workers: WorkerPool,
    // db_connection_pool: Pool,
    mirai_connection: Websocket,
    commands: HashMap<String,Command>,
}

impl Bot {
    pub fn from_config(cfg: Config) -> Bot {
        let ws = format!("ws://localhost:8080/all?verifyKey={}&qq={}",cfg.verify_key,cfg.id);
        let (ws, _) = connect(Url::parse(&ws).unwrap()).unwrap();

        let mut commands = HashMap::new();
        for instruction in cfg.instructions {
            commands.insert(instruction.command, Command::new(instruction.params, instruction.content, instruction.is_public));
        }

        
        Bot {
            id: cfg.id,
            verify_key: cfg.verify_key,
            admin_id: cfg.admin_id,
            workers: WorkerPool::new(cfg.worker_amount),
            // db_connection_pool: Pool::new(mysql::Opts::from_url(&cfg.dbUrl).unwrap()).unwrap(),
            mirai_connection: ws,
            commands
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

            // 上报消息采用脏类型解析
            let msg: Value = match serde_json::from_str(&msg) {
                Ok(v) => v,
                Err(_) => { eprintln!("parse error"); continue },
            };

            if !self.filter(&msg) {
                eprintln!("Unsupported message type");
                continue;
            }

            // 分发消息
            self.workers.execute(move || {
                println!("RECV A NEW MESSAGE");
            });
        }    
    }

    pub fn filter(&self, msg: &serde_json::Value) -> bool {
        //!过滤器
        //!过滤不需要的信息

        if msg["data"]["type"] == "FriendMessage" || msg["data"]["type"] == "StrangerMessage" || msg["data"]["type"] == "TempMessage" {
            true
        }
        else {
            false
        }
    }
}