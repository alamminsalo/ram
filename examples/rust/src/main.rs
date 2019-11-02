#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

mod models;
mod api;

fn main() {
    println!("Hello, world!");
}
