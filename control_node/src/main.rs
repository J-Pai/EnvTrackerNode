use actix_web::{post, web, App, HttpServer, Result, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
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

    let resp = EchoResp {
        echo: req_body.message.to_string(),
    };

    Ok(web::Json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("CONSOLE_NODE_ROOT").is_none() {
        std::env::set_var("CONSOLE_NODE_ROOT", "../console_node");
    }

    HttpServer::new(|| App::new().service(echo))
        .bind(("localhost", 8080))?
        .run()
        .await
}
