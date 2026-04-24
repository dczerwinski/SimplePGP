use adw::prelude::*;

use crate::app::APP_ID;

const APP_NAME: &str = "SimplePGP";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const WEBSITE: &str = "https://github.com/dczerwinski/SimplePGP";
const ISSUE_URL: &str = "https://github.com/dczerwinski/SimplePGP/issues";

/// Shows the About dialog with program information.
pub fn show_about_dialog(parent: &impl IsA<gtk::Widget>) {
    let developers: Vec<&str> = if AUTHORS.is_empty() {
        vec!["Dominik Czerwiński"]
    } else {
        AUTHORS.split(':').collect()
    };

    let about = adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .version(VERSION)
        .comments(DESCRIPTION)
        .website(WEBSITE)
        .issue_url(ISSUE_URL)
        .developers(developers)
        .copyright("© 2026 Dominik Czerwiński")
        .license_type(license_type_from(LICENSE))
        .build();

    about.present(Some(parent));
}

fn license_type_from(spdx: &str) -> gtk::License {
    match spdx {
        "Apache-2.0" => gtk::License::Apache20,
        "MIT" => gtk::License::MitX11,
        "GPL-3.0" | "GPL-3.0-or-later" | "GPL-3.0-only" => gtk::License::Gpl30,
        "GPL-2.0" | "GPL-2.0-or-later" | "GPL-2.0-only" => gtk::License::Gpl20,
        "LGPL-3.0" | "LGPL-3.0-or-later" | "LGPL-3.0-only" => gtk::License::Lgpl30,
        "LGPL-2.1" | "LGPL-2.1-or-later" | "LGPL-2.1-only" => gtk::License::Lgpl21,
        "MPL-2.0" => gtk::License::Mpl20,
        "BSD-2-Clause" => gtk::License::Bsd,
        "BSD-3-Clause" => gtk::License::Bsd3,
        _ => gtk::License::Custom,
    }
}
