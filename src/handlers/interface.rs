use async_trait::async_trait;
use hyper::body::Bytes;

// Use of traits (interfaces) assists with adhereing to the SOLID principles
// I mainly focused on the first 3 principles
// Single responsibility
// Open for extension closed for modification
// Liskovs subsitution (evident in the interfaces declared below)

#[async_trait]
pub trait InputformInterface {
    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>>;
    async fn search_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>>;
    async fn delete_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait LoginformInterface {
    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>>;
}

#[allow(dead_code)]
#[async_trait]
pub trait ViewformInterface {
    async fn get_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>>;
    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>>;
}
