use adw::prelude::*;

use crate::ui::main_window;

pub const APP_ID: &str = "org.tailsos.simplepgp";

pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        register_icon_search_paths();
    });

    app.connect_activate(|app| {
        main_window::build_main_window(app);
    });

    app.run()
}

/// Teach the default GTK icon theme where our bundled icons live.
///
/// On a proper system install icons end up under `/usr/share/icons/hicolor/...`
/// and are picked up automatically. For `cargo run` during development and for
/// portable Windows builds, we also search next to the executable and next to
/// `Cargo.toml` so the app icon shows up without a full system install.
fn register_icon_search_paths() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    let theme = gtk::IconTheme::for_display(&display);

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("data").join("icons"));
            candidates.push(dir.join("icons"));
        }
    }
    candidates.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/icons"));

    for path in candidates {
        if path.is_dir() {
            theme.add_search_path(&path);
        }
    }
}
