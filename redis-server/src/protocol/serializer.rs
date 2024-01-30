use crate::datatypes::DataType;

impl DataType {
    pub fn to_wire_protocol(&self) -> String {
        match self {
            DataType::SimpleString(str) => format!("+{}\r\n", str),
            DataType::BulkString(str) => format!("${}\r\n{}\r\n", str.len(), str),
            DataType::Array(data) => {
                let output = data
                    .iter()
                    .map(|x| x.to_wire_protocol())
                    .collect::<Vec<String>>();
                format!("*{}\r\n{}", output.len(), output.join(""))
            },
            DataType::Error(str)  => format!("-{}\r\n", str),
            DataType::Nil => "$-1\r\n".to_string(),
        }
    }
}
