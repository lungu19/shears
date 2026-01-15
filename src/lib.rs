#![warn(clippy::all, rust_2018_idioms)]

pub use app::ShearsApp;

mod app;
mod helpers;
mod scan;
mod settings;
mod types;
