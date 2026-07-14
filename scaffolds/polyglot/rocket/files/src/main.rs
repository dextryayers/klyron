#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate serde;

use rocket::serde::json::Json;
use rocket::serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: String,
    service: String,
    version: String,
}

#[get("/api/health")]
fn health() -> Json<Health> {
    Json(Health {
        status: "ok".to_string(),
        service: "{{ name }}".to_string(),
        version: "{{ version }}".to_string(),
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![health])
}
