use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction {
    pub command: String,
    pub is_public: bool,
    pub params: Vec<String>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub id: String,
    pub verify_key: String,
    pub db_url: String,
    pub worker_amount: usize,
    pub instructions: Vec<Instruction>,
}

impl Config {
    pub fn from(path: &str) -> Config {
        let content = fs::read_to_string(path).unwrap();
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Config {
        serde_json::from_str(content).unwrap()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let res = crate::config::Config::from("config.json");
        println!("{:#?}",res);
    }
}