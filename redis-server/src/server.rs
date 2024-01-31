use crate::{commands::Command, datatypes::DataType};
use crate::data::engine::InMemoryEngine;

pub struct Server {
    engine: InMemoryEngine,
}

impl Server {
    pub fn new() -> Server {
        Server {
            engine: InMemoryEngine::new(),
        }
    }

    pub fn process_command(&self, command: Command) -> Result<DataType, String> {
        match command {
            Command::Set(command) => self.engine.process_set(command),
            Command::Get { key } => self.engine.process_get(key),
            Command::ConfigGet { .. } => {
                Ok(DataType::Array(vec![DataType::BulkString("save".into()), DataType::BulkString("3600 1 300 100 60 10000".into())]))
            },
            Command::Dump => self.engine.process_dump(),
        }
    }
}
