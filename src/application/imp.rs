use std::cell::RefCell;
use std::sync::Arc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib;
use gtk::gio;

use crate::api::HarvestClient;
use crate::credentials;
use crate::tray;
use crate::window::HarvuxWindow;

#[derive(Default)]
pub struct HarvuxApplication {
    pub tokio_rt: RefCell<Option<Arc<tokio::runtime::Runtime>>>,
    hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
}

#[glib::object_subclass]
impl ObjectSubclass for HarvuxApplication {
    const NAME: &'static str = "HarvuxApplication";
    type Type = super::HarvuxApplication;
    type ParentType = adw::Application;
}

impl ObjectImpl for HarvuxApplication {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.setup_actions();
    }
}

impl ApplicationImpl for HarvuxApplication {
    fn startup(&self) {
        self.parent_startup();

        let app = self.obj();

        // Keep the application alive even when all windows are hidden
        self.hold_guard.replace(Some(app.hold()));

        // Spawn the system tray icon on the tokio runtime
        let tray = tray::HarvuxTray::new(app.as_ref());
        app.tokio_rt().spawn(tray.run());
    }

    fn activate(&self) {
        let app = self.obj();

        // If we already have a window, just toggle its visibility
        if let Some(window) = app.active_window() {
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.present();
            }
            return;
        }

        // Check for hidden windows — present them
        let windows = app.windows();
        if let Some(window) = windows.first() {
            window.present();
            return;
        }

        // No window exists yet — create one and load credentials
        let adw_app: &adw::Application = app.upcast_ref();
        let window = HarvuxWindow::new(adw_app);
        window.present();

        let rt = app.tokio_rt();
        let window_clone = window.clone();
        let rt_clone = rt.clone();
        glib::spawn_future_local(async move {
            let creds_result = rt_clone
                .spawn(async move { credentials::load_credentials().await })
                .await
                .unwrap();

            match creds_result {
                Ok(Some(creds)) => {
                    match HarvestClient::new(&creds.access_token, &creds.account_id) {
                        Ok(client) => {
                            window_clone.show_timer(rt, client);
                        }
                        Err(_) => {
                            window_clone.show_welcome();
                        }
                    }
                }
                _ => {
                    window_clone.show_welcome();
                }
            }
        });
    }
}

impl GtkApplicationImpl for HarvuxApplication {}
impl AdwApplicationImpl for HarvuxApplication {}
