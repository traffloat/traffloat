use std::env;
use std::fs;
use std::path::PathBuf;

pub mod models;

pub fn main() {
    let dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let file = dir.join("models.rs");
    fs::write(file, models::write().to_string()).unwrap();
}
