use serde::{Deserialize, Serialize};

// --- Full API response structs (from dedicated endpoints) ---

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub timezone: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
    pub is_billable: bool,
    pub client: Option<Client>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskAssignment {
    pub id: i64,
    pub task: Task,
    pub is_active: bool,
    pub billable: Option<bool>,
}

// --- Simplified nested structs (as embedded in time entry responses) ---

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntryProject {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntryTask {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntryUser {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntryClient {
    pub id: i64,
    pub name: String,
}

// --- Time entry ---

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntry {
    pub id: i64,
    pub spent_date: String,
    pub hours: f64,
    pub rounded_hours: Option<f64>,
    pub notes: Option<String>,
    pub is_running: bool,
    pub timer_started_at: Option<String>,
    pub started_time: Option<String>,
    pub ended_time: Option<String>,
    pub project: Option<TimeEntryProject>,
    pub task: Option<TimeEntryTask>,
    pub client: Option<TimeEntryClient>,
    pub user: Option<TimeEntryUser>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTimeEntry {
    pub project_id: i64,
    pub task_id: i64,
    pub spent_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTimeEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spent_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// Paginated response wrappers

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
    pub total_entries: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskAssignmentsResponse {
    pub task_assignments: Vec<TaskAssignment>,
    pub total_entries: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<TimeEntry>,
    pub total_entries: i64,
}
