use serde_derive::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Parameters {
    pub name: String,
    pub description: String,
    pub port: String,
    pub log_level: String,
    pub certs_dir: Option<String>,
    pub cert_mode: String,
    pub db_path: String,
    pub deploy_dir: String,
    pub static_dir: String,
}

pub trait ConfigInterface {
    fn read(&self, dir: String) -> Result<Parameters, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct ImplConfigInterface {}

impl ConfigInterface for ImplConfigInterface {
    fn read(&self, name: String) -> Result<Parameters, Box<dyn std::error::Error>> {
        let json_data = File::open(&name)?;
        let params = serde_json::from_reader(json_data)?;
        Ok(params)
    }
}
