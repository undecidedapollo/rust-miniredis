use crate::{commands::Command, datatypes::DataType};
use dashmap::DashMap;

pub struct Server {
    keymap: DashMap<String, String>,
}

fn process_set(server: &Server, key: String, value: String) -> Result<DataType, String>{
    server.keymap.insert(key, value);
    Ok(DataType::SimpleString("OK".into()))
}

fn process_get(server: &Server, key: String) -> Result<DataType, String>{
    let val = server.keymap.get(&key);
    match val {
        Some(value) => Ok(DataType::BulkString(value.to_string())),
        None => Ok(DataType::Nil),
    }
}

impl Server {
    pub fn new() -> Server {
        Server {
            keymap: DashMap::new(),
        }
    }

    pub fn process_command(&self, command: Command) -> Result<DataType, String> {
        match command {
            Command::Set { key, value } => process_set(&self, key, value),
            Command::Get { key } => process_get(&self, key),
        }
    }
}
