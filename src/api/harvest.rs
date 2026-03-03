use anyhow::{Context, Result};

use super::client::HarvestClient;
use super::models::*;

impl HarvestClient {
    pub async fn me(&self) -> Result<User> {
        self.get("/users/me")
            .send()
            .await
            .context("Failed to fetch current user")?
            .error_for_status()
            .context("API error fetching current user")?
            .json()
            .await
            .context("Failed to parse user response")
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let resp: ProjectsResponse = self
            .get("/projects")
            .query(&[("is_active", "true"), ("per_page", "2000")])
            .send()
            .await
            .context("Failed to fetch projects")?
            .error_for_status()
            .context("API error fetching projects")?
            .json()
            .await
            .context("Failed to parse projects response")?;
        Ok(resp.projects)
    }

    pub async fn list_task_assignments(&self, project_id: i64) -> Result<Vec<TaskAssignment>> {
        let resp: TaskAssignmentsResponse = self
            .get(&format!("/projects/{project_id}/task_assignments"))
            .query(&[("is_active", "true"), ("per_page", "2000")])
            .send()
            .await
            .context("Failed to fetch task assignments")?
            .error_for_status()
            .context("API error fetching task assignments")?
            .json()
            .await
            .context("Failed to parse task assignments response")?;
        Ok(resp.task_assignments)
    }

    pub async fn list_time_entries_today(
        &self,
        date: &str,
        user_id: Option<i64>,
    ) -> Result<Vec<TimeEntry>> {
        let mut request = self
            .get("/time_entries")
            .query(&[("from", date), ("to", date)]);

        if let Some(uid) = user_id {
            request = request.query(&[("user_id", uid)]);
        }

        let resp: TimeEntriesResponse = request
            .send()
            .await
            .context("Failed to fetch time entries")?
            .error_for_status()
            .context("API error fetching time entries")?
            .json()
            .await
            .context("Failed to parse time entries response")?;
        Ok(resp.time_entries)
    }

    pub async fn create_time_entry(&self, entry: &CreateTimeEntry) -> Result<TimeEntry> {
        self.post("/time_entries")
            .json(entry)
            .send()
            .await
            .context("Failed to create time entry")?
            .error_for_status()
            .context("API error creating time entry")?
            .json()
            .await
            .context("Failed to parse created time entry")
    }

    pub async fn update_time_entry(
        &self,
        entry_id: i64,
        update: &UpdateTimeEntry,
    ) -> Result<TimeEntry> {
        self.patch(&format!("/time_entries/{entry_id}"))
            .json(update)
            .send()
            .await
            .context("Failed to update time entry")?
            .error_for_status()
            .context("API error updating time entry")?
            .json()
            .await
            .context("Failed to parse updated time entry")
    }

    pub async fn stop_time_entry(&self, entry_id: i64) -> Result<TimeEntry> {
        self.patch(&format!("/time_entries/{entry_id}/stop"))
            .send()
            .await
            .context("Failed to stop time entry")?
            .error_for_status()
            .context("API error stopping time entry")?
            .json()
            .await
            .context("Failed to parse stopped time entry")
    }

    pub async fn restart_time_entry(&self, entry_id: i64) -> Result<TimeEntry> {
        self.patch(&format!("/time_entries/{entry_id}/restart"))
            .send()
            .await
            .context("Failed to restart time entry")?
            .error_for_status()
            .context("API error restarting time entry")?
            .json()
            .await
            .context("Failed to parse restarted time entry")
    }

    pub async fn delete_time_entry(&self, entry_id: i64) -> Result<()> {
        self.delete(&format!("/time_entries/{entry_id}"))
            .send()
            .await
            .context("Failed to delete time entry")?
            .error_for_status()
            .context("API error deleting time entry")?;
        Ok(())
    }
}
