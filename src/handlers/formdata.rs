use crate::handlers::common::{get_error, get_map_item, get_opts};
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use surrealkv::Store;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormData {
    pub key: Option<String>,
    pub title: String,
    pub file: String,
    pub prompt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchData {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
    pub id: String,
}

impl Form {
    pub fn new(id: String) -> Self {
        Self { id: id }
    }

    pub fn db_upsert(&self, fd: FormData) -> Result<String, Box<dyn std::error::Error>> {
        let opts = get_opts("formdata".to_string())?;
        let store = Store::new(opts).map_err(|e| get_error(e.to_string()))?;
        // start transaction
        let mut txn = store.begin().map_err(|e| get_error(e.to_string()))?;
        let key = Bytes::from(self.id.to_string());
        let json_data = serde_json::to_string(&fd)?;
        let value = Bytes::from(json_data);
        txn.insert_or_replace(&key, &value)
            .map_err(|e| get_error(e.to_string()))?;
        // commit transaction
        txn.commit().map_err(|e| get_error(e.to_string()))?;
        store.close().map_err(|e| get_error(e.to_string()))?;
        let msg = format!("form data {} created successfully", self.id);
        Ok(msg)
    }

    pub fn db_read(&self) -> Result<FormData, Box<dyn std::error::Error>> {
        let opts = get_opts("formdata".to_string())?;
        let store = Store::new(opts).map_err(|e| get_error(e.to_string()))?;
        // start transaction
        let mut txn = store.begin().map_err(|e| get_error(e.to_string()))?;
        let key = Bytes::from(self.id.clone());
        let result = txn.get(&key).map_err(|e| get_error(e.to_string()))?;
        // commit transaction
        txn.commit().map_err(|e| get_error(e.to_string()))?;
        store.close().map_err(|e| get_error(e.to_string()))?;
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
                    prompt: "".to_string(),
                };
                Ok(fd)
            }
        }
    }

    pub fn db_read_search(
        &self,
        sd: SearchData,
    ) -> Result<HashMap<String, FormData>, Box<dyn std::error::Error>> {
        let mut hm: HashMap<String, FormData> = HashMap::new();
        let opts = get_opts("formdata".to_string())?;
        let store = Store::new(opts).map_err(|e| get_error(e.to_string()))?;
        // start transaction
        let mut txn = store.begin().map_err(|e| get_error(e.to_string()))?;
        let start = format!("{}000059", sd.from.replace("-", ""));
        let end = format!("{}235959", sd.to.replace("-", ""));
        let range = std::ops::Range {
            start: start.as_bytes(),
            end: end.as_bytes(),
        };
        let results = txn.scan(range, Some(24));
        for x in results.into_iter() {
            let (a, b, _c) = x?;
            let key = String::from_utf8(a.to_vec())?;
            let value = String::from_utf8(b)?;
            let fd = serde_json::from_str(&value)?;
            hm.insert(key, fd);
        }
        // commit transaction
        txn.commit().map_err(|e| get_error(e.to_string()))?;
        store.close().map_err(|e| get_error(e.to_string()))?;
        Ok(hm.clone())
    }
}

pub fn deploy_formdata(name: String, data: Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let deploy_dir = get_map_item("deploy_dir".to_string())?;
    fs::write(format!("{}/{}", deploy_dir, name), data)?;
    let result = format!("formdata {} deployed successfully", name);
    Ok(result)
}

pub fn render_results_html(rows: HashMap<String, FormData>) -> String {
    let mut html = String::new();
    for (key, fd) in rows.iter() {
        let html_row = format!(
            "
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td><i class=\"fa fa-trash-o\" hx-post=\"/delete/{}\" hx-trigger=\"click\"></i>&nbsp&nbsp;&nbsp;<i id=\"icon-formdata\" class=\"fa fa-edit\" hx-get=\"/formdata/{}\" hx-target=\"#inputForm\" hx-trigger=\"click\"></i></td>
        </tr>",
            key, fd.file, fd.title, fd.prompt,key,key
        );
        html.push_str(&html_row);
    }
    html
}

pub fn render_form_html(key: String, fd: FormData) -> String {
    let html = format!(
        r##"
            <h2>AI Form Details</h2>
            <form id="formdata" hx-post="/formdata" hx-ext="json-enc">
                <input type="hidden" id="key" name="key" value="{}">
                <div class="form-group">
                    <label for="title">Title</label>
                    <input type="text" id="title" name="title" value="{}" required>
                </div>
                <div class="form-group">
                    <label for="file">File</label>
                    <input type="text" id="file" name="file" value="{}" required>
                </div>
                <div class="form-group">
                    <label for="prompt">Prompt</label>
                    <textarea id="prompt" name="prompt" rows="8" required>{}</textarea>
                </div>
                <div style="display: flex; flex-direction: rows;">
                    <button type="submit">Submit</button>
                    <span style="margin-right: 10px"></span>
                    <button id="button-deploy" hx-post="/deploy" hx-ext="json-enc" hx-trigger="click" hx-target="#response">Deploy</button>
                </div>
            </form>
    "##,
        key, fd.title, fd.file, fd.prompt
    );
    html
}
