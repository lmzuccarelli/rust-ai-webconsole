use crate::handlers::common::get_map_item;
use crate::handlers::formdata::Form;
use crate::handlers::interface::{InputformInterface, LoginformInterface, ViewformInterface};
use crate::handlers::login::User;
use crate::handlers::view::View;
use custom_logger as log;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::HeaderMap;
use hyper::body::{Bytes, Incoming};
use hyper::header::CONTENT_TYPE;
use std::fs;

async fn get_index() -> Result<String, Box<dyn std::error::Error>> {
    let base_dir = get_map_item("static_dir".to_string())?;
    let html = fs::read_to_string(format!("{}/index.html", base_dir))?;
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
                let fd_res = Form::get_formdata(req_uri.clone()).await;
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
            // GET /document for viewing /view/{key}
            if req_uri.contains("view") {
                let res = View::get_formdata(req_uri).await;
                match res {
                    Ok(doc) => {
                        let mut headers = HeaderMap::new();
                        //headers.insert(ACCEPT, "*".parse().unwrap());
                        headers.insert(CONTENT_TYPE, "text/markdown".parse().unwrap());
                        *response.headers_mut() = headers;
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(doc);
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
                let res = User::get_formdata(data.clone()).await;
                match res {
                    Ok(value) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(value);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /register
            if req_uri.contains("register") {
                let result = User::save_formdata(data.clone()).await;
                match result {
                    Ok(value) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(value);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /formdata
            if req_uri.contains("formdata") {
                let res = Form::save_formdata(data.clone()).await;
                match res {
                    Ok(fd) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(fd);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() = Full::from(e.to_string());
                    }
                }
            }
            // POST /search
            if req_uri.contains("search") {
                let result = Form::search_formdata(data.clone()).await;
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
            // POST /view
            if req_uri.contains("view") {
                let result = View::save_formdata(data.clone()).await;
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
        }
        &Method::DELETE => {
            // DELETE /formdata/{key}
            if req_uri.contains("formdata") {
                let fd_res = Form::delete_formdata(req_uri.clone()).await;
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

        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
