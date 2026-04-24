use adw::prelude::*;
use gtk::gio;

use crate::app::APP_ID;
use crate::ui::{about, decrypt_view, encrypt_view, key_list_view};
use crate::viewmodels::{CryptoViewModel, KeyListViewModel};

pub fn build_main_window(app: &adw::Application) {
    let key_vm = KeyListViewModel::new();
    let crypto_vm = CryptoViewModel::new();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("SimplePGP")
        .default_width(900)
        .default_height(700)
        .icon_name(APP_ID)
        .build();

    // --- header bar ---
    let header = adw::HeaderBar::new();

    let title_widget = adw::WindowTitle::builder()
        .title("SimplePGP")
        .subtitle("Privacy-focused key management")
        .build();
    header.set_title_widget(Some(&title_widget));

    // --- primary menu with About entry ---
    let menu = gio::Menu::new();
    menu.append(Some("About SimplePGP"), Some("win.about"));

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text("Main Menu")
        .build();
    header.pack_end(&menu_button);

    let about_action = gio::ActionEntry::builder("about")
        .activate({
            let win = window.clone();
            move |_, _, _| about::show_about_dialog(&win)
        })
        .build();
    window.add_action_entries([about_action]);

    // --- view stack + switcher ---
    let stack = adw::ViewStack::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let keys_page = stack.add_titled(
        &key_list_view::build_key_list_view(&key_vm, &window),
        Some("keys"),
        "Keys",
    );
    keys_page.set_icon_name(Some("system-users-symbolic"));

    let encrypt_page = stack.add_titled(
        &encrypt_view::build_encrypt_view(&crypto_vm, &key_vm),
        Some("encrypt"),
        "Encrypt",
    );
    encrypt_page.set_icon_name(Some("channel-secure-symbolic"));

    let decrypt_page = stack.add_titled(
        &decrypt_view::build_decrypt_view(&crypto_vm),
        Some("decrypt"),
        "Decrypt",
    );
    decrypt_page.set_icon_name(Some("channel-insecure-symbolic"));

    let view_switcher = adw::ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();
    header.set_title_widget(Some(&view_switcher));

    // --- assemble window ---
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&stack);

    window.set_content(Some(&content));
    window.present();

    // Initial key load
    key_vm.load_keys();

    // Wire error display for crypto operations
    {
        let win = window.clone();
        crypto_vm.subscribe(Box::new(move |state| {
            if let Some(ref err) = state.error {
                crate::ui::dialogs::show_error_dialog(&win, err);
            }
        }));
    }
}
