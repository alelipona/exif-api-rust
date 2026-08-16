mod handlers;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    log::info!("🚀 Starting EXIF API on {}:{}", host, port);
    log::info!("📦 exiftool-rs version: {}", exiftool_rs::VERSION);

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header().max_age(3600))
            .route("/health", web::get().to(handlers::health_check))
            .route("/read", web::post().to(handlers::read_metadata))
            .route("/write", web::post().to(handlers::write_metadata))
            .route("/delete", web::post().to(handlers::delete_metadata))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
