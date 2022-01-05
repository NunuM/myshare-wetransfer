use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use tera::{Tera, Context};
use actix_web::error::ErrorInternalServerError;
use serde::{Serialize, Deserialize};
use actix_files::NamedFile;
use zip::write::FileOptions;
use actix_web::web::Buf;

const BASE_DIR: &str = "./tmp";

mod utils;

#[derive(Debug, Serialize)]
struct FileInfo {
    filename: String,
    created: String,
    size: u64,
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {

    let mut f = web::block(|| std::fs::File::create(format!("{}/m.zip", BASE_DIR)))
        .await
        .unwrap();

    let mut zipper = zip::ZipWriter::new(f);

    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();

        zipper.start_file(filename, options).unwrap();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();

            zipper = web::block(move || zipper.write_all(data.bytes()).map(|_| zipper)).await?;
        }
    }

    zipper.finish().unwrap();

    Ok(HttpResponse::SeeOther().header("Location", "/").finish())
}

#[derive(Debug, Deserialize)]
struct FileQuery {
    file: String
}

async fn download_file(query: web::Query<FileQuery>) -> Result<NamedFile, Error> {
    Ok(NamedFile::open(format!("{}/{}", BASE_DIR, sanitize_filename::sanitize(&query.file))).unwrap())
}


async fn index(data: web::Data<Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();

    let mut files = Vec::new();

    for result_entry in std::fs::read_dir(BASE_DIR).unwrap() {
        let entry = result_entry.unwrap();

        let filename = entry.file_name().into_string().unwrap();

        let file_mata = entry.metadata().unwrap();

        let size = file_mata.len();
        let created = file_mata.created().unwrap();


        files.push(FileInfo {
            filename,
            created: format!("{:?}", created.elapsed().unwrap().as_secs()),
            size,
        })
    }

    context.insert("files", &files);

    let index_content = data
        .render("index.html", &context)
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(index_content))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all(BASE_DIR).unwrap();

    let ip = "0.0.0.0:3000";


    HttpServer::new(|| {
        App::new()
            .data(match Tera::new("templates/**/*") {
                Ok(t) => t,
                Err(e) => {
                    println!("Parsing error(s): {}", e);
                    ::std::process::exit(1);
                }
            })
            .service(
                web::resource("/")
                    .route(web::get().to(index))
                    .route(web::post().to(save_file))
            ).route("/download", web::get().to(download_file))
    })
        .bind(ip)?
        .run()
        .await
}
