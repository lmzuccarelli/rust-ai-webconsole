use crate::handlers::formdata::{
    Form, FormData, SearchData, deploy_formdata, render_form_html, render_results_html,
};
use crate::handlers::login::User;
use crate::handlers::common::get_map_item;
use chrono::Local;
use custom_logger as log;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use std::fs;

async fn get_index() -> Result<String, Box<dyn std::error::Error>> {
    let base_dir = get_map_item("static_dir".to_string())?;
    let html = fs::read_to_string(format!("{}/index.html",base_dir))?;
    Ok(html)
}

fn evaluate_login_params(data: Bytes) -> Result<(String, String), Box<dyn std::error::Error>> {
    let value = String::from_utf8(data.to_vec())?;
    let (user_res, password_res) = value.split_once("&").ok_or("could not parse parameters")?;
    let user = user_res.split("=").last().ok_or("could not parse user")?;
    let password = password_res
        .split("=")
        .last()
        .ok_or("could not parse password")?;
    Ok((user.to_string(), password.to_string()))
}

fn evaluate_formdata_params(data: Bytes) -> Result<FormData, Box<dyn std::error::Error>> {
    let mut fd: FormData = serde_json::from_slice(&data)?;
    match fd.key.clone() {
        Some(key) => {
            if key == "" {
                let now = Local::now();
                fd.key = Some(now.format("%Y%m%d%H%M%S").to_string());
            }
        }
        None => {
            let now = Local::now();
            fd.key = Some(now.format("%Y%m%d%H%M%S").to_string());
        }
    }
    Ok(fd)
}

fn evaluate_searchdata_params(data: Bytes) -> Result<SearchData, Box<dyn std::error::Error>> {
    let sd: SearchData = serde_json::from_slice(&data)?;
    Ok(sd)
}

fn get_formdata(req_uri: String) -> Result<String, Box<dyn std::error::Error>> {
    let key = req_uri.split("/").last().unwrap_or("no-key");
    let db_form = Form::new(key.to_string());
    let fd = db_form.db_read()?;
    let html = render_form_html(fd.clone().key.unwrap_or("".to_string()), fd);
    Ok(html)
}

// ai webconsole
pub async fn ai_service(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    log::debug!("request uri {}", req.uri());
    let req_uri = req.uri().to_string();
    match req.method() {
        &Method::GET => {
            // GET /index
            if req_uri.contains("index") {
                let result = get_index().await;
                match result {
                    Ok(html) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(html);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // GET /formdata/{key}
            if req_uri.contains("formdata") {
                let fd_res = get_formdata(req_uri);
                match fd_res {
                    Ok(html) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(html);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
        }

        &Method::POST => {
            let data = req.into_body().collect().await?.to_bytes();
            // POST /login
            if req_uri.contains("login") {
                let res = evaluate_login_params(data.clone());
                match res {
                    Ok(value) => {
                        let (user, password) = value;
                        let db_user = User::new(user);
                        let db_res = db_user.db_read(password);
                        match db_res {
                            Ok(_) => {
                                *response.status_mut() = StatusCode::OK;
                                *response.body_mut() = Full::from("ok".to_string());
                            }
                            Err(e) => {
                                *response.status_mut() = StatusCode::UNAUTHORIZED;
                                *response.body_mut() = Full::from(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /register
            if req_uri.contains("register") {
                let res = evaluate_login_params(data.clone());
                match res {
                    Ok(value) => {
                        let (user, password) = value;
                        let db_user = User::new(user);
                        let db_res = db_user.db_upsert(password, "".to_string());
                        match db_res {
                            Ok(_) => {
                                *response.status_mut() = StatusCode::OK;
                                *response.body_mut() = Full::from("ok".to_string());
                            }
                            Err(e) => {
                                *response.status_mut() = StatusCode::UNAUTHORIZED;
                                *response.body_mut() = Full::from(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /formdata
            if req_uri.contains("formdata") {
                let res = evaluate_formdata_params(data.clone());
                match res {
                    Ok(fd) => {
                        // this is safe as we update the key in the evaluate function
                        let form = Form::new(fd.clone().key.unwrap());
                        let result = form.db_upsert(fd);
                        match result {
                            Ok(res) => {
                                *response.status_mut() = StatusCode::OK;
                                *response.body_mut() = Full::from(res);
                            }
                            Err(e) => {
                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                *response.body_mut() = Full::from(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /search
            if req_uri.contains("search") {
                let res = evaluate_searchdata_params(data.clone());
                match res {
                    Ok(sd) => {
                        let form = Form::new("any".to_string());
                        let result = form.db_read_search(sd);
                        match result {
                            Ok(res) => {
                                let html = render_results_html(res);
                                *response.status_mut() = StatusCode::OK;
                                *response.body_mut() = Full::from(html);
                            }
                            Err(e) => {
                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                *response.body_mut() = Full::from(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /deploy
            if req_uri.contains("deploy") {
                let res = evaluate_formdata_params(data.clone());
                match res {
                    Ok(fd) => {
                        let result = deploy_formdata(fd.category, fd.file, data);
                        match result {
                            Ok(res) => {
                                *response.status_mut() = StatusCode::OK;
                                *response.body_mut() = Full::from(res);
                            }
                            Err(e) => {
                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                *response.body_mut() = Full::from(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
