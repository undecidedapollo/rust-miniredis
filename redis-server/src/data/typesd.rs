use crate::{commands::SetCommand, datatypes::DataType};

pub(crate) trait StorageEngine {
    async fn process_set(&self, cmd: SetCommand) -> Result<DataType, String>;
    async fn process_get(&self, key: String) -> Result<DataType, String>;
    async fn process_dump(&self) -> Result<DataType, String>;
}
