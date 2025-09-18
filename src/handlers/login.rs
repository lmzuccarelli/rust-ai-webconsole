use crate::handlers::common::{get_error, get_opts};
use crate::handlers::interface::LoginformInterface;
use async_trait::async_trait;
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub password: String,
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {}

#[async_trait]
impl LoginformInterface for User {
    async fn get_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
        let value = String::from_utf8(data.to_vec())?;
        let (user_res, password_res) = value.split_once("&").ok_or("could not parse parameters")?;
        let user = user_res.split("=").last().ok_or("could not parse user")?;
        let password = password_res
            .split("=")
            .last()
            .ok_or("could not parse password")?;
        let result = db_read(user.to_string(), password.to_string()).await?;
        Ok(result)
    }

    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
        let value = String::from_utf8(data.to_vec())?;
        let (user_res, password_res) = value.split_once("&").ok_or("could not parse parameters")?;
        let user = user_res.split("=").last().ok_or("could not parse user")?;
        let password = password_res
            .split("=")
            .last()
            .ok_or("could not parse password")?;
        let result =
            db_upsert(user.to_string(), password.to_string(), "123456".to_string()).await?;
        Ok(result)
    }
}

async fn db_upsert(
    id: String,
    password: String,
    session_id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts("login".to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    log::debug!("transaction begin");
    let key = Bytes::from(id.clone());
    let ud = UserData {
        password,
        session_id,
    };
    let json_data = serde_json::to_string(&ud)?;
    let value = Bytes::from(json_data);
    txn.set(&key, &value)
        .map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    let msg = format!("user {} registered successfully", id);
    Ok(msg)
}

async fn db_read(id: String, password: String) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts("login".to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let key = Bytes::from(id.clone());
    let res = txn.get(&key).map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    match res {
        Some(val) => {
            let ud: UserData =
                serde_json::from_slice(&val).map_err(|e| get_error(e.to_string()))?;
            if ud.password != password {
                return Err(get_error("incorrect credentials".to_string()));
            }
            Ok("login successful".to_string())
        }
        None => {
            let msg = format!("no record found for user {} (have you registered ?)", id);
            log::error!("{}", msg);
            Err(get_error(msg))
        }
    }
}
