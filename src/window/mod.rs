mod imp;

use std::sync::Arc;

use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::api::HarvestClient;

glib::wrapper! {
    pub struct HarvuxWindow(ObjectSubclass<imp::HarvuxWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap,
                    gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl HarvuxWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    pub fn show_timer(&self, rt: Arc<tokio::runtime::Runtime>, client: HarvestClient) {
        let imp = self.imp();
        imp.timer_page.setup(rt, client);
        imp.stack.set_visible_child_name("timer");
    }

    pub fn show_welcome(&self) {
        self.imp().stack.set_visible_child_name("welcome");
    }
}
