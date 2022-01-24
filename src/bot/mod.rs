use crate::workerpool::WorkerPool;
mod command;

// use mysql::Pool;
// use mysql::*;
use serde_json::Value;
use std::collections::HashMap;
use tungstenite::connect;
use url::Url;

use crate::config::Config;
use command::Command;

type Websocket = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub struct Bot {
    id: String,
    verify_key: String,
    admin_id: String,
    workers: WorkerPool,
    // db_connection_pool: Pool,
    mirai_connection: Websocket,
    commands: HashMap<String, Command>,
}

impl Bot {
    pub fn from_config(cfg: Config) -> Bot {
        let ws = format!(
            "ws://localhost:8080/message?verifyKey={}&qq={}",
            cfg.verify_key, cfg.id
        );
        let (ws, _) = connect(Url::parse(&ws).unwrap()).unwrap();

        let mut commands = HashMap::new();
        for instruction in cfg.instructions {
            commands.insert(
                instruction.command,
                Command::new(
                    instruction.params,
                    instruction.content,
                    instruction.is_public,
                ),
            );
        }

        Bot {
            id: cfg.id,
            verify_key: cfg.verify_key,
            admin_id: cfg.admin_id,
            workers: WorkerPool::new(cfg.worker_amount),
            // db_connection_pool: Pool::new(mysql::Opts::from_url(&cfg.dbUrl).unwrap()).unwrap(),
            mirai_connection: ws,
            commands,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = match self.mirai_connection.read_message() {
                Ok(s) => match s.into_text() {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("message corrupted");
                        continue;
                    }
                },
                Err(_) => {
                    eprintln!("recv error");
                    break;
                }
            };

            // 上报消息采用脏类型解析
            let msg: Value = match serde_json::from_str(&msg) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("parse error");
                    continue;
                }
            };

            if !self.filter(&msg) {
                eprintln!("Unsupported message type");
                continue;
            }

            let (command, id, args) = match self.parse_command(&msg) {
                Ok((c, id, a)) => (c, id, a),
                Err(_) => {
                    eprintln!("Parse Error");
                    continue;
                }
            };

            // 分发消息
            match self.check_permission(command, &id) {
                true => {
                    let db_operation_result = command.get(args);
                    self.workers.execute(move || {
                        eprintln!("RECV A NEW REQUEST");
                        let tmp;
                        match db_operation_result {
                            Ok(op) => {
                                tmp = format!("{}: {}",id,op);
                                send_message("932942142", &tmp);
                            }
                            Err(_) => {
                                send_message(&id, "WRONG FORMAT");
                            }
                        }
                        eprintln!("Reply Sent");
                    });
                }
                false => {
                    self.workers.execute(move || {
                        send_message(&id, "NO PERMISSION");
                    });
                }
            }
        }
    }

    fn filter(&self, msg: &serde_json::Value) -> bool {
        //!过滤器
        //!过滤不需要的信息

        if msg["data"]["type"] == "FriendMessage"
            || msg["data"]["type"] == "StrangerMessage"
            || msg["data"]["type"] == "TempMessage"
        {
            true
        } else {
            false
        }
    }

    fn check_permission(&self, command: &Command, id: &str) -> bool {
        if command.is_public() && id != self.admin_id {
            false
        }
        else {
            true
        }
    }

    fn parse_command(
        &self,
        msg: &serde_json::Value,
    ) -> std::result::Result<(&Command, String, Vec<String>), ()> {
        let content: String = match msg["data"]["messageChain"].get(1) {
            Some(s) => s.to_string(),
            None => return Err(()),
        };
        let mut content = content.split(" ");
        let first = match content.next() {
            Some(s) => s,
            None => return Err(()),
        };

        match self.commands.get(first) {
            Some(c) => {
                let mut args = Vec::new();
                for s in content {
                    args.push(s.to_string());
                }
                Ok((c, msg["data"]["Sender"]["id"].to_string(), args))
            }
            None => Err(()),
        }
    }
}

fn send_message(dst_id: &str, msg: &str) {
    let client = reqwest::blocking::Client::new();
    let body = format!(
        r#"{{ "sessionKey":"", "target":{}, "messageChain":[ {{ "type":"Plain", "text":"{}"}} ] }}"#,
        dst_id, msg
    );

    match client
        .post("http://127.0.0.1:8080/sendFriendMessage")
        .body(body)
        .send()
    {
        Ok(_) => return (),
        Err(_) => eprintln!("Response lost"),
    }
}
