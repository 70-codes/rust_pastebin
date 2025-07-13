use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::response::content::{self, RawText};
use rocket::tokio::fs::File;

use std::fs;
use std::path::Path;

#[macro_use]
extern crate rocket;

#[get("/pastes/<id>")]
async fn paste(id: String) -> Option<content::RawText<String>> {
    let path = format!("uploads/{}", id);
    let content = rocket::tokio::fs::read_to_string(&path).await.ok()?;

    Some(RawText(content))
}

#[post("/upload", data = "<paste>")]
async fn upload(paste: Data<'_>) -> Result<String, Status> {
    let id = nanoid::nanoid!(8);
    let path = format!("uploads/{}", id);

    if !Path::new("uploads").exists() {
        fs::create_dir("uploads").map_err(|_| Status::InternalServerError)?;
    }
    let mut file = File::create(&path)
        .await
        .map_err(|_| Status::InternalServerError)?;

    paste
        .open(32.kilobytes())
        .stream_to(&mut file)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(format!("http://localhost:8000/pastes/{}", &id))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![paste, upload])
}
