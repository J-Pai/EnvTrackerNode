use std::{fs::File, io::BufReader};

use actix_web::{post, web, App, HttpServer, Responder, Result};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct EchoResp {
    echo: String,
}

#[derive(Deserialize, Debug)]
struct EchoReq {
    message: String,
}

#[post("/api/echo")]
async fn echo(req_body: web::Json<EchoReq>) -> Result<impl Responder> {
    println!("ECHO {:?}", req_body);

    let res = EchoResp {
        echo: std::format!("From Rust ~~~ {:}", req_body.message.to_string()),
    };

    println!("res {:?}", res);

    Ok(web::Json(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = load_rustls_config();

    HttpServer::new(|| App::new().service(echo))
        .bind_rustls_021(("localhost", 8443), config)?
        .run()
        .await
}

fn load_rustls_config() -> rustls::ServerConfig {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let cert_file = &mut BufReader::new(File::open("certificates/server.crt").unwrap());
    let key_file = &mut BufReader::new(File::open("certificates/server.key").unwrap());

    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
