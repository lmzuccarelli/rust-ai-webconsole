use crate::certs::controller::{CertificateInterface, ImplCertificateInterface, error};
use crate::cli::schema::Cli;
use crate::config::process::{ConfigInterface, ImplConfigInterface, Parameters};
use crate::handlers::service::ai_service;
use clap::Parser;
use custom_logger as log;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls::ServerConfig;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

mod certs;
mod cli;
mod config;
mod handlers;

// used for lookup in read mode only
static MAP_LOOKUP: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn main() {
    // Serve an jwt auth service over HTTPS, with proper error handling.
    let args = Cli::parse();
    let config = args.config;
    let impl_config = ImplConfigInterface {};

    // setup logging
    log::Logging::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("log should initialize");

    // read and parse config
    let params = impl_config.read(config);
    if params.is_err() {
        log::error!("{}", params.err().unwrap());
        std::process::exit(1);
    }

    let level = match params.as_ref().unwrap().log_level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        &_ => log::LevelFilter::Info,
    };

    log::info!("application : {}", env!("CARGO_PKG_NAME"));
    log::info!("author      : {}", env!("CARGO_PKG_AUTHORS"));
    log::info!("version     : {}", env!("CARGO_PKG_VERSION"));

    // override level if other than info
    if level == log::LevelFilter::Debug || level == log::LevelFilter::Trace {
        let _ = log::Logging::new().with_level(level).init();
    }

    // set up for (read) reference in auth handler
    let mut hm = HashMap::new();
    hm.insert(
        "db_path".to_string(),
        params.as_ref().unwrap().db_path.to_string(),
    );
    hm.insert(
        "deploy_dir".to_string(),
        params.as_ref().unwrap().deploy_dir.to_string(),
    );
    hm.insert(
        "static_dir".to_string(),
        params.as_ref().unwrap().static_dir.to_string(),
    );

    *MAP_LOOKUP.lock().unwrap() = Some(hm.clone());

    if let Err(e) = run_server(params.unwrap()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}

#[tokio::main]
async fn run_server(params: Parameters) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), params.port.parse()?);
    let certs_dir = params.certs_dir.unwrap_or("".to_string()).to_string();
    log::debug!("certs directory {}", certs_dir);
    let impl_certs = ImplCertificateInterface::new(params.cert_mode, Some(certs_dir));
    // Load public certificate.
    let certs = impl_certs.get_public_cert().await?;
    // Load private key.
    let key = impl_certs.get_private_cert().await?;
    log::info!("starting {} on https://{}", params.name, addr);
    // Create a TCP listener via tokio.
    let incoming = TcpListener::bind(&addr).await?;
    // Build TLS configuration.
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| error(e.to_string()))?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));
    let service = service_fn(ai_service);

    loop {
        let (tcp_stream, _remote_addr) = incoming.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => tls_stream,
                Err(err) => {
                    log::error!("failed to perform tls handshake: {err:#}");
                    return;
                }
            };
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(tls_stream), service)
                .await
            {
                log::error!("failed to serve connection: {err:#}");
            }
        });
    }
}
