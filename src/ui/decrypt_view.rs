use std::rc::Rc;

use adw::prelude::*;

use crate::utils::copy_to_clipboard;
use crate::viewmodels::CryptoViewModel;

/// Builds the "Decrypt" tab content.
pub fn build_decrypt_view(crypto_vm: &Rc<CryptoViewModel>) -> gtk::Box {
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

    // --- ciphertext input ---
    let input_group = adw::PreferencesGroup::builder()
        .title("Encrypted Text")
        .description("Paste the PGP-encrypted message")
        .vexpand(true)
        .build();

    let input_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(200)
        .vexpand(true)
        .hexpand(true)
        .css_classes(["card"])
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

    let decrypt_btn = gtk::Button::builder()
        .label("Decrypt")
        .css_classes(["suggested-action"])
        .build();

    let clear_btn = gtk::Button::builder()
        .label("Clear")
        .css_classes(["destructive-action"])
        .tooltip_text("Securely clear all text fields")
        .build();

    btn_box.append(&clear_btn);
    btn_box.append(&decrypt_btn);

    // --- spinner ---
    let spinner = gtk::Spinner::builder()
        .spinning(false)
        .halign(gtk::Align::Center)
        .visible(false)
        .build();

    // --- output ---
    let output_group = adw::PreferencesGroup::builder()
        .title("Decrypted Output")
        .vexpand(true)
        .build();

    let output_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(200)
        .vexpand(true)
        .hexpand(true)
        .css_classes(["card"])
        .build();

    let output_text = gtk::TextView::builder()
        .monospace(true)
        .editable(false)
        .wrap_mode(gtk::WrapMode::WordChar)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
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

    inner.append(&input_group);
    inner.append(&btn_box);
    inner.append(&spinner);
    inner.append(&output_group);
    clamp.set_child(Some(&inner));
    container.append(&clamp);

    // --- decrypt button ---
    {
        let cvm = Rc::clone(crypto_vm);
        let input_buf = input_text.buffer();

        decrypt_btn.connect_clicked(move |_| {
            let buf = &input_buf;
            let ciphertext = buf
                .text(&buf.start_iter(), &buf.end_iter(), false)
                .to_string();
            cvm.decrypt(ciphertext);
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
            cvm.clear_decrypt();
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

            if !state.decrypt_output.is_empty() {
                output_buf.set_text(&state.decrypt_output);
                copy_btn.set_sensitive(true);
            } else {
                output_buf.set_text("");
                copy_btn.set_sensitive(false);
            }
        }));
    }

    container
}
