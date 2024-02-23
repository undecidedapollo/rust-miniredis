use crate::{commands::{Command, SetCommand, SetExistingOptions}, data::shared::hashy, datatypes::DataType};
use std::{cell::RefCell, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}, sync::mpsc::{channel, Receiver}, thread::{self, JoinHandle}};
use std::sync::mpsc::Sender;
use std::thread::available_parallelism;
use tokio::sync::oneshot;

use super::typesd::StorageEngine;

#[derive(Debug, Clone)]
enum StorageValue {
    String(String),
}

#[derive(Debug, Clone)]
struct StorageRecord {
    value: StorageValue,
    ttl: Option<u128>,
}

struct ThreadEngineInternal {
    map: HashMap<String, StorageRecord>,
}

impl ThreadEngineInternal{
    fn new() -> ThreadEngineInternal {
        ThreadEngineInternal {
            map: HashMap::new()
        }
    }

    pub fn process_set(&mut self, cmd: SetCommand) -> Result<DataType, String> {
        let map = &mut self.map;
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
        let map = &self.map;
        let val = map.get(&key);
        match val {
            Some(StorageRecord{
                value: StorageValue::String(x),
                ..
            }) => Ok(DataType::BulkString(x.into())),
            None => Ok(DataType::Nil),
        }
    }
}

pub struct ThreadEngine {
    handle: JoinHandle<()>,
}

impl ThreadEngine {
    pub fn new(receiver: Receiver<ThreadEngineProcessMessage>) -> ThreadEngine {
        let handle = thread::spawn(move || {
            let mut interal_thread_engine = ThreadEngineInternal::new();
            while let Ok(msg) = receiver.recv() {
                match msg.command {
                    Command::Get { key } => {
                        let res = interal_thread_engine.process_get(key);
                        msg.response.send(res).unwrap(); // TODO Fix this
                    },
                    Command::Set(set_cmd) => {
                        let res = interal_thread_engine.process_set(set_cmd);
                        msg.response.send(res).unwrap(); // TODO Fix this
                    },
                    _ => {
                        todo!()
                    }
                }
            }
        });

        ThreadEngine {
            handle,
        }
    }
}

pub struct ThreadEngineProcessMessage{
    command: Command,
    response: oneshot::Sender<Result<DataType, String>>,
}

struct ThreadEngineRecord{
    engine: ThreadEngine,
    sender: Sender<ThreadEngineProcessMessage>,
}

pub struct ThreadEngineManager {
    parallelism_count: u64,
    keymap: Vec<ThreadEngineRecord>,
}

impl ThreadEngineManager {
    pub fn new() -> ThreadEngineManager {
        let default_parallelism_approx = available_parallelism().unwrap().get();
        let mut v = Vec::with_capacity(default_parallelism_approx);

        for _ in 0..default_parallelism_approx {
            let (sender, receiver) = channel::<ThreadEngineProcessMessage>();
            let engine = ThreadEngine::new(receiver);
            v.push(ThreadEngineRecord { engine, sender });
        }

        ThreadEngineManager {
            parallelism_count: default_parallelism_approx as u64,
            keymap: v,
        }
    }
}

impl ThreadEngineManager {
    fn get_engine_for_matching_thread(&self, str: &str) -> &ThreadEngineRecord {
        let hash = hashy(str);
        let index = (hash % self.parallelism_count) as usize;
        // println!("hash: {str}: {index}({hash}) (");
        &self.keymap[index]
    }
}

impl StorageEngine for ThreadEngineManager {
    async fn process_set(&self, cmd: SetCommand) -> Result<DataType, String> {
        let engine = self.get_engine_for_matching_thread(&cmd.key);
        let (sender, receiver) = oneshot::channel::<Result<DataType, String>>();
        engine.sender.send(ThreadEngineProcessMessage {
            command: Command::Set(cmd),
            response: sender,
        }).map_err(|e| format!("An error occurred sending a message to the thread engine: {}", e.to_string()))?;

        let output = receiver.await.map_err(|e| format!("An error occurred waiting on a response from the thread engine: {}", e.to_string()))?;
        return output;
    }

    async fn process_get(&self, key: String) -> Result<DataType, String> {
        let engine = self.get_engine_for_matching_thread(&key);
        let (sender, receiver) = oneshot::channel::<Result<DataType, String>>();
        engine.sender.send(ThreadEngineProcessMessage {
            command: Command::Get { key: key },
            response: sender,
        }).map_err(|e| format!("An error occurred sending a message to the thread engine: {}", e.to_string()))?;

        let output = receiver.await.map_err(|e| format!("An error occurred waiting on a response from the thread engine: {}", e.to_string()))?;
        return output;
    }

    async fn process_dump(&self) -> Result<DataType, String> {
        todo!()
    }
}

