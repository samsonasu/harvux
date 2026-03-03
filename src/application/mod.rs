mod imp;

use std::sync::Arc;

use adw::prelude::*;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::preferences::HarvuxPreferences;

glib::wrapper! {
    pub struct HarvuxApplication(ObjectSubclass<imp::HarvuxApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl HarvuxApplication {
    pub fn new(
        application_id: &str,
        flags: &gio::ApplicationFlags,
        rt: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        let app: Self = glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build();
        app.imp().tokio_rt.replace(Some(rt));
        app
    }

    pub fn tokio_rt(&self) -> Arc<tokio::runtime::Runtime> {
        self.imp()
            .tokio_rt
            .borrow()
            .as_ref()
            .expect("Tokio runtime not initialized")
            .clone()
    }

    fn setup_actions(&self) {
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| {
                app.show_about();
            })
            .build();

        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| {
                app.show_preferences();
            })
            .build();

        self.add_action_entries([about_action, preferences_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();

        let about = adw::AboutDialog::builder()
            .application_name("Harvux")
            .application_icon("com.github.samsonasu.Harvux")
            .developer_name("bsamson")
            .version("0.1.0")
            .developers(vec!["bsamson"])
            .copyright("2026 bsamson")
            .license_type(gtk::License::Gpl30)
            .comments("A native Linux time tracker for Harvest")
            .build();

        about.present(Some(&window));
    }

    fn show_preferences(&self) {
        let window = self.active_window().unwrap();
        let prefs = HarvuxPreferences::new(self.tokio_rt());
        prefs.present(Some(&window));
    }
}
