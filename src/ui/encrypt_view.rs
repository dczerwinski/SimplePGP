use std::rc::Rc;

use adw::prelude::*;
use gtk::prelude::*;

use crate::utils::copy_to_clipboard;
use crate::viewmodels::{CryptoViewModel, KeyListViewModel};

/// Builds the "Encrypt" tab content.
pub fn build_encrypt_view(
    crypto_vm: &Rc<CryptoViewModel>,
    key_vm: &Rc<KeyListViewModel>,
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

    // --- recipient selector ---
    let recipient_group = adw::PreferencesGroup::builder()
        .title("Recipient")
        .build();

    let recipient_dropdown = gtk::DropDown::from_strings(&["(select a key)"]);
    recipient_dropdown.set_sensitive(false);

    let recipient_row = adw::ActionRow::builder()
        .title("Encrypt to")
        .build();
    recipient_row.add_suffix(&recipient_dropdown);
    recipient_group.add(&recipient_row);

    // --- plaintext input ---
    let input_group = adw::PreferencesGroup::builder()
        .title("Plaintext")
        .description("Enter the text you want to encrypt")
        .build();

    let input_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(120)
        .build();

    let input_text = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    input_scroll.set_child(Some(&input_text));
    input_group.add(&input_scroll);

    // --- action buttons ---
    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();

    let encrypt_btn = gtk::Button::builder()
        .label("Encrypt")
        .css_classes(["suggested-action"])
        .build();

    let clear_btn = gtk::Button::builder()
        .label("Clear")
        .css_classes(["flat"])
        .build();

    btn_box.append(&clear_btn);
    btn_box.append(&encrypt_btn);

    // --- spinner ---
    let spinner = gtk::Spinner::builder()
        .spinning(false)
        .halign(gtk::Align::Center)
        .visible(false)
        .build();

    // --- output ---
    let output_group = adw::PreferencesGroup::builder()
        .title("Encrypted Output")
        .build();

    let output_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(120)
        .build();

    let output_text = gtk::TextView::builder()
        .monospace(true)
        .editable(false)
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .css_classes(["dim-label"])
        .build();
    output_scroll.set_child(Some(&output_text));
    output_group.add(&output_scroll);

    let copy_btn = gtk::Button::builder()
        .label("Copy to Clipboard")
        .icon_name("edit-copy-symbolic")
        .css_classes(["flat"])
        .sensitive(false)
        .build();
    output_group.add(&copy_btn);

    inner.append(&recipient_group);
    inner.append(&input_group);
    inner.append(&btn_box);
    inner.append(&spinner);
    inner.append(&output_group);
    clamp.set_child(Some(&inner));
    container.append(&clamp);

    // --- populate dropdown from key_vm ---
    let key_ids: Rc<std::cell::RefCell<Vec<String>>> =
        Rc::new(std::cell::RefCell::new(Vec::new()));

    {
        let dropdown = recipient_dropdown.clone();
        let key_ids = Rc::clone(&key_ids);

        key_vm.subscribe(Box::new(move |state| {
            let model = gtk::StringList::new(&[]);
            let mut ids = Vec::new();

            if state.keys.is_empty() {
                model.append("(no keys available)");
            } else {
                for key in &state.keys {
                    model.append(&format!("{} ({})", key.display_name(), key.short_id()));
                    ids.push(key.key_id.clone());
                }
            }

            dropdown.set_model(Some(&model));
            dropdown.set_sensitive(!state.keys.is_empty());
            dropdown.set_selected(0);
            *key_ids.borrow_mut() = ids;
        }));
    }

    // --- encrypt button ---
    {
        let cvm = Rc::clone(crypto_vm);
        let input_buf = input_text.buffer();
        let dropdown = recipient_dropdown.clone();
        let key_ids = Rc::clone(&key_ids);

        encrypt_btn.connect_clicked(move |_| {
            let buf = &input_buf;
            let plaintext = buf
                .text(&buf.start_iter(), &buf.end_iter(), false)
                .to_string();

            let ids = key_ids.borrow();
            let selected = dropdown.selected() as usize;
            let recipient = ids.get(selected).cloned().unwrap_or_default();
            cvm.encrypt(plaintext, recipient);
        });
    }

    // --- clear button ---
    {
        let cvm = Rc::clone(crypto_vm);
        let input_buf = input_text.buffer();
        let output_buf = output_text.buffer();

        clear_btn.connect_clicked(move |_| {
            input_buf.set_text("");
            output_buf.set_text("");
            cvm.clear_encrypt();
        });
    }

    // --- copy button ---
    {
        let output_buf = output_text.buffer();
        copy_btn.connect_clicked(move |_| {
            let text = output_buf
                .text(&output_buf.start_iter(), &output_buf.end_iter(), false)
                .to_string();
            if !text.is_empty() {
                copy_to_clipboard(&text);
            }
        });
    }

    // --- subscribe to crypto state ---
    {
        let output_buf = output_text.buffer();
        let spinner = spinner.clone();
        let copy_btn = copy_btn.clone();

        crypto_vm.subscribe(Box::new(move |state| {
            spinner.set_visible(state.loading);
            spinner.set_spinning(state.loading);

            if !state.encrypt_output.is_empty() {
                output_buf.set_text(&state.encrypt_output);
                copy_btn.set_sensitive(true);
            } else {
                copy_btn.set_sensitive(false);
            }
        }));
    }

    container
}
