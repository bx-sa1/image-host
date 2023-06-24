use std::{sync::atomic::{AtomicU8, Ordering}, path::Path};

use actix_multipart::{form::{MultipartForm, bytes::Bytes}};
use actix_web::{get, post, web::{self}, App, HttpServer, Responder, HttpResponse, http::header, Error};

#[derive(MultipartForm)]
struct ImageForm {
    image: Bytes
}

fn get_id() -> u8 { 
    static COUNTER: AtomicU8 = AtomicU8::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[post("/")]
async fn save_image(MultipartForm(form): MultipartForm<ImageForm>) -> Result<impl Responder, Error> {
    let file_name = form.image.file_name.unwrap();
    let file_path = Path::new(file_name.as_str());
    let id = get_id();
    let full_file_name = format!("{}.{}", id, file_path.extension().unwrap().to_str().unwrap());

    if let Err(e) = std::fs::write(&full_file_name, form.image.data) {
        return Ok(HttpResponse::InternalServerError().body(e.to_string()));
    }

    Ok(HttpResponse::Ok().body(full_file_name))
}

#[get("/{name}")]
async fn load_image(name: web::Path<String>) -> impl Responder {
    let name = name.to_string();
    let path = Path::new(&name);

    let content_type = match path.extension().unwrap().to_str().unwrap() {
        "jpg" | "jpeg" => header::ContentType::jpeg(),
        "png" => header::ContentType::png(),
        _ => { return HttpResponse::NotFound().finish(); }
    };

    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    HttpResponse::Ok().content_type(content_type).body(data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .service(save_image)
        .service(load_image)
    })
    .bind(("127.0.0.1", 7878))?
    .run()
    .await
}