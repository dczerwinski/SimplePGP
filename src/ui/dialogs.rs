use adw::prelude::*;

/// Shows a simple informational dialog with the given title and message.
pub fn show_info_dialog(parent: &impl IsA<gtk::Widget>, title: &str, message: &str) {
    let dialog = adw::AlertDialog::builder()
        .heading(title)
        .body(message)
        .build();
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(parent));
}

/// Shows an error dialog.
pub fn show_error_dialog(parent: &impl IsA<gtk::Widget>, message: &str) {
    show_info_dialog(parent, "Error", message);
}

/// Shows a success dialog.
pub fn show_success_dialog(parent: &impl IsA<gtk::Widget>, message: &str) {
    show_info_dialog(parent, "Success", message);
}
