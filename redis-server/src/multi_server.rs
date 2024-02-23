use crate::data::memory_engine::InMemoryEngine;
use crate::data::thread_engine::ThreadEngineManager;
use crate::data::typesd::StorageEngine;
use crate::{commands::Command, datatypes::DataType};
// use crate::data::memory_engine::InMemoryEngine;

pub struct Server {
    // engine: Box<dyn StorageEngine>, Why doesn't this work? https://doc.rust-lang.org/reference/items/traits.html#object-safety
    engine: InMemoryEngine,
    // engine: ThreadEngineManager,
}

impl Server {
    pub fn new() -> Server {
        Server {
            engine: InMemoryEngine::new(),
            // engine: ThreadEngineManager::new(),
        }
    }

    pub async fn process_command(&self, command: Command) -> Result<DataType, String> {
        match command {
            Command::Set(command) => self.engine.process_set(command).await,
            Command::Get { key } => self.engine.process_get(key).await,
            Command::ConfigGet { .. } => {
                Ok(DataType::Array(vec![DataType::BulkString("save".into()), DataType::BulkString("3600 1 300 100 60 10000".into())]))
            },
            Command::Dump => self.engine.process_dump().await,
        }
    }
}
