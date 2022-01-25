use crate::workerpool::WorkerPool;
mod command;

use mysql::Pool;
use mysql::*;
use mysql::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tungstenite::connect;
use url::Url;

use crate::config::Config;
use command::Command;

type Websocket = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub struct Bot {
    // id: String,
    // verify_key: String,
    admin_id: String,
    workers: WorkerPool,
    db_connection_pool: Pool,
    mirai_connection: Websocket,
    commands: HashMap<String, Command>,
    help_info: String,
}

enum CommandErr {
    ParseError,
    Help(String),
    CommandNotFound(String),
}

impl Bot {
    pub fn from_config(cfg: Config) -> Bot {
        let ws = format!(
            "ws://localhost:8080/message?verifyKey={}&qq={}",
            cfg.verify_key, cfg.id
        );
        let (ws, _) = connect(Url::parse(&ws).unwrap()).unwrap();

        let mut commands = HashMap::new();
        let mut help_info = String::new();
        for instruction in cfg.instructions {
            commands.insert(
                instruction.command.clone(),
                Command::new(
                    instruction.params.clone(),
                    instruction.content.clone(),
                    instruction.is_public,
                ),
            );
            help_info.push_str(&format!("{}",instruction.command));
            for p in instruction.params {
                help_info.push_str(&format!(" {{{}}}",p));
            }
            help_info.push_str(&format!(": {}\n",instruction.description));
        }

        Bot {
            // id: cfg.id,
            // verify_key: cfg.verify_key,
            admin_id: cfg.admin_id,
            workers: WorkerPool::new(cfg.worker_amount),
            db_connection_pool: Pool::new(mysql::Opts::from_url(&cfg.db_url).unwrap()).unwrap(),
            mirai_connection: ws,
            commands,
            help_info,
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
                    eprintln!("Parse error");
                    continue;
                }
            };

            if !self.filter(&msg) {
                eprintln!("Unsupported message type");
                continue;
            }

            let (command, id, args) = match self.parse_command(&msg) {
                Ok((c, id, a)) => (c, id, a),
                Err(ce) => {
                    match ce {
                        CommandErr::CommandNotFound(id) => send_message(&id, "WRONG OPERATION"),
                        CommandErr::ParseError => eprintln!("Parse error"),
                        CommandErr::Help(id) => {
                            let help = self.help_info.clone();
                            self.workers.execute(move || {
                                send_message(&id, &help);
                            });
                        }
                    };
                    continue;
                }
            };

            // 分发消息
            let pool = self.db_connection_pool.clone();
            match self.check_permission(command, &id) {
                true => {
                    let db_operation_result = command.get(args, id.clone());
                    self.workers.execute(move || {
                        eprintln!("RECV A NEW REQUEST");
                        match db_operation_result {
                            Ok(op) => {
                                eprintln!("PROCESS {}: {}",id,op);
                                match process_db_query(pool, &op) {
                                    Ok(_) => send_message(&id, "OK"),
                                    Err(DbError::ConnectError) => send_message(&id, "DB CONNECTION FAIL"),
                                    Err(DbError::ProcessError) => send_message(&id, "QUERY PROCESS FAIL"),
                                }
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
        } else {
            true
        }
    }

    fn parse_command(
        &self,
        msg: &serde_json::Value,
    ) -> std::result::Result<(&Command, String, Vec<String>), CommandErr> {
        let content: &Value = match msg["data"]["messageChain"].get(1) {
            Some(v) => v,
            None => return Err(CommandErr::ParseError),
        };

        let content: String = match content["text"].as_str() {
            Some(s) => s.to_string(),
            None => "".to_string(),
        };

        let mut content = content.split(" ");
        let id = msg["data"]["sender"]["id"]
            .to_string()
            .trim_matches('"')
            .to_string();

        let first = match content.next() {
            Some(s) => s.to_string(),
            None => return Err(CommandErr::ParseError),
        };

        if first == "help" {
            return Err(CommandErr::Help(id));
        }


        match self.commands.get(&first) {
            Some(c) => {
                let mut args = Vec::new();
                for s in content {
                    args.push(s.to_string());
                }
                // args.push(id.clone());
                Ok((c, id, args))
            }
            None => Err(CommandErr::CommandNotFound(id)),
        }
    }
}

fn send_message(dst_id: &str, msg: &str) {
    let client = reqwest::blocking::Client::new();
    // let body = format!(
    // r#"{{ "sessionKey":"", "target":{}, "messageChain":[ {{ "type":"Plain", "text":"{}"}} ] }}"#,
    // dst_id, msg
    // );
    let body = json!({
        "sessionKey": "",
        "target": dst_id,
        "messageChain": [
            {
                "type": "Plain",
                "text": msg
            }
        ]
    });
    eprintln!("SEND: {}", body.to_string());

    match client
        .post("http://127.0.0.1:8080/sendFriendMessage")
        .body(body.to_string())
        .send()
    {
        Ok(_) => return (),
        Err(_) => eprintln!("Response lost"),
    }
}

enum DbError {
    ConnectError,
    ProcessError,
}

fn process_db_query(pool: mysql::Pool,op: &str) -> std::result::Result<(),DbError> {
    let mut conn = match pool.get_conn() {
        Ok(c) => c,
        Err(_) => return Err(DbError::ConnectError),
    };

    match conn.exec_drop(op,()) {
        Ok(_) => Ok(()),
        Err(_) => Err(DbError::ProcessError),
    }
}
