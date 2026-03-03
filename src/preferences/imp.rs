use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib;
use gtk::CompositeTemplate;

use crate::api::HarvestClient;
use crate::credentials;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/bsamson/Harvux/preferences.ui")]
pub struct HarvuxPreferences {
    #[template_child]
    pub token_row: TemplateChild<adw::PasswordEntryRow>,
    #[template_child]
    pub account_id_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub test_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub save_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub status_label: TemplateChild<gtk::Label>,

    pub tokio_rt: RefCell<Option<std::sync::Arc<tokio::runtime::Runtime>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for HarvuxPreferences {
    const NAME: &'static str = "HarvuxPreferences";
    type Type = super::HarvuxPreferences;
    type ParentType = adw::PreferencesDialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl HarvuxPreferences {
    #[template_callback]
    fn on_test_clicked(&self) {
        let token = self.token_row.text().to_string();
        let account_id = self.account_id_row.text().to_string();

        if token.is_empty() || account_id.is_empty() {
            self.status_label
                .set_label("Please enter both access token and account ID.");
            self.status_label.remove_css_class("success");
            self.status_label.add_css_class("error");
            return;
        }

        let status_label = self.status_label.clone();
        let test_button = self.test_button.clone();
        let rt = self
            .tokio_rt
            .borrow()
            .as_ref()
            .expect("Tokio runtime not set")
            .clone();

        test_button.set_sensitive(false);
        status_label.set_label("Testing connection...");
        status_label.remove_css_class("success");
        status_label.remove_css_class("error");

        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move {
                    let client = HarvestClient::new(&token, &account_id)?;
                    client.me().await
                })
                .await
                .unwrap();

            match result {
                Ok(user) => {
                    status_label.set_label(&format!(
                        "Connected as {} {} ({})",
                        user.first_name, user.last_name, user.email
                    ));
                    status_label.remove_css_class("error");
                    status_label.add_css_class("success");
                }
                Err(err) => {
                    status_label.set_label(&format!("Connection failed: {err}"));
                    status_label.remove_css_class("success");
                    status_label.add_css_class("error");
                }
            }
            test_button.set_sensitive(true);
        });
    }

    #[template_callback]
    fn on_save_clicked(&self) {
        let token = self.token_row.text().to_string();
        let account_id = self.account_id_row.text().to_string();

        if token.is_empty() || account_id.is_empty() {
            self.status_label
                .set_label("Please enter both access token and account ID.");
            self.status_label.remove_css_class("success");
            self.status_label.add_css_class("error");
            return;
        }

        let status_label = self.status_label.clone();
        let save_button = self.save_button.clone();
        let rt = self
            .tokio_rt
            .borrow()
            .as_ref()
            .expect("Tokio runtime not set")
            .clone();

        let dialog = self.obj().clone();

        save_button.set_sensitive(false);
        status_label.set_label("Saving credentials...");
        status_label.remove_css_class("success");
        status_label.remove_css_class("error");

        glib::spawn_future_local(async move {
            let token_for_store = token.clone();
            let account_id_for_store = account_id.clone();
            let rt_clone = rt.clone();
            let result = rt_clone
                .spawn(async move {
                    credentials::store_credentials(&token_for_store, &account_id_for_store).await
                })
                .await
                .unwrap();

            match result {
                Ok(()) => {
                    // Try to create client and switch to timer view
                    match HarvestClient::new(&token, &account_id) {
                        Ok(client) => {
                            // Find the main window via the application and switch to timer
                            if let Some(app) = gtk::gio::Application::default() {
                                if let Some(window) = app
                                    .downcast_ref::<gtk::Application>()
                                    .and_then(|a| a.active_window())
                                    .and_then(|w| w.downcast::<crate::window::HarvuxWindow>().ok())
                                {
                                    window.show_timer(rt, client);
                                }
                            }
                            dialog.close();
                        }
                        Err(_) => {
                            status_label.set_label("Credentials saved, but could not create client.");
                            status_label.remove_css_class("error");
                            status_label.add_css_class("success");
                            save_button.set_sensitive(true);
                        }
                    }
                }
                Err(err) => {
                    status_label.set_label(&format!("Failed to save: {err}"));
                    status_label.remove_css_class("success");
                    status_label.add_css_class("error");
                    save_button.set_sensitive(true);
                }
            }
        });
    }

    /// Load existing credentials into the form fields.
    /// Called from mod.rs after the tokio runtime has been set.
    pub fn load_existing_credentials(&self) {
        let rt = self
            .tokio_rt
            .borrow()
            .as_ref()
            .expect("Tokio runtime not set")
            .clone();

        let token_row = self.token_row.clone();
        let account_id_row = self.account_id_row.clone();

        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move { credentials::load_credentials().await })
                .await
                .unwrap();

            if let Ok(Some(creds)) = result {
                token_row.set_text(&creds.access_token);
                account_id_row.set_text(&creds.account_id);
            }
        });
    }
}

impl ObjectImpl for HarvuxPreferences {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for HarvuxPreferences {}
impl AdwDialogImpl for HarvuxPreferences {}
impl PreferencesDialogImpl for HarvuxPreferences {}
