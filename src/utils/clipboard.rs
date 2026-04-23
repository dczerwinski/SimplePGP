use gtk::gdk;

/// Copies text to the system clipboard via GDK.
pub fn copy_to_clipboard(text: &str) {
    if let Some(display) = gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(text);
    }
}
