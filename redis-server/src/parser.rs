use std::collections::HashMap;

use phf::phf_map;

use crate::commands::Command;
use crate::datatypes::DataType;

static COMMAND_PARSER: phf::Map<
    &'static str,
    fn(datatype: &[DataType]) -> Result<Command, String>,
> = phf_map! {
    "set" => parse_set,
    "get" => parse_get,
};

fn parse_set(x: &[DataType]) -> Result<Command, String> {
    match x {
        [DataType::BulkString(key), DataType::BulkString(value)] => Ok(Command::Set {
            key: key.to_string(),
            value: value.to_string(),
        }),
        _ => Err("Invalid structure".into()),
    }
}

fn parse_get(x: &[DataType]) -> Result<Command, String> {
    match x {
        [DataType::BulkString(key)] => Ok(Command::Get {
            key: key.to_string(),
        }),
        _ => Err("Invalid structure".into()),
    }
}

impl DataType {
    pub fn to_command(&self) -> Result<Command, String> {
        match self {
            DataType::Array(arr) => {
                let (command, values) =
                    arr.split_first().ok_or("Invalid array shape".to_string())?;

                match command {
                    DataType::BulkString(x) => {
                        let Some(handler) = COMMAND_PARSER.get(x.to_lowercase().as_ref()) else {
                            return Err(format!("Unknown command: {}", x.to_lowercase()));
                        };

                        handler(values)
                    }
                    _ => Err("Invalid array command type, expected bulk string".to_string()),
                }
            }
            _ => Err("Invalid command".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_set_kv() {
        let data = DataType::Array(vec![
            DataType::BulkString("SET".into()),
            DataType::BulkString("X".into()),
            DataType::BulkString("1".into()),
        ]);

        let output = data
            .to_command()
            .expect("Expected the command to parse successfully");

        assert_eq!(
            output,
            Command::Set {
                key: "X".into(),
                value: "1".into()
            }
        );
    }

        
    #[test]
    pub fn test_get_key() {
        let data = DataType::Array(vec![
            DataType::BulkString("GET".into()),
            DataType::BulkString("Y".into()),
        ]);

        let output = data
            .to_command()
            .expect("Expected the command to parse successfully");

        assert_eq!(
            output,
            Command::Get {
                key: "Y".into(),
            }
        );
    }
    
    #[test]
    pub fn test_invalid_structure() {
        let data = DataType::Array(vec![
            DataType::BulkString("SET".into()),
            DataType::BulkString("Z".into()),
            DataType::BulkString("2".into()),
            DataType::BulkString("3".into()),
        ]);

        let output = data.to_command();

        assert!(output.is_err());
    }
    
    #[test]
    pub fn test_unknown_command() {
        let data = DataType::Array(vec![
            DataType::BulkString("UNKNOWN".into()),
            DataType::BulkString("A".into()),
            DataType::BulkString("B".into()),
        ]);

        let output = data.to_command();

        assert!(output.is_err());
    }
}
