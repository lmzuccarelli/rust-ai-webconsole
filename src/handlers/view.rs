use crate::handlers::common::{get_error, get_opts};
use crate::handlers::interface::ViewformInterface;
use async_trait::async_trait;
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub document: String,
}

#[async_trait]
impl ViewformInterface for View {
    async fn get_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>> {
        let params: Vec<&str> = req_uri.split("/").collect();
        let key = params.last().unwrap().to_string();
        log::debug!("[get_fromdata] view key {}", key);
        let result = db_read(key).await?;
        Ok(result)
    }

    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
        let result = db_upsert(data).await?;
        Ok(result)
    }
}

#[allow(dead_code)]
async fn db_upsert(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts("documents".to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    txn.set_durability(surrealkv::Durability::Immediate);
    let view: View = serde_json::from_slice(&data)?;
    let b_key = Bytes::from(view.name.clone());
    let b_value = Bytes::from(view.document);
    log::debug!("[db_upsert] document with key {}", view.name);
    txn.set(&b_key, &b_value)
        .map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    let msg = format!("document {} created/updated successfully", view.name);
    Ok(msg)
}

async fn db_read(key: String) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts("documents".to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let b_key = Bytes::from(key.clone());
    let res = txn.get(&b_key).map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    match res {
        Some(val) => {
            let document = String::from_utf8(val.to_vec())?;
            Ok(document)
        }
        None => {
            let msg = format!("no document found with key {}", key);
            log::error!("{}", msg);
            Err(get_error(msg))
        }
    }
}
