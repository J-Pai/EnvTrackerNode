use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
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
        path.push(std::format!("{}/public", root));
        path.push(req_path);
    }

    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("CONSOLE_NODE_ROOT").is_none() {
        std::env::set_var("CONSOLE_NODE_ROOT", "..");
    }

    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
