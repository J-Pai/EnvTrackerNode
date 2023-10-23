use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct EchoResp {
    echo: String,
}

#[post("/api/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("ECHO {req_body}");

    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("CONSOLE_NODE_ROOT").is_none() {
        std::env::set_var("CONSOLE_NODE_ROOT", "../console_node");
    }

    HttpServer::new(|| App::new().service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
