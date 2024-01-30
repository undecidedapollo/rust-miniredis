#[derive(Debug, PartialEq)]
pub enum DataType {
    Nil,
    SimpleString(String),
    BulkString(String),
    Array(Vec<DataType>),
    Error(String),
}
