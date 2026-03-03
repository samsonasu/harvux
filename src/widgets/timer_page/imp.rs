use std::cell::{Cell, RefCell};
use std::sync::Arc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib;
use gtk::CompositeTemplate;

use crate::api::{self, HarvestClient};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/bsamson/Harvux/timer_page.ui")]
pub struct HarvuxTimerPage {
    #[template_child]
    pub project_row: TemplateChild<adw::ComboRow>,
    #[template_child]
    pub task_row: TemplateChild<adw::ComboRow>,
    #[template_child]
    pub notes_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub timer_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub timer_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub total_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub entries_list: TemplateChild<gtk::ListBox>,

    pub tokio_rt: RefCell<Option<Arc<tokio::runtime::Runtime>>>,
    pub client: RefCell<Option<HarvestClient>>,
    pub projects: RefCell<Vec<api::Project>>,
    pub task_assignments: RefCell<Vec<api::TaskAssignment>>,
    pub time_entries: RefCell<Vec<api::TimeEntry>>,
    pub user_id: Cell<Option<i64>>,
    pub running_entry_id: Cell<Option<i64>>,
    pub last_stopped_entry_id: Cell<Option<i64>>,
    pub last_stopped_project_id: Cell<Option<i64>>,
    pub last_stopped_task_id: Cell<Option<i64>>,
    pub timer_source_id: RefCell<Option<glib::SourceId>>,
    pub timer_started_at: RefCell<Option<chrono::DateTime<chrono::Utc>>>,
    pub running_hours: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for HarvuxTimerPage {
    const NAME: &'static str = "HarvuxTimerPage";
    type Type = super::HarvuxTimerPage;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl HarvuxTimerPage {
    #[template_callback]
    fn on_timer_clicked(&self) {
        if self.running_entry_id.get().is_some() {
            self.stop_timer();
        } else {
            self.start_timer();
        }
    }

    fn start_timer(&self) {
        let project_idx = self.project_row.selected();
        let task_idx = self.task_row.selected();

        let projects = self.projects.borrow();
        let tasks = self.task_assignments.borrow();

        if project_idx == gtk::INVALID_LIST_POSITION || task_idx == gtk::INVALID_LIST_POSITION {
            return;
        }

        let project = &projects[project_idx as usize];
        let task_assignment = &tasks[task_idx as usize];
        let project_id = project.id;
        let task_id = task_assignment.task.id;
        drop(projects);
        drop(tasks);

        // Check if we should restart the last stopped entry
        let should_restart = self.last_stopped_entry_id.get().is_some()
            && self.last_stopped_project_id.get() == Some(project_id)
            && self.last_stopped_task_id.get() == Some(task_id);

        if should_restart {
            let entry_id = self.last_stopped_entry_id.get().unwrap();
            self.restart_entry(entry_id);
        } else {
            self.create_new_entry(project_id, task_id);
        }
    }

    fn restart_entry(&self, entry_id: i64) {
        let notes = self.notes_row.text().to_string();

        let client = self.client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let timer_button = self.timer_button.clone();
        let obj = self.obj().clone();

        timer_button.set_sensitive(false);

        glib::spawn_future_local(async move {
            // Update notes before restarting
            let update = api::UpdateTimeEntry {
                project_id: None,
                task_id: None,
                spent_date: None,
                hours: None,
                notes: Some(if notes.is_empty() { String::new() } else { notes }),
            };
            let client_clone = client.clone();
            let rt_clone = rt.clone();
            let _ = rt_clone
                .spawn(async move { client_clone.update_time_entry(entry_id, &update).await })
                .await;

            // Now restart the timer
            let result = rt
                .spawn(async move { client.restart_time_entry(entry_id).await })
                .await
                .unwrap();

            match result {
                Ok(time_entry) => {
                    let imp = obj.imp();
                    imp.running_entry_id.set(Some(time_entry.id));
                    imp.last_stopped_entry_id.set(None);
                    imp.last_stopped_project_id.set(None);
                    imp.last_stopped_task_id.set(None);
                    imp.timer_started_at.replace(Some(chrono::Utc::now()));
                    // Keep existing running_hours — it was saved by stop_timer

                    obj.imp().set_timer_running(&timer_button);
                    obj.refresh_entries();
                }
                Err(err) => {
                    eprintln!("Failed to restart timer: {err}");
                    timer_button.set_sensitive(true);
                }
            }
        });
    }

    fn create_new_entry(&self, project_id: i64, task_id: i64) {
        let notes = self.notes_row.text().to_string();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let entry = api::CreateTimeEntry {
            project_id,
            task_id,
            spent_date: today,
            hours: None,
            notes: if notes.is_empty() { None } else { Some(notes) },
        };

        let client = self.client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let timer_button = self.timer_button.clone();
        let obj = self.obj().clone();

        timer_button.set_sensitive(false);

        glib::spawn_future_local(async move {
            let result = rt
                .spawn(async move { client.create_time_entry(&entry).await })
                .await
                .unwrap();

            match result {
                Ok(time_entry) => {
                    let imp = obj.imp();
                    imp.running_entry_id.set(Some(time_entry.id));
                    imp.last_stopped_entry_id.set(None);
                    imp.last_stopped_project_id.set(None);
                    imp.last_stopped_task_id.set(None);
                    imp.timer_started_at.replace(Some(chrono::Utc::now()));
                    imp.running_hours.set(0.0);

                    obj.imp().set_timer_running(&timer_button);
                    obj.refresh_entries();
                }
                Err(err) => {
                    eprintln!("Failed to start timer: {err}");
                    timer_button.set_sensitive(true);
                }
            }
        });
    }

    fn set_timer_running(&self, timer_button: &gtk::Button) {
        timer_button.set_label("Stop Timer");
        timer_button.remove_css_class("suggested-action");
        timer_button.add_css_class("destructive-action");
        timer_button.set_sensitive(true);

        // Start a 1-second tick to update the timer display
        if self.timer_source_id.borrow().is_none() {
            let obj_weak = self.obj().downgrade();
            let source_id = glib::timeout_add_seconds_local(1, move || {
                let Some(obj) = obj_weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                let imp = obj.imp();
                if let Some(started) = *imp.timer_started_at.borrow() {
                    let elapsed = chrono::Utc::now() - started;
                    let total_secs = elapsed.num_seconds()
                        + (imp.running_hours.get() * 3600.0) as i64;
                    let h = total_secs / 3600;
                    let m = (total_secs % 3600) / 60;
                    let s = total_secs % 60;
                    imp.timer_label
                        .set_label(&format!("{h:02}:{m:02}:{s:02}"));
                }
                glib::ControlFlow::Continue
            });
            self.timer_source_id.replace(Some(source_id));
        }
    }

    fn stop_timer(&self) {
        let Some(entry_id) = self.running_entry_id.get() else {
            return;
        };

        // Grab current notes to push to Harvest
        let notes = self.notes_row.text().to_string();

        // Remember the current project/task so we can restart this entry
        let project_idx = self.project_row.selected();
        let task_idx = self.task_row.selected();
        let projects = self.projects.borrow();
        let tasks = self.task_assignments.borrow();
        let stopped_project_id = if project_idx != gtk::INVALID_LIST_POSITION {
            projects.get(project_idx as usize).map(|p| p.id)
        } else {
            None
        };
        let stopped_task_id = if task_idx != gtk::INVALID_LIST_POSITION {
            tasks.get(task_idx as usize).map(|a| a.task.id)
        } else {
            None
        };
        drop(projects);
        drop(tasks);

        let client = self.client.borrow().as_ref().cloned();
        let Some(client) = client else { return };
        let rt = self.tokio_rt.borrow().as_ref().cloned();
        let Some(rt) = rt else { return };

        let timer_button = self.timer_button.clone();
        let obj = self.obj().clone();

        timer_button.set_sensitive(false);

        glib::spawn_future_local(async move {
            // Update notes before stopping
            let update = api::UpdateTimeEntry {
                project_id: None,
                task_id: None,
                spent_date: None,
                hours: None,
                notes: Some(if notes.is_empty() { String::new() } else { notes }),
            };
            let client_clone = client.clone();
            let _ = rt
                .spawn(async move { client_clone.update_time_entry(entry_id, &update).await })
                .await;

            // Now stop the timer
            let result = rt
                .spawn(async move { client.stop_time_entry(entry_id).await })
                .await
                .unwrap();

            match result {
                Ok(_) => {
                    let imp = obj.imp();
                    imp.running_entry_id.set(None);

                    // Save total accumulated hours so restart picks up from here
                    if let Some(started) = imp.timer_started_at.take() {
                        let elapsed_secs = (chrono::Utc::now() - started).num_seconds();
                        let total = imp.running_hours.get() + (elapsed_secs as f64 / 3600.0);
                        imp.running_hours.set(total);
                    }

                    // Remember this entry so Start restarts it
                    imp.last_stopped_entry_id.set(Some(entry_id));
                    imp.last_stopped_project_id.set(stopped_project_id);
                    imp.last_stopped_task_id.set(stopped_task_id);

                    // Stop the tick timer but keep the display value
                    if let Some(source_id) = imp.timer_source_id.take() {
                        source_id.remove();
                    }

                    timer_button.set_label("Start Timer");
                    timer_button.remove_css_class("destructive-action");
                    timer_button.add_css_class("suggested-action");
                    timer_button.set_sensitive(true);

                    obj.refresh_entries();
                }
                Err(err) => {
                    eprintln!("Failed to stop timer: {err}");
                    timer_button.set_sensitive(true);
                }
            }
        });
    }
}

impl ObjectImpl for HarvuxTimerPage {
    fn constructed(&self) {
        self.parent_constructed();

        // When project selection changes, load tasks and clear last-stopped state
        let obj = self.obj().clone();
        self.project_row.connect_selected_notify(move |row| {
            let idx = row.selected();
            if idx == gtk::INVALID_LIST_POSITION {
                return;
            }
            let imp = obj.imp();
            imp.last_stopped_entry_id.set(None);
            imp.last_stopped_project_id.set(None);
            imp.last_stopped_task_id.set(None);
            obj.load_tasks_for_project(idx as usize);
        });

        // Enable timer button when both project and task are selected;
        // clear last-stopped state when task changes
        let timer_button = self.timer_button.clone();
        let project_row = self.project_row.clone();
        let obj2 = self.obj().clone();
        self.task_row.connect_selected_notify(move |row| {
            let has_task = row.selected() != gtk::INVALID_LIST_POSITION;
            let has_project = project_row.selected() != gtk::INVALID_LIST_POSITION;
            timer_button.set_sensitive(has_task && has_project);
            let imp = obj2.imp();
            imp.last_stopped_entry_id.set(None);
            imp.last_stopped_project_id.set(None);
            imp.last_stopped_task_id.set(None);
        });
    }
}

impl WidgetImpl for HarvuxTimerPage {}
impl BoxImpl for HarvuxTimerPage {}
