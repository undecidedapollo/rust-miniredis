#[derive(Debug, PartialEq)]
pub enum Command {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
}
