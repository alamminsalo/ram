mod api;
mod model;

use actix_web::{App, HttpServer};

fn main() {
    HttpServer::new(|| App::new().service(api::routes()))
        .bind("127.0.0.1:8000")
        .expect("Can not bind to port 8000")
        .run()
        .unwrap();
}
