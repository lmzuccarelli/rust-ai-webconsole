use crate::handlers::common::{get_error, get_opts};
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};
use surrealkv::Store;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub password: String,
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
}

impl User {
    pub fn new(id: String) -> Self {
        Self { id: id }
    }

    pub fn db_upsert(
        &self,
        password: String,
        session_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let opts = get_opts("login".to_string())?;
        let store = Store::new(opts).map_err(|e| get_error(e.to_string()))?;
        // start transaction
        let mut txn = store.begin().map_err(|e| get_error(e.to_string()))?;
        let key = Bytes::from(self.id.to_string());
        let ud = UserData {
            password,
            session_id,
        };
        let json_data = serde_json::to_string(&ud)?;
        let value = Bytes::from(json_data);
        txn.insert_or_replace(&key, &value)
            .map_err(|e| get_error(e.to_string()))?;
        // commit transaction
        txn.commit().map_err(|e| get_error(e.to_string()))?;
        store.close().map_err(|e| get_error(e.to_string()))?;
        let msg = format!("user {} registered successfully", self.id);
        Ok(msg)
    }

    pub fn db_read(&self, password: String) -> Result<String, Box<dyn std::error::Error>> {
        let opts = get_opts("login".to_string())?;
        let store = Store::new(opts).map_err(|e| get_error(e.to_string()))?;
        // start transaction
        let mut txn = store.begin().map_err(|e| get_error(e.to_string()))?;
        let key = Bytes::from(self.id.to_owned());
        let res = txn.get(&key).map_err(|e| get_error(e.to_string()))?;
        // commit transaction
        txn.commit().map_err(|e| get_error(e.to_string()))?;
        store.close().map_err(|e| get_error(e.to_string()))?;
        match res {
            Some(val) => {
                let ud: UserData =
                    serde_json::from_slice(&val).map_err(|e| get_error(e.to_string()))?;
                if ud.password != password {
                    return Err(get_error("incorrect credentials".to_string()));
                }
                Ok(ud.session_id)
            }
            None => {
                let msg = format!(
                    "no record found for user {} (have you registered ?)",
                    self.id
                );
                log::error!("{}", msg);
                Err(get_error(msg))
            }
        }
    }
}
