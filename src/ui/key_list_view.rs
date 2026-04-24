use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::services::{KeyAlgorithm, KeyGenParams};
use crate::ui::dialogs;
use crate::viewmodels::KeyListViewModel;

/// Builds the "Keys" tab content.
pub fn build_key_list_view(
    vm: &Rc<KeyListViewModel>,
    window: &adw::ApplicationWindow,
) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder()
        .maximum_size(1400)
        .tightening_threshold(800)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    let inner = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    // --- toolbar: refresh + import + generate ---
    let toolbar = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let refresh_btn = gtk::Button::builder()
        .label("Refresh")
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Reload keys from GnuPG keyring")
        .css_classes(["flat"])
        .build();

    let import_btn = gtk::Button::builder()
        .label("Import Key")
        .icon_name("document-open-symbolic")
        .tooltip_text("Import an ASCII-armored PGP key")
        .build();

    let generate_btn = gtk::Button::builder()
        .label("Generate Key")
        .icon_name("list-add-symbolic")
        .tooltip_text("Generate a new PGP key pair")
        .css_classes(["suggested-action"])
        .build();

    toolbar.append(&refresh_btn);
    toolbar.append(&import_btn);
    toolbar.append(&generate_btn);

    // --- loading spinner ---
    let spinner = gtk::Spinner::builder()
        .spinning(false)
        .halign(gtk::Align::Center)
        .visible(false)
        .build();

    // --- key list ---
    let list_group = adw::PreferencesGroup::builder()
        .title("PGP Keys")
        .description("Keys from your GnuPG keyring")
        .build();

    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .css_classes(["boxed-list"])
        .build();

    list_group.add(&list_box);

    inner.append(&toolbar);
    inner.append(&spinner);
    inner.append(&list_group);
    clamp.set_child(Some(&inner));

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .propagate_natural_height(false)
        .child(&clamp)
        .build();

    container.append(&scrolled);

    // --- connect refresh button ---
    {
        let vm = Rc::clone(vm);
        refresh_btn.connect_clicked(move |_| {
            vm.load_keys();
        });
    }

    // --- connect import button ---
    {
        let vm = Rc::clone(vm);
        let win = window.clone();
        import_btn.connect_clicked(move |_| {
            show_import_dialog(&win, &vm);
        });
    }

    // --- connect generate button ---
    {
        let vm = Rc::clone(vm);
        let win = window.clone();
        generate_btn.connect_clicked(move |_| {
            show_generate_dialog(&win, &vm);
        });
    }

    // Shared flag to suppress `row-selected` handling while the list is
    // being rebuilt from a state update. Without it, `list_box.remove()`
    // and `list_box.select_row()` emit spurious selection events that
    // re-enter the VM, re-notify, and eventually overflow the stack.
    let rebuilding: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    // --- connect selection ---
    {
        let vm = Rc::clone(vm);
        let rebuilding = Rc::clone(&rebuilding);
        list_box.connect_row_selected(move |_, row| {
            if *rebuilding.borrow() {
                return;
            }
            let idx = row.map(|r| r.index() as usize);
            vm.select_key(idx);
        });
    }

    // --- subscribe to state changes ---
    {
        let list_box = list_box.clone();
        let spinner = spinner.clone();
        let window = window.clone();
        let vm_ref = Rc::clone(vm);
        let rebuilding = Rc::clone(&rebuilding);

        // Track which result/error strings have already been shown so we
        // don't pop the same dialog on every state notification.
        let last_error: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let last_import: Rc<RefCell<Option<Result<String, String>>>> =
            Rc::new(RefCell::new(None));
        let last_generate: Rc<RefCell<Option<Result<String, String>>>> =
            Rc::new(RefCell::new(None));
        let last_delete: Rc<RefCell<Option<Result<String, String>>>> =
            Rc::new(RefCell::new(None));

        vm.subscribe(Box::new(move |state| {
            spinner.set_visible(state.loading);
            spinner.set_spinning(state.loading);

            *rebuilding.borrow_mut() = true;

            while let Some(row) = list_box.row_at_index(0) {
                list_box.remove(&row);
            }

            for (idx, key) in state.keys.iter().enumerate() {
                let row = adw::ActionRow::builder()
                    .title(&*glib::markup_escape_text(&key.display_name()))
                    .subtitle(&format!(
                        "{} · {} {}bit · {}",
                        key.short_id(),
                        key.algorithm,
                        key.key_length,
                        key.trust
                    ))
                    .build();

                if key.has_secret {
                    let badge = gtk::Label::builder()
                        .label("SECRET")
                        .css_classes(["caption", "accent"])
                        .valign(gtk::Align::Center)
                        .build();
                    row.add_suffix(&badge);
                }

                let delete_btn = gtk::Button::builder()
                    .icon_name("user-trash-symbolic")
                    .tooltip_text("Delete this key")
                    .valign(gtk::Align::Center)
                    .css_classes(["flat", "circular"])
                    .build();

                {
                    let vm = Rc::clone(&vm_ref);
                    let win = window.clone();
                    let fingerprint = if key.fingerprint.is_empty() {
                        key.key_id.clone()
                    } else {
                        key.fingerprint.clone()
                    };
                    let display = key.display_name();
                    let has_secret = key.has_secret;
                    delete_btn.connect_clicked(move |_| {
                        show_delete_confirm(
                            &win,
                            &vm,
                            fingerprint.clone(),
                            display.clone(),
                            has_secret,
                        );
                    });
                }

                row.add_suffix(&delete_btn);

                list_box.append(&row);

                if state.selected_index == Some(idx) {
                    list_box.select_row(Some(&row));
                }
            }

            *rebuilding.borrow_mut() = false;

            // --- one-shot notifications ---
            if state.error != *last_error.borrow() {
                if let Some(ref err) = state.error {
                    dialogs::show_error_dialog(&window, err);
                }
                *last_error.borrow_mut() = state.error.clone();
            }

            if state.import_result != *last_import.borrow() {
                if let Some(ref import_res) = state.import_result {
                    match import_res {
                        Ok(msg) => dialogs::show_success_dialog(
                            &window,
                            &format!("Key imported successfully.\n\n{}", msg),
                        ),
                        Err(msg) => dialogs::show_error_dialog(
                            &window,
                            &format!("Import failed:\n{}", msg),
                        ),
                    }
                }
                *last_import.borrow_mut() = state.import_result.clone();
            }

            if state.generate_result != *last_generate.borrow() {
                if let Some(ref gen_res) = state.generate_result {
                    match gen_res {
                        Ok(msg) => {
                            let body = if msg.trim().is_empty() {
                                "Your new PGP key has been created.".to_string()
                            } else {
                                format!("Your new PGP key has been created.\n\n{}", msg)
                            };
                            dialogs::show_success_dialog(&window, &body);
                        }
                        Err(msg) => dialogs::show_error_dialog(
                            &window,
                            &format!("Key generation failed:\n{}", msg),
                        ),
                    }
                }
                *last_generate.borrow_mut() = state.generate_result.clone();
            }

            if state.delete_result != *last_delete.borrow() {
                if let Some(ref del_res) = state.delete_result {
                    match del_res {
                        Ok(msg) => dialogs::show_success_dialog(&window, msg),
                        Err(msg) => dialogs::show_error_dialog(
                            &window,
                            &format!("Delete failed:\n{}", msg),
                        ),
                    }
                }
                *last_delete.borrow_mut() = state.delete_result.clone();
            }
        }));
    }

    container
}

