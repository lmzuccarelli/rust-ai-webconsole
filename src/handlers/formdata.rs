use crate::handlers::common::{get_error, get_opts};
use crate::handlers::interface::InputformInterface;
use async_trait::async_trait;
use chrono::Local;
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormData {
    pub key: Option<String>,
    pub title: String,
    pub file: String,
    pub category: String,
    pub prompt: String,
    pub credentials: String,
    pub run_once: String,
    pub db: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchData {
    pub dbsearch: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {}

#[async_trait]
impl InputformInterface for Form {
    async fn save_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
        log::debug!(
            "[save_formdata] {}",
            String::from_utf8(data.to_vec()).unwrap()
        );
        let mut fd: FormData = serde_json::from_slice(&data)?;
        log::debug!("[save_formdata] struct {:?}", fd);
        let key = match fd.key.clone() {
            Some(key) => {
                if key == "" {
                    let now = Local::now();
                    now.format("%Y%m%d%H%M%S").to_string()
                } else {
                    key
                }
            }
            None => {
                let now = Local::now();
                now.format("%Y%m%d%H%M%S").to_string()
            }
        };
        fd.key = Some(key.clone());
        let result = db_upsert(key.clone(), fd.clone()).await?;
        Ok(result)
    }

    async fn search_formdata(data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
        let sd: SearchData = serde_json::from_slice(&data)?;
        let result = db_read_search(sd).await?;
        let html = render_results_html(result);
        Ok(html)
    }

    async fn get_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>> {
        let params: Vec<&str> = req_uri.split("/").collect();
        log::debug!("[get_formdata] params {:?}", params);
        if params.len() <= 4 {
            return Err(get_error("uri parameters are incorrrect".to_string()));
        }
        let key = params.get(params.len() - 2);
        log::debug!(
            "[get_formdata] key {}",
            params.get(params.len() - 2).unwrap().to_string(),
        );
        let fd = db_read(key.unwrap().to_string(), params.last().unwrap().to_string()).await?;
        let html = render_form_html(fd.clone().key.unwrap_or("".to_string()), fd);
        Ok(html)
    }
}

async fn db_upsert(id: String, fd: FormData) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts("formdata".to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    txn.set_durability(surrealkv::Durability::Immediate);
    let key = Bytes::from(id.clone());
    let json_data = serde_json::to_string(&fd)?;
    log::debug!("[db_upsert] formdata {}", json_data);
    let value = Bytes::from(json_data);
    txn.set(&key, &value)
        .map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    let msg = format!("form data {} created/updated successfully", id);
    Ok(msg)
}

async fn db_read(id: String, db: String) -> Result<FormData, Box<dyn std::error::Error>> {
    let tree = get_opts(db)?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let key = Bytes::from(id.clone());
    log::debug!("[db_read] key {}", id);
    let result = txn.get(&key).map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    match result {
        Some(value) => {
            let fd = serde_json::from_slice(&value).map_err(|e| get_error(e.to_string()))?;
            log::trace!("[db_read] {:?}", fd);
            Ok(fd)
        }
        None => {
            let fd = FormData {
                key: None,
                title: "".to_string(),
                file: "".to_string(),
                category: "".to_string(),
                prompt: "".to_string(),
                credentials: "".to_string(),
                run_once: "on".to_string(),
                db: "formdata".to_string(),
            };
            Ok(fd)
        }
    }
}

async fn db_read_search(
    sd: SearchData,
) -> Result<HashMap<String, FormData>, Box<dyn std::error::Error>> {
    let mut hm: HashMap<String, FormData> = HashMap::new();
    log::debug!("[db_read_search] db list {}", sd.dbsearch);
    let db = match sd.dbsearch.as_str() {
        "kv-queue-db" => "queue",
        "kv-formdata-db" => "formdata",
        "kv-archive-db" => "archive",
        _ => "formdata",
    };
    log::debug!("using db {}", db);
    let tree = get_opts(db.to_string())?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let start = format!("{}000059", sd.from.replace("-", ""));
    let end = format!("{}235959", sd.to.replace("-", ""));
    let start_b = start.as_bytes();
    let end_b = end.as_bytes();
    let results = txn.range(start_b, end_b, None)?;
    for x in results.into_iter() {
        let kv = x.as_ref().unwrap();
        let key = kv.0.to_vec();
        let value = kv.1.as_ref().unwrap().to_vec();
        let s_key = str::from_utf8(&key);
        let s_value = str::from_utf8(&value);
        log::info!("{} {}", s_key.unwrap(), s_value.unwrap());
        let v = String::from_utf8(value)?;
        let fd = serde_json::from_str(&v)?;
        hm.insert(s_key.unwrap().to_owned(), fd);
    }
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    Ok(hm.clone())
}

fn render_results_html(rows: HashMap<String, FormData>) -> String {
    let mut html = String::new();
    for (key, fd) in rows.iter() {
        let html_row = format!(
            "
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td><i class=\"fa fa-trash-o\" hx-post=\"/webconsole/delete/{}\" hx-trigger=\"click\"></i>&nbsp&nbsp;&nbsp;<i id=\"icon-formdata\" class=\"fa fa-edit\" hx-get=\"/webconsole/formdata/{}/{}\" hx-target=\"#inputForm\" hx-trigger=\"click\"></i></td>
        </tr>",
            key, fd.title, fd.category, fd.file, fd.prompt, key, key,fd.db
        );
        html.push_str(&html_row);
    }
    html
}

fn render_form_html(key: String, fd: FormData) -> String {
    let checked = match fd.run_once.as_str() {
        "on" => "checked",
        _ => "",
    };
    let html = format!(
        r##"
            <h2>AI Form Details</h2>
            <form id="formdata" hx-post="/webconsole/formdata" hx-ext="json-enc">
                <input type="hidden" id="key" name="key" value="{}">
                <input type="hidden" id="credentials" name="credentials" value="{}">
                <input type="hidden" id="db" name="db" value="{}">
                <div class="form-group">
                    <label for="title">Title</label>
                    <input type="text" id="title" name="title" value="{}" required>
                </div>
                <div class="form-group">
                    <label for="category">Category</label>
                    <select id="category" name="category">
                        {}
                    </select>
                </div>
                <div class="form-group">
                    <label for="file">File</label>
                    <input type="text" id="file" name="file" value="{}" required>
                </div>
                <div class="form-group">
                    <label for="prompt">Prompt</label>
                    <textarea id="prompt" name="prompt" rows="8" required>{}</textarea>
                </div>
                <div class="form-group">
                    <label for="run-once">Run Once</label>
                    <input type="checkbox" id="run_once" name="run_once" style="accent-color: #444444;" {}>
                </div>
                <button id="submit-formdata" type="submit">Submit</button>
            </form>
    "##,
        key,
        fd.credentials,
        fd.db,
        fd.title,
        check_selected(fd.category),
        fd.file,
        fd.prompt,
        checked
    );
    html
}

fn check_selected(category: String) -> String {
    let mut vec_selected = vec!["", "", "", ""];
    match category.as_str() {
        "generic" => {
            vec_selected[0] = "selected";
        }
        "stock" => {
            vec_selected[1] = "selected";
        }
        "projects" => {
            vec_selected[2] = "selected";
        }
        "programming" => {
            vec_selected[3] = "selected";
        }
        _ => {
            vec_selected[0] = "selected";
        }
    };

    let result = format!(
        r#"
    <option id="generic" name="generic" {}>generic</option>
    <option id="stock" name="stock" {}>stock</option>
    <option id="projects" name="projects" {}>projects</option>
    <option id="programming" name="programming" {}>programming</option>
    "#,
        vec_selected[0], vec_selected[1], vec_selected[2], vec_selected[3]
    );
    result
}
