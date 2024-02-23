#[derive(Debug, PartialEq)]
pub enum DataType {
    Nil,
    SimpleString(String),
    BulkString(String),
    Array(Vec<DataType>),
    Error(String),
}

#[derive(Debug, Clone)]
pub(crate) enum StorageValue {
    String(String),
}

#[derive(Debug, Clone)]
pub(crate) struct StorageRecord {
    pub value: StorageValue,
    pub ttl: Option<u128>,
}
