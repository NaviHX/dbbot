extern crate strfmt;
use strfmt::strfmt;

use std::collections::HashMap;

pub struct Command {
    params: Vec<String>,
    content: String,
}

impl Command {
    pub fn new(params: &Vec<String>, content: &String) -> Command {
        Command {
            params: params.clone(),
            content: content.clone(),
        }
    }

    pub fn get(&self, args: Vec<String>) -> Result<String,()> {
        if self.params.len() != args.len() {
            return Err(());
        }

        let mut arg_map = HashMap::new();

        for (i, arg) in args.iter().enumerate() {
            arg_map.insert(self.params[i].clone(), arg);
        }

        match strfmt(&self.content, &arg_map) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        let params = vec!["test1".to_string(),"test2".to_string()];
        let content = "SELECT {test1}, {test2} FROM testtable".to_string();

        let command = super::Command::new(&params, &content);

        match command.get(vec!["age".to_string(),"sex".to_string()]) {
            Ok(s) => println!("COMMAND: {}",s),
            Err(_) => panic!(),
        }
    }
}