use std::cell::RefCell;
use std::sync::Arc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib;

use crate::api::HarvestClient;
use crate::credentials;
use crate::window::HarvuxWindow;

#[derive(Default)]
pub struct HarvuxApplication {
    pub tokio_rt: RefCell<Option<Arc<tokio::runtime::Runtime>>>,
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
    fn activate(&self) {
        let app = self.obj();

        let window: HarvuxWindow = app
            .active_window()
            .and_then(|w| w.downcast().ok())
            .unwrap_or_else(|| {
                let adw_app: &adw::Application = app.upcast_ref();
                HarvuxWindow::new(adw_app)
            });

        window.present();

        // Try to load credentials and connect
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
