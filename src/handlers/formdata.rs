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
    pub category: String,
    pub prompt: String,
    pub credentials: String,
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
        log::debug!("[db_upsert] formdata {}", json_data);
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
                    category: "".to_string(),
                    prompt: "".to_string(),
                    credentials: "".to_string(),
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

pub fn deploy_formdata(
    category: String,
    name: String,
    data: Bytes,
) -> Result<String, Box<dyn std::error::Error>> {
    let deploy_dir = get_map_item("deploy_dir".to_string())?;
    let dir = format!("{}/{}", deploy_dir, category);
    fs::create_dir_all(dir.clone())?;
    fs::write(format!("{}/{}", dir, name.replace("md", "json")), data)?;
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
            <td>{}</td>
            <td><i class=\"fa fa-trash-o\" hx-post=\"/webconsole/delete/{}\" hx-trigger=\"click\"></i>&nbsp&nbsp;&nbsp;<i id=\"icon-formdata\" class=\"fa fa-edit\" hx-get=\"/webconsole/formdata/{}\" hx-target=\"#inputForm\" hx-trigger=\"click\"></i></td>
        </tr>",
            key, fd.file, fd.category, fd.title,  fd.prompt,key,key
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
                <input type="hidden" id="credentials" name="credentials" value="{}">
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
                <div style="display: flex; flex-direction: rows;">
                    <button type="submit">Submit</button>
                    <span style="margin-right: 10px"></span>
                    <button id="button-deploy" hx-post="/webconsole/deploy" hx-ext="json-enc" hx-trigger="click" hx-target="#response">Deploy</button>
                </div>
            </form>
    "##,
        key,
        fd.credentials,
        fd.title,
        check_selected(fd.category),
        fd.file,
        fd.prompt
    );
    html
}

fn check_selected(category: String) -> String {
    let mut vec_selected = vec!["", "", "", "", ""];
    match category.as_str() {
        "hobby" => {
            vec_selected[0] = "selected";
        }
        "finance" => {
            vec_selected[1] = "selected";
        }
        "projects" => {
            vec_selected[2] = "selected";
        }
        "programming" => {
            vec_selected[3] = "selected";
        }
        "sw-architecture" => {
            vec_selected[4] = "selected";
        }
        _ => {
            vec_selected[0] = "selected";
        }
    };

    let result = format!(
        r#"
    <option id="hobby" name="hobby" {}>hobby</option>
    <option id="finance" name="finance" {}>finance</option>
    <option id="projects" name="projects" {}>projects</option>
    <option id="programming" name="programming" {}>programming</option>
    <option id="sw-architecture" name="sw-architecture" {}>sw-architecture</option>
    "#,
        vec_selected[0], vec_selected[1], vec_selected[2], vec_selected[3], vec_selected[4]
    );
    result
}
