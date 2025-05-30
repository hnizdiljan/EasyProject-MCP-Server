use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value;

/// Project model podle EasyProject API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_members: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ProjectReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trackers: Option<Vec<Tracker>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_categories: Option<Vec<IssueCategory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_modules: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReference {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    #[serde(rename = "1")]
    Active,
    #[serde(rename = "5")]
    Closed,
    #[serde(rename = "9")]
    Archived,
    #[serde(rename = "15")]
    Planned,
    #[serde(rename = "19")]
    Deleted,
}

/// Issue (Task) model podle EasyProject API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i32,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub project: ProjectReference,
    pub tracker: Tracker,
    pub status: IssueStatus,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<UserReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<IssueCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_version: Option<Version>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<IssueReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spent_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueReference {
    pub id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
}

/// User model podle EasyProject API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy_system_flag: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy_lesser_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy_external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy_user_type: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easy_user_type_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utc_offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twofa_scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_time_calendar: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor_user_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passwd_changed_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i32,
    pub project: ProjectReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<IssueReference>,
    pub user: UserReference,
    pub activity: TimeEntryActivity,
    pub hours: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    pub spent_on: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryActivity {
    pub id: i32,
    pub name: String,
}

/// API Response wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub project: Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuesResponse {
    pub issues: Vec<Issue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueResponse {
    pub issue: Issue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<TimeEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryResponse {
    pub time_entry: TimeEntry,
}

/// Request models pro vytváření/aktualizaci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub project: CreateProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProject {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_members: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_module_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub issue: CreateIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssue {
    pub project_id: i32,
    pub tracker_id: i32,
    pub status_id: i32,
    pub priority_id: i32,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_version_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_issue_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeEntryRequest {
    pub time_entry: CreateTimeEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i32>,
    pub spent_on: NaiveDate,
    pub hours: f64,
    pub activity_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
} 