use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::widgets::HarvuxTimerPage;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/bsamson/Harvux/window.ui")]
pub struct HarvuxWindow {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,
    #[template_child]
    pub status_page: TemplateChild<adw::StatusPage>,
    #[template_child]
    pub menu_button: TemplateChild<gtk::MenuButton>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub timer_page: TemplateChild<HarvuxTimerPage>,
}

#[glib::object_subclass]
impl ObjectSubclass for HarvuxWindow {
    const NAME: &'static str = "HarvuxWindow";
    type Type = super::HarvuxWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        HarvuxTimerPage::ensure_type();
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for HarvuxWindow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for HarvuxWindow {}

impl WindowImpl for HarvuxWindow {
    fn close_request(&self) -> glib::Propagation {
        // Hide the window instead of destroying it so the app stays in the tray
        self.obj().set_visible(false);
        glib::Propagation::Stop
    }
}

impl ApplicationWindowImpl for HarvuxWindow {}
impl AdwApplicationWindowImpl for HarvuxWindow {}