fn show_import_dialog(window: &adw::ApplicationWindow, vm: &Rc<KeyListViewModel>) {
    let dialog = adw::AlertDialog::builder()
        .heading("Import PGP Key")
        .body("Paste an ASCII-armored PGP key below:")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("import", "Import");
    dialog.set_response_appearance("import", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("import"));
    dialog.set_close_response("cancel");

    let scrolled = gtk::ScrolledWindow::builder()
        .min_content_height(200)
        .min_content_width(500)
        .build();

    let text_view = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();

    scrolled.set_child(Some(&text_view));
    dialog.set_extra_child(Some(&scrolled));

    let vm = Rc::clone(vm);
    let buffer = text_view.buffer();

    dialog.connect_response(None, move |_dialog, response| {
        if response == "import" {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            let key_text = text.to_string();
            if !key_text.trim().is_empty() {
                vm.import_key(key_text);
            }
        }
    });

    dialog.present(Some(window));
}

fn show_generate_dialog(window: &adw::ApplicationWindow, vm: &Rc<KeyListViewModel>) {
    let dialog = adw::AlertDialog::builder()
        .heading("Generate New PGP Key")
        .body("Fill in the identity and key parameters below.")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("generate", "Generate");
    dialog.set_response_appearance("generate", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("generate"));
    dialog.set_close_response("cancel");

    let form = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .width_request(440)
        .build();

    let group = adw::PreferencesGroup::new();

    let name_row = adw::EntryRow::builder().title("Full name").build();
    let email_row = adw::EntryRow::builder().title("Email (optional)").build();
    let comment_row = adw::EntryRow::builder()
        .title("Comment (optional)")
        .build();

    let algo_row = adw::ComboRow::builder().title("Algorithm").build();
    let algo_model = gtk::StringList::new(&["Ed25519 (recommended)", "RSA 4096", "RSA 2048"]);
    algo_row.set_model(Some(&algo_model));
    algo_row.set_selected(0);

    let expire_row = adw::ComboRow::builder().title("Expiration").build();
    let expire_model = gtk::StringList::new(&[
        "Never",
        "1 year",
        "2 years",
        "5 years",
        "6 months",
    ]);
    expire_row.set_model(Some(&expire_model));
    expire_row.set_selected(1);

    let pass_row = adw::PasswordEntryRow::builder()
        .title("Passphrase (empty = no protection)")
        .build();
    let pass_confirm_row = adw::PasswordEntryRow::builder()
        .title("Confirm passphrase")
        .build();

    group.add(&name_row);
    group.add(&email_row);
    group.add(&comment_row);
    group.add(&algo_row);
    group.add(&expire_row);
    group.add(&pass_row);
    group.add(&pass_confirm_row);

    form.append(&group);
    dialog.set_extra_child(Some(&form));

    let vm = Rc::clone(vm);
    let window_weak = window.downgrade();

    dialog.connect_response(None, move |_dialog, response| {
        if response != "generate" {
            return;
        }

        let name = name_row.text().to_string();
        let email = email_row.text().to_string();
        let comment = comment_row.text().to_string();
        let passphrase = pass_row.text().to_string();
        let confirm = pass_confirm_row.text().to_string();

        let (algorithm, key_length) = match algo_row.selected() {
            0 => (KeyAlgorithm::Ed25519, 0),
            1 => (KeyAlgorithm::Rsa, 4096),
            _ => (KeyAlgorithm::Rsa, 2048),
        };

        let expire = match expire_row.selected() {
            0 => "0",
            1 => "1y",
            2 => "2y",
            3 => "5y",
            4 => "6m",
            _ => "0",
        }
        .to_string();

        if name.trim().is_empty() {
            if let Some(win) = window_weak.upgrade() {
                dialogs::show_error_dialog(
                    &win,
                    "Name is required to generate a key.",
                );
            }
            return;
        }

        if passphrase != confirm {
            if let Some(win) = window_weak.upgrade() {
                dialogs::show_error_dialog(&win, "Passphrases do not match.");
            }
            return;
        }

        let params = KeyGenParams {
            name,
            email,
            comment,
            algorithm,
            key_length,
            expire,
            passphrase,
        };

        vm.generate_key(params);
    });

    dialog.present(Some(window));
}

fn show_delete_confirm(
    window: &adw::ApplicationWindow,
    vm: &Rc<KeyListViewModel>,
    fingerprint: String,
    display_name: String,
    has_secret: bool,
) {
    let body = if has_secret {
        format!(
            "You are about to permanently delete the key:\n\n{}\n\nThis key has a SECRET component. \
             Deleting it will destroy your ability to decrypt messages or sign with this identity.\n\n\
             This operation cannot be undone.",
            display_name
        )
    } else {
        format!(
            "You are about to permanently delete the public key:\n\n{}\n\nThis operation cannot be undone.",
            display_name
        )
    };

    let dialog = adw::AlertDialog::builder()
        .heading("Delete key?")
        .body(&body)
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let vm = Rc::clone(vm);
    dialog.connect_response(None, move |_dialog, response| {
        if response == "delete" {
            vm.delete_key(fingerprint.clone(), has_secret);
        }
    });

    dialog.present(Some(window));
}
