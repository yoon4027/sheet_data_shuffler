use std::path::Path;

use eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use toml::from_str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    auth_file: String,
    sheet_id: String,
    range: String,
    sheet_name: String,
}

impl Config {
    pub async fn new(path: &Path) -> Result<Config> {
        let data = read_to_string(Path::new(&path)).await?;

        let result = from_str::<Config>(data.as_str())?;

        Ok(result)
    }

    pub fn get_auth_file(&self) -> &String {
        &self.auth_file
    }

    pub fn get_sheet_id(&self) -> &String {
        &self.sheet_id
    }

    pub fn get_range(&self) -> &String {
        &self.range
    }

    pub fn get_sheet_name(&self) -> &String {
        &self.sheet_name
    }
}
