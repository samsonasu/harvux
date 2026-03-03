mod imp;

use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

glib::wrapper! {
    pub struct HarvuxPreferences(ObjectSubclass<imp::HarvuxPreferences>)
        @extends adw::PreferencesDialog, adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HarvuxPreferences {
    pub fn new(rt: std::sync::Arc<tokio::runtime::Runtime>) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().tokio_rt.replace(Some(rt));
        obj.imp().load_existing_credentials();
        obj
    }
}
