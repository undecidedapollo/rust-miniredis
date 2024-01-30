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
                let output_result = output.join("");
                format!("*{}\r\n{}", output.len(), output_result)
            }
            DataType::Nil => "Nil".to_string(),
        }
    }
}
