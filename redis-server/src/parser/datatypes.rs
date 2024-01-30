use crate::commands::{Command, SetCommand, SetExistingOptions};
use crate::datatypes::DataType;
use phf::phf_map;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

struct CommandParsingContext {
    now: Duration,
}

static COMMAND_PARSER: phf::Map<
    &'static str,
    fn(ctx: &CommandParsingContext, datatype: &[DataType]) -> Result<Command, String>,
> = phf_map! {
    "set" => parse_set,
    "get" => parse_get,
    "dump" => parse_dump,
};

fn current_unix_timestamp_millis() -> Duration {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).expect("SystemTime before UNIX EPOCH!")
}

fn parse_set(ctx: &CommandParsingContext, x: &[DataType]) -> Result<Command, String> {
    match x {
        [DataType::BulkString(key), DataType::BulkString(value), rest@..] => {
            let mut command = SetCommand {
                key: key.to_string(),
                value: value.to_string(),
                ..Default::default()
            };

            let mut idx = 0;
            while idx < rest.len() {
                let mut read_next = || {
                    let Some(DataType::BulkString(x)) = rest.get(idx) else {
                        return Err("Invalid datatype, expected BulkString".to_string());
                    };
                    idx += 1;
                    return Ok(x);
                };

                let x = read_next()?;

                match x.as_ref() {
                    "NX" => {
                        command.set_existing = Some(SetExistingOptions::OnlySetIfNotExists);
                    },
                    "XX" => {
                        command.set_existing = Some(SetExistingOptions::OnlySetIfExists);
                    },
                    "EX" => {
                        let next_val = read_next()?;
                        let seconds = next_val.parse::<u64>().map_err(|err| err.to_string())?;
                        let duration = Duration::from_secs(seconds);
                        let result = ctx.now + duration;
                        let ms_since = result.as_millis();
                        command.expiration = Some(ms_since);
                    },
                    "PX" => {
                        let next_val = read_next()?;
                        let milliseconds = next_val.parse::<u64>().map_err(|err| err.to_string())?;
                        let duration = Duration::from_millis(milliseconds);
                        let result = ctx.now + duration;
                        let ms_since = result.as_millis();
                        command.expiration = Some(ms_since);
                    },
                    "EXAT" => {
                        let next_val = read_next()?;
                        let seconds = next_val.parse::<u128>().map_err(|err| err.to_string())?;
                        command.expiration = Some(seconds.saturating_mul(1000));
                    },
                    "PXAT" => {
                        let next_val = read_next()?;
                        let milliseconds = next_val.parse::<u128>().map_err(|err| err.to_string())?;
                        command.expiration = Some(milliseconds);
                    },
                    x => {
                        return Err(format!("Invalid command sequence: {}", x));
                    }
                }
            }

            Ok(Command::Set(command))
        },
        _ => Err("Invalid structure".into()),
    }
}

fn parse_get(_: &CommandParsingContext, x: &[DataType]) -> Result<Command, String> {
    match x {
        [DataType::BulkString(key)] => Ok(Command::Get {
            key: key.to_string(),
        }),
        _ => Err("Invalid structure".into()),
    }
}

fn parse_dump(_: &CommandParsingContext, _: &[DataType]) -> Result<Command, String> {
    Ok(Command::Dump)
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

                        let context = CommandParsingContext {
                            now: current_unix_timestamp_millis(),
                        };

                        handler(&context, values)
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
            Command::Set(SetCommand{
                key: "X".into(),
                value: "1".into(),
                ..Default::default()
            })
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

        assert_eq!(output, Command::Get { key: "Y".into() });
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

    mod tests_set_expirations {
        use super::*;
    
        #[test]
        pub fn test_set_ex() {
            let data = vec![
                DataType::BulkString("X".into()),
                DataType::BulkString("1".into()),
                DataType::BulkString("EX".into()),
                DataType::BulkString("10".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "X".into(),
                    value: "1".into(),
                    expiration: Some(Duration::from_secs(20).as_millis()),
                    ..Default::default()
                })
            );
        }
    
        #[test]
        pub fn test_set_px() {
            let data = vec![
                DataType::BulkString("Y".into()),
                DataType::BulkString("2".into()),
                DataType::BulkString("PX".into()),
                DataType::BulkString("10000".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "Y".into(),
                    value: "2".into(),
                    expiration: Some(Duration::from_millis(20000).as_millis()),
                    ..Default::default()
                })
            );
        }
    
        #[test]
        pub fn test_set_exat() {
            let data = vec![
                DataType::BulkString("Z".into()),
                DataType::BulkString("3".into()),
                DataType::BulkString("EXAT".into()),
                DataType::BulkString("1234".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "Z".into(),
                    value: "3".into(),
                    expiration: Some(Duration::from_secs(1234).as_millis()),
                    ..Default::default()
                })
            );
        }
    
        #[test]
        pub fn test_set_pxat() {
            let data = vec![
                DataType::BulkString("A".into()),
                DataType::BulkString("4".into()),
                DataType::BulkString("PXAT".into()),
                DataType::BulkString("5678".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "A".into(),
                    value: "4".into(),
                    expiration: Some(5678),
                    ..Default::default()
                })
            );
        }
    }

    mod tests_set_nx_xx {
        use super::*;
    
        #[test]
        pub fn test_set_nx() {
            let data = vec![
                DataType::BulkString("X".into()),
                DataType::BulkString("1".into()),
                DataType::BulkString("NX".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "X".into(),
                    value: "1".into(),
                    set_existing: Some(SetExistingOptions::OnlySetIfNotExists),
                    ..Default::default()
                })
            );
        }

        #[test]
        pub fn test_set_xx() {
            let data = vec![
                DataType::BulkString("X".into()),
                DataType::BulkString("1".into()),
                DataType::BulkString("XX".into()),
            ];
    
            let output = parse_set(&CommandParsingContext {
                now: Duration::from_secs(10),
            }, &data).expect("Expect parsing to be successful");
    
            assert_eq!(
                output,
                Command::Set(SetCommand {
                    key: "X".into(),
                    value: "1".into(),
                    set_existing: Some(SetExistingOptions::OnlySetIfExists),
                    ..Default::default()
                })
            );
        }
    }
}
