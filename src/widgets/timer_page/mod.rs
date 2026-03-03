mod imp;

use std::sync::Arc;

use adw::prelude::*;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib;

use crate::api::{self, HarvestClient};

glib::wrapper! {
    pub struct HarvuxTimerPage(ObjectSubclass<imp::HarvuxTimerPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl HarvuxTimerPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn setup(&self, rt: Arc<tokio::runtime::Runtime>, client: HarvestClient) {
        let imp = self.imp();
        imp.tokio_rt.replace(Some(rt));
        imp.client.replace(Some(client));
        self.fetch_current_user();
        self.load_projects();
    }

    fn fetch_current_user(&self) {
        let client = self.imp().client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.imp().tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let obj = self.clone();
        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move { client.me().await })
                .await
                .unwrap();

            match result {
                Ok(user) => {
                    obj.imp().user_id.set(Some(user.id));
                    obj.refresh_entries();
                }
                Err(err) => {
                    eprintln!("Failed to fetch current user: {err}");
                    obj.refresh_entries();
                }
            }
        });
    }

    fn load_projects(&self) {
        let client = self.imp().client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.imp().tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let obj = self.clone();
        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move { client.list_projects().await })
                .await
                .unwrap();

            match result {
                Ok(projects) => {
                    let imp = obj.imp();
                    let names: Vec<String> = projects
                        .iter()
                        .map(|p| {
                            if let Some(ref c) = p.client {
                                format!("{} - {}", c.name, p.name)
                            } else {
                                p.name.clone()
                            }
                        })
                        .collect();

                    let model = gtk::StringList::new(&names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                    imp.project_row.set_model(Some(&model));
                    imp.projects.replace(projects);

                    if !names.is_empty() {
                        imp.project_row.set_selected(0);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to load projects: {err}");
                }
            }
        });
    }

    fn load_tasks_for_project(&self, project_idx: usize) {
        let projects = self.imp().projects.borrow();
        let Some(project) = projects.get(project_idx) else {
            return;
        };
        let project_id = project.id;
        drop(projects);

        let client = self.imp().client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.imp().tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let obj = self.clone();
        self.imp().task_row.set_sensitive(false);

        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move { client.list_task_assignments(project_id).await })
                .await
                .unwrap();

            match result {
                Ok(assignments) => {
                    let imp = obj.imp();
                    let names: Vec<String> =
                        assignments.iter().map(|a| a.task.name.clone()).collect();

                    let model = gtk::StringList::new(&names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                    imp.task_row.set_model(Some(&model));
                    imp.task_row.set_sensitive(true);
                    imp.task_assignments.replace(assignments);

                    if !names.is_empty() {
                        // Check for pending task from entry selection
                        if let Some(target_task_id) = imp.pending_task_id.take() {
                            let task_idx = imp
                                .task_assignments
                                .borrow()
                                .iter()
                                .position(|a| a.task.id == target_task_id);
                            imp.task_row.set_selected(task_idx.unwrap_or(0) as u32);

                            // Set last_stopped AFTER task handler has cleared it
                            if let Some(entry_id) = imp.pending_entry_id.take() {
                                let projects = imp.projects.borrow();
                                let pid = projects
                                    .get(project_idx)
                                    .map(|p| p.id);
                                drop(projects);
                                imp.last_stopped_entry_id.set(Some(entry_id));
                                imp.last_stopped_project_id.set(pid);
                                imp.last_stopped_task_id.set(Some(target_task_id));
                            }
                            imp.timer_button.set_sensitive(true);
                        } else {
                            imp.task_row.set_selected(0);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to load tasks: {err}");
                    obj.imp().task_row.set_sensitive(false);
                }
            }
        });
    }

    pub(crate) fn on_entry_activated(&self, idx: usize) {
        let imp = self.imp();
        let entries = imp.time_entries.borrow();
        let Some(entry) = entries.get(idx) else {
            return;
        };
        if entry.is_running {
            return;
        }

        let entry_id = entry.id;
        let project_id = entry.project.as_ref().map(|p| p.id);
        let task_id = entry.task.as_ref().map(|t| t.id);
        let hours = entry.hours;
        let notes = entry.notes.clone().unwrap_or_default();
        drop(entries);

        let Some(project_id) = project_id else { return };
        let Some(task_id) = task_id else { return };

        imp.select_entry_from_list(entry_id, project_id, task_id, hours, &notes);
    }

    pub fn refresh_entries(&self) {
        let client = self.imp().client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.imp().tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let user_id = self.imp().user_id.get();
        let obj = self.clone();

        glib::spawn_future_local(async move {
            let date = today.clone();
            let result = rt
                .spawn(async move { client.list_time_entries_today(&date, user_id).await })
                .await
                .unwrap();

            match result {
                Ok(entries) => {
                    obj.update_entries_list(&entries);
                    obj.imp().time_entries.replace(entries);
                }
                Err(err) => {
                    eprintln!("Failed to load time entries: {err}");
                }
            }
        });
    }

    fn update_entries_list(&self, entries: &[api::TimeEntry]) {
        let imp = self.imp();
        let list = &imp.entries_list;

        // Remove all existing rows
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let mut total_hours: f64 = 0.0;

        for entry in entries {
            total_hours += entry.hours;

            let project_name = entry
                .project
                .as_ref()
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown");
            let task_name = entry
                .task
                .as_ref()
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown");
            let notes = entry.notes.as_deref().unwrap_or("");

            let hours = entry.hours;
            let h = hours as i64;
            let m = ((hours - h as f64) * 60.0) as i64;
            let time_str = if h > 0 {
                format!("{h}h {m:02}m")
            } else {
                format!("{m}m")
            };

            let title = format!("{project_name} · {task_name}");
            let subtitle = if notes.is_empty() {
                time_str.clone()
            } else {
                format!("{notes}\n{time_str}")
            };

            let row = adw::ActionRow::builder()
                .title(&title)
                .subtitle(&subtitle)
                .activatable(!entry.is_running)
                .build();

            if entry.is_running {
                let running_label = gtk::Label::builder()
                    .label("Running")
                    .css_classes(vec!["success".to_string()])
                    .valign(gtk::Align::Center)
                    .build();
                row.add_suffix(&running_label);

                // Update our timer state from a running entry
                imp.running_entry_id.set(Some(entry.id));
                imp.running_hours.set(entry.hours);
                imp.timer_started_at.replace(Some(chrono::Utc::now()));
                imp.timer_button.set_label("Stop Timer");
                imp.timer_button
                    .remove_css_class("suggested-action");
                imp.timer_button.add_css_class("destructive-action");
                imp.timer_button.set_sensitive(true);

                // Start tick timer if not already running
                if imp.timer_source_id.borrow().is_none() {
                    let obj_weak = self.downgrade();
                    let source_id = glib::timeout_add_seconds_local(1, move || {
                        let Some(obj) = obj_weak.upgrade() else {
                            return glib::ControlFlow::Break;
                        };
                        let imp = obj.imp();
                        if let Some(started) = *imp.timer_started_at.borrow() {
                            let elapsed = chrono::Utc::now() - started;
                            let total_secs =
                                elapsed.num_seconds() + (imp.running_hours.get() * 3600.0) as i64;
                            let hh = total_secs / 3600;
                            let mm = (total_secs % 3600) / 60;
                            let ss = total_secs % 60;
                            imp.timer_label
                                .set_label(&format!("{hh:02}:{mm:02}:{ss:02}"));
                        }
                        glib::ControlFlow::Continue
                    });
                    imp.timer_source_id.replace(Some(source_id));
                }
            } else {
                let time_label = gtk::Label::builder()
                    .label(&time_str)
                    .css_classes(vec!["dim-label".to_string()])
                    .valign(gtk::Align::Center)
                    .build();
                row.add_suffix(&time_label);
            }

            list.append(&row);
        }

        // Update total
        let th = total_hours as i64;
        let tm = ((total_hours - th as f64) * 60.0) as i64;
        imp.total_label
            .set_label(&format!("{th}h {tm:02}m"));
    }

}
