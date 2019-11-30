#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod api;
mod model;

fn main() {
    rocket::ignite().mount("/", api::routes()).launch();
}
