use adw::prelude::*;

use crate::ui::main_window;

const APP_ID: &str = "org.tailsos.simplepgp";

pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        main_window::build_main_window(app);
    });

    app.run()
}
