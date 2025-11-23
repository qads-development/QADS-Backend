use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub business_name: String,
    pub business_website: String,
    pub business_sector: String,
    pub revenue: String,
    pub goals: String,
    pub email: String,
    pub job_title: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl Client {
    pub fn new(
        business_name: String,
        business_website: String,
        business_sector: String,
        revenue: String,
        goals: String,
        email: String,
        job_title: String,
        username: String,
        password_hash: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            business_name,
            business_website,
            business_sector,
            revenue,
            goals,
            email,
            job_title,
            username,
            password_hash,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub title: String,
    pub salary: f64,
    pub status: String,
    pub paid: bool,
    pub created_at: DateTime<Utc>,
}

impl Employee {
    pub fn new(client_id: String, name: String, title: String, salary: f64, status: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            name,
            title,
            salary,
            status,
            paid: false,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub client_id: String,
    pub title: String,
    pub priority: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(client_id: String, title: String, priority: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            title,
            priority,
            done: false,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub client_id: String,
    pub title: String,
    pub description: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

impl Event {
    pub fn new(
        client_id: String,
        title: String,
        description: String,
        start_date: String,
        start_time: String,
        end_date: String,
        end_time: String,
        color: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            title,
            description,
            start_date,
            start_time,
            end_date,
            end_time,
            color,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingRequest {
    pub business_name: String,
    pub business_website: String,
    pub business_sector: String,
    pub revenue: String,
    pub goals: String,
    pub custom_goal_text: Option<String>,
    pub email: String,
    pub job_title: String,
    pub services: Vec<String>,
    pub other_service_text: Option<String>,
    pub platforms: Vec<String>,
    pub generated_username: String,
    pub generated_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_id: String,
    pub client_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub title: String,
    pub salary: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEmployeePaymentRequest {
    pub paid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_date: String,
    pub start_time: Option<String>,
    pub end_date: String,
    pub end_time: Option<String>,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    DbError(String),
    NotFound(String),
    InvalidInput(String),
    Unauthorized,
    InternalError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DbError(msg) => write!(f, "Database Error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized Access"),
            AppError::InternalError => write!(f, "Internal Server Error"),
        }
    }
}

impl std::error::Error for AppError {}

pub struct ServiceMetrics {
    pub active_sessions: u32,
    pub total_requests: u64,
    pub uptime_seconds: u64,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            active_sessions: 0,
            total_requests: 0,
            uptime_seconds: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String,
    pub version: String,
    pub maintenance_mode: bool,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            status: "Operational".to_string(),
            version: "1.0.0".to_string(),
            maintenance_mode: false,
        }
    }
}

pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

impl Validatable for OnboardingRequest {
    fn validate(&self) -> Result<(), String> {
        if self.business_name.is_empty() {
            return Err("Business name is required".to_string());
        }
        if self.email.is_empty() || !self.email.contains('@') {
            return Err("Valid email is required".to_string());
        }
        if self.generated_username.len() < 3 {
            return Err("Username too short".to_string());
        }
        if self.generated_password.len() < 6 {
            return Err("Password too short".to_string());
        }
        Ok(())
    }
}

impl Validatable for LoginRequest {
    fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.trim().is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        Ok(())
    }
}

impl Validatable for CreateEmployeeRequest {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Employee name is required".to_string());
        }
        if self.salary < 0.0 {
            return Err("Salary cannot be negative".to_string());
        }
        Ok(())
    }
}

impl Validatable for CreateEventRequest {
    fn validate(&self) -> Result<(), String> {
        if self.title.is_empty() {
            return Err("Event title is required".to_string());
        }
        if self.start_date.is_empty() || self.end_date.is_empty() {
            return Err("Start and end dates are required".to_string());
        }
        Ok(())
    }
}

pub struct SessionData {
    pub client_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_employees: i64,
    pub monthly_payroll: f64,
    pub active_tasks: i64,
    pub total_events: i64,
}

#[derive(Debug, Serialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub file_type: String,
}

impl DocumentMetadata {
    pub fn mock_list() -> Vec<Self> {
        vec![
            Self {
                id: Uuid::new_v4().to_string(),
                name: "Q4_Financial_Report.pdf".to_string(),
                size_bytes: 1024 * 500,
                uploaded_at: Utc::now(),
                file_type: "application/pdf".to_string(),
            },
            Self {
                id: Uuid::new_v4().to_string(),
                name: "Employee_Handbook_2025.docx".to_string(),
                size_bytes: 1024 * 2500,
                uploaded_at: Utc::now(),
                file_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            },
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct SpreadsheetMetadata {
    pub id: String,
    pub name: String,
    pub last_modified: DateTime<Utc>,
    pub row_count: u32,
}

impl SpreadsheetMetadata {
    pub fn mock_list() -> Vec<Self> {
        vec![
            Self {
                id: Uuid::new_v4().to_string(),
                name: "Inventory_Tracking_Sheet".to_string(),
                last_modified: Utc::now(),
                row_count: 1542,
            },
            Self {
                id: Uuid::new_v4().to_string(),
                name: "Q1_Sales_Forecast".to_string(),
                last_modified: Utc::now(),
                row_count: 350,
            },
        ]
    }
}

pub fn generate_secure_token() -> String {
    let mut token = String::new();
    for _ in 0..32 {
        let random_char = match rand::random::<u8>() % 3 {
            0 => (rand::random::<u8>() % 10 + 48) as char,
            1 => (rand::random::<u8>() % 26 + 65) as char,
            _ => (rand::random::<u8>() % 26 + 97) as char,
        };
        token.push(random_char);
    }
    token
}

pub fn format_money(amount: f64) -> String {
    format!("${:.2}", amount)
}

pub fn sanitize_string(input: &str) -> String {
    input.replace('<', "&lt;").replace('>', "&gt;").trim().to_string()
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

pub fn log_event(level: LogLevel, message: &str) {
    let timestamp = Utc::now().to_rfc3339();
    println!("[{}] {:?}: {}", timestamp, level, message);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub uptime: u64,
    pub database_connected: bool,
    pub timestamp: String,
}

pub trait EntityIdentity {
    fn get_id(&self) -> &str;
    fn get_type(&self) -> &str;
}

impl EntityIdentity for Client {
    fn get_id(&self) -> &str { &self.id }
    fn get_type(&self) -> &str { "Client" }
}

impl EntityIdentity for Employee {
    fn get_id(&self) -> &str { &self.id }
    fn get_type(&self) -> &str { "Employee" }
}

impl EntityIdentity for Task {
    fn get_id(&self) -> &str { &self.id }
    fn get_type(&self) -> &str { "Task" }
}

impl EntityIdentity for Event {
    fn get_id(&self) -> &str { &self.id }
    fn get_type(&self) -> &str { "Event" }
}
