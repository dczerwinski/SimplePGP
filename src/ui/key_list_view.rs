use std::rc::Rc;

use adw::prelude::*;

use crate::ui::dialogs;
use crate::viewmodels::KeyListViewModel;

/// Builds the "Keys" tab content.
pub fn build_key_list_view(
    vm: &Rc<KeyListViewModel>,
    window: &adw::ApplicationWindow,
) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let clamp = adw::Clamp::builder()
        .maximum_size(800)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let inner = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // --- toolbar: refresh + import ---
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
        .icon_name("list-add-symbolic")
        .tooltip_text("Import an ASCII-armored PGP key")
        .css_classes(["suggested-action"])
        .build();

    toolbar.append(&refresh_btn);
    toolbar.append(&import_btn);

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
    container.append(&clamp);

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

    // --- connect selection ---
    {
        let vm = Rc::clone(vm);
        list_box.connect_row_selected(move |_, row| {
            let idx = row.map(|r| r.index() as usize);
            vm.select_key(idx);
        });
    }

    // --- subscribe to state changes ---
    {
        let list_box = list_box.clone();
        let spinner = spinner.clone();
        let window = window.clone();

        vm.subscribe(Box::new(move |state| {
            spinner.set_visible(state.loading);
            spinner.set_spinning(state.loading);

            // rebuild list
            while let Some(row) = list_box.row_at_index(0) {
                list_box.remove(&row);
            }

            for key in &state.keys {
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

                list_box.append(&row);
            }

            if let Some(ref err) = state.error {
                dialogs::show_error_dialog(&window, err);
            }

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
