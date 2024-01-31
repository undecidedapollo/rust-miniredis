#[derive(Debug, PartialEq)]
pub enum SetExistingOptions {
    OnlySetIfNotExists,
    OnlySetIfExists,
}

#[derive(Debug, PartialEq, Default)]
pub struct SetCommand {
    pub key: String,
    pub value: String,
    pub expiration: Option<u128>,
    pub set_existing: Option<SetExistingOptions>,
    pub keep_previous_ttl: bool,
    pub get_previous_value: bool,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Set(SetCommand),
    Get {
        key: String,
    },
    ConfigGet {
        key: Option<String>,
    },
    Dump,
}
