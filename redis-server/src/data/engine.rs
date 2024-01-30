use crate::{commands::{SetCommand, SetExistingOptions}, datatypes::DataType};
use std::{collections::HashMap, sync::Mutex};

#[derive(Debug)]
enum StorageValue {
    String(String),
}

#[derive(Debug)]
struct StorageRecord {
    value: StorageValue,
    ttl: Option<u128>,
}

pub struct InMemoryEngine {
    keymap: Mutex<HashMap<String, StorageRecord>>,
}

impl InMemoryEngine {
    pub fn new() -> InMemoryEngine {
        InMemoryEngine {
            keymap: Mutex::from(HashMap::new()),
        }
    }

    pub fn process_set(&self, cmd: SetCommand) -> Result<DataType, String> {
        let mut map = self.keymap.lock().map_err(|err| err.to_string())?;
        let previous_obj: Option<&StorageRecord> = map.get(&cmd.key);
        let previous_value = match (cmd.get_previous_value, previous_obj) {
            (true, Some(stored_value)) => {
                let StorageValue::String(x) = &stored_value.value else {
                    // TODO: Update the error message to match redis message
                    return Ok(DataType::Error(format!(
                        "Error: Expected key {} to be of type String",
                        cmd.key
                    )));
                };
                Some(x.to_string())
            }
            _ => None,
        };

        let previous_ttl = match (cmd.keep_previous_ttl, previous_obj) {
            (true, Some(stored_value)) => stored_value.ttl,
            _ => None,
        };

        let ttl = previous_ttl.or(cmd.expiration);
        let storage_record = StorageRecord {
            ttl,
            value: StorageValue::String(cmd.value),
        };

        let should_insert = match (cmd.set_existing, previous_obj) {
            (None, _) => true,
            (Some(SetExistingOptions::OnlySetIfExists), Some(_)) => true,
            (Some(SetExistingOptions::OnlySetIfNotExists), None) => true,
            _ => false,
        };

        if should_insert {
            map.insert(cmd.key, storage_record);
        }

        return match previous_value {
            Some(value) => Ok(DataType::BulkString(value)),
            None => Ok(DataType::SimpleString("OK".into())),
        };
    }

    pub fn process_get(&self, key: String) -> Result<DataType, String> {
        let map = self.keymap.lock().map_err(|err| err.to_string())?;
        let val = map.get(&key);
        match val {
            Some(StorageRecord{
                value: StorageValue::String(x),
                ..
            }) => Ok(DataType::BulkString(x.into())),
            None => Ok(DataType::Nil),
        }
    }

    pub fn process_dump(&self) -> Result<DataType, String> {
        let map = self.keymap.lock().map_err(|err| err.to_string())?;

        println!("{:#?}", map);

        Ok(DataType::Nil)
    }
}
