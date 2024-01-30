#[derive(Debug)]
pub enum DataType {
    Nil,
    SimpleString(String),
    BulkString(String),
    Array(Vec<DataType>),
}
