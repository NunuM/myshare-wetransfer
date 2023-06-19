#[macro_use]
extern crate log;


use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::error::ErrorInternalServerError;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::app::AppData;
use crate::auth_middleware::BasicAuth;
use crate::upload::DisplayDirectories;

mod app;
mod auth_middleware;
mod errors;
mod upload;
mod utils;
mod app_configs;
mod authenticator;

#[derive(Debug, Serialize)]
struct FileInfo {
    filename: String,
    created: String,
    size: u64,
}

async fn save_file(payload: Multipart, data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let manager = data.manager();

    let _ = manager.store(payload).await?;

    Ok(HttpResponse::SeeOther().header("Location", "/").finish())
}

#[derive(Debug, Deserialize)]
struct FilePath {
    file: String,
}

async fn download_file(
    path: web::Path<FilePath>,
    data: web::Data<AppData>,
) -> Result<NamedFile, Error> {
    data.manager()
        .get_file_from_link(path.file.as_str())
        .map_err(|e| e.into())
}

async fn index(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let context = Context::new();

    let index_content = data
        .templates()
        .render("home.html", &context)
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(index_content))
}

async fn list_files(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();

    context.insert(
        "entries",
        &DisplayDirectories::from(&data.manager().list_directory()?),
    );

    let index_content = data
        .templates()
        .render("files.html", &context)
        .map_err(|e| {
            error!("Error rendering files list: {:?}", e);

            actix_web::Error::from(
                HttpResponse::InternalServerError()
                    .set_header("Content-Type", "text/plain")
                    .body(e.to_string()),
            )
        })?;

    Ok(HttpResponse::Ok().body(index_content))
}

#[derive(Debug, Deserialize)]
struct LinkPath {
    link: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "fshare=debug,actix_web=info");
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    let application_configurations = app_configs::ApplicationConfigurations::from_config_file(
        args.get(1)
    ).expect(format!("Error loading application properties").as_str());

    let addr = format!("{}:{}",
                       application_configurations.server_configs().host(),
                       application_configurations.server_configs().port());

    debug!("Starting application with configs:{:?}", application_configurations);

    let log_format = application_configurations.server_configs().log_format().to_string();
    let number_of_threads = application_configurations.server_configs().number_thread() as usize;

    let auth_middleware = BasicAuth::new(application_configurations.server_configs().auth_strategy())
        .expect("Unable to starting authentication middleware");

    HttpServer::new(move || {
        App::new()
            .data(AppData::new(application_configurations.clone())
                .expect(format!("Error creating application properties").as_str()))
            .wrap(Logger::new(&log_format))
            .wrap(Compress::default())
            .service(
                web::resource("/")
                    .route(web::get().to(index))
                    .route(web::post().to(save_file)),
            )
            .service(
                web::resource("files")
                    .route(web::get().to(list_files))
                    .wrap(auth_middleware.clone()),
            )
            .route("/share/{file}", web::get().to(download_file))
            .service(actix_files::Files::new("/static", "static/"))
    })
        .workers(number_of_threads)
        .bind(addr)?
        .run()
        .await
}
