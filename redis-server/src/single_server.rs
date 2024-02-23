use std::collections::HashMap;
use crate::data::shared::{process_get, process_set};
use crate::datatypes::StorageRecord;
use crate::{commands::Command, datatypes::DataType};

pub struct Server {
    map: HashMap<String, StorageRecord>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            map: HashMap::new(),
        }
    }

    pub fn process_command(&mut self, command: Command) -> Result<DataType, String> {
        match command {
            Command::Set(command) => process_set(&mut self.map, command),
            Command::Get { key } => process_get(&mut self.map, key),
            Command::ConfigGet { .. } => {
                Ok(DataType::Array(vec![DataType::BulkString("save".into()), DataType::BulkString("3600 1 300 100 60 10000".into())]))
            },
            Command::Dump => {
                println!("{:#?}", self.map);
                Ok(DataType::Nil)
            },
        }
    }
}
