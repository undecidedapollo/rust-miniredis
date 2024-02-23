use std::{collections::HashMap, hash::{Hash, Hasher, SipHasher}};

use crate::{commands::{SetCommand, SetExistingOptions}, datatypes::{DataType, StorageRecord, StorageValue}};

pub(crate) fn hashy(str: &str) -> u64 {
    let mut hasher = SipHasher::new_with_keys(0, 0);
    str.hash(&mut hasher);
    let res = hasher.finish();
    // println!("Hash result: {str} -> {res}");
    return res;
}

pub(crate) fn process_set(map: &mut HashMap<String, StorageRecord>, cmd: SetCommand) -> Result<DataType, String> {
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

pub(crate) fn process_get(map: &HashMap<String, StorageRecord>, key: String) -> Result<DataType, String> {
    let val = map.get(&key);
    match val {
        Some(StorageRecord{
            value: StorageValue::String(x),
            ..
        }) => Ok(DataType::BulkString(x.into())),
        None => Ok(DataType::Nil),
    }
}
