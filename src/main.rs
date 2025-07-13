use rocket::data::{Data, ToByteUnit};
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::content::{self};
use rocket::{Build, Rocket, State};
use tokio::io::AsyncReadExt;

use serde::{Deserialize, Serialize};

use sqlx::SqlitePool;

#[macro_use]
extern crate rocket;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct Paste {
    id: String,
    content: String,
}

async fn run_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let pool = SqlitePool::connect("sqlite://pastes.db").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    Ok(rocket.manage(pool))
}

#[get("/pastes/<id>")]
async fn show_paste(id: String, db: &State<SqlitePool>) -> Option<content::RawText<String>> {
    let paste = sqlx::query_as::<_, Paste>("SELECT id, content FROM pastes WHERE id = ?")
        .bind(&id)
        .fetch_one(db.inner())
        .await
        .ok()?;
    Some(content::RawText(paste.content))
}

#[post("/upload", data = "<paste>")]
async fn upload(paste: Data<'_>, db: &State<SqlitePool>) -> Result<String, Status> {
    let id = nanoid::nanoid!(8);
    let mut content = String::new();
    let mut stream = paste.open(32.kilobytes());
    stream
        .read_to_string(&mut content)
        .await
        .map_err(|_| Status::BadRequest)?;

    sqlx::query("INSERT INTO pastes (id, content) VALUES (?, ?)")
        .bind(&id)
        .bind(&content)
        .execute(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(format!("http://localhost:8000/pastes/{}", &id))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![show_paste, upload])
}
