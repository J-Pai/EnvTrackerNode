use actix_files::NamedFile;
use actix_web::{post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let root = std::env::var_os("CONSOLE_NODE_ROOT")
        .unwrap()
        .into_string()
        .unwrap();
    let mut path: PathBuf = PathBuf::from(std::format!("{}/.next", root));
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();

    if req_path == PathBuf::from("") {
        path.push("server/app/index.html");
    } else if req_path.starts_with(PathBuf::from("_next/static")) {
        path.push(req_path.strip_prefix("_next").unwrap());
    } else if req_path.parent().unwrap() == PathBuf::from("") {
        path.clear();
        path.push(root);
        path.push("public");
        path.push(req_path);
    }

    println!("index: {}", path.display());

    Ok(NamedFile::open(path)?)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("ECHO {req_body}");
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("CONSOLE_NODE_ROOT").is_none() {
        std::env::set_var("CONSOLE_NODE_ROOT", "../console_node");
    }

    HttpServer::new(|| {
        App::new()
            .service(echo)
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
