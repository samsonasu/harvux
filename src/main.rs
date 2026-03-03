mod api;
mod application;
mod credentials;
mod preferences;
mod widgets;
mod window;

use std::sync::Arc;

use gtk::{gio, glib, prelude::*};

use application::HarvuxApplication;

const APP_ID: &str = "com.github.samsonasu.Harvux";

fn main() -> glib::ExitCode {
    gio::resources_register_include!("harvux.gresource")
        .expect("Failed to register resources.");

    let rt = Arc::new(
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
    );

    let app = HarvuxApplication::new(APP_ID, &gio::ApplicationFlags::empty(), rt);
    app.run()
}
