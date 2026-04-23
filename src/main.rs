mod app;
mod models;
mod security;
mod services;
mod ui;
mod utils;
mod viewmodels;

fn main() -> glib::ExitCode {
    app::run()
}
