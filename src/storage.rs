use rusqlite::{params, Connection, Result, OpenFlags};
use std::sync::{Arc, Mutex};
use crate::models::{Client, Employee, Task, Event, DashboardStats};
use std::path::Path;
use chrono::{DateTime, Utc, NaiveDateTime};

pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    db_path: String,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.to_string(),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clients (
                id TEXT PRIMARY KEY,
                business_name TEXT NOT NULL,
                business_website TEXT,
                business_sector TEXT,
                revenue TEXT,
                goals TEXT,
                email TEXT,
                job_title TEXT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS employees (
                id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                name TEXT NOT NULL,
                title TEXT NOT NULL,
                salary REAL NOT NULL,
                status TEXT NOT NULL,
                paid INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(client_id) REFERENCES clients(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                title TEXT NOT NULL,
                priority TEXT NOT NULL,
                done INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(client_id) REFERENCES clients(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                start_date TEXT NOT NULL,
                start_time TEXT,
                end_date TEXT NOT NULL,
                end_time TEXT,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(client_id) REFERENCES clients(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_employees_client ON employees(client_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_client ON tasks(client_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_client ON events(client_id)",
            [],
        )?;

        Ok(())
    }

    pub fn create_client(&self, client: &Client) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO clients (id, business_name, business_website, business_sector, revenue, goals, email, job_title, username, password_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                client.id,
                client.business_name,
                client.business_website,
                client.business_sector,
                client.revenue,
                client.goals,
                client.email,
                client.job_title,
                client.username,
                client.password_hash,
                client.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_client_by_username(&self, username: &str) -> Result<Option<Client>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM clients WHERE username = ?1")?;
        
        let client_iter = stmt.query_map(params![username], |row| {
            let created_str: String = row.get(10)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc);

            Ok(Client {
                id: row.get(0)?,
                business_name: row.get(1)?,
                business_website: row.get(2)?,
                business_sector: row.get(3)?,
                revenue: row.get(4)?,
                goals: row.get(5)?,
                email: row.get(6)?,
                job_title: row.get(7)?,
                username: row.get(8)?,
                password_hash: row.get(9)?,
                created_at,
            })
        })?;

        for client in client_iter {
            return Ok(Some(client?));
        }
        Ok(None)
    }

    pub fn create_employee(&self, employee: &Employee) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO employees (id, client_id, name, title, salary, status, paid, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                employee.id,
                employee.client_id,
                employee.name,
                employee.title,
                employee.salary,
                employee.status,
                if employee.paid { 1 } else { 0 },
                employee.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_employees(&self, client_id: &str) -> Result<Vec<Employee>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM employees WHERE client_id = ?1 ORDER BY name ASC")?;
        
        let employee_iter = stmt.query_map(params![client_id], |row| {
            let created_str: String = row.get(7)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc);
            
            let paid_int: i32 = row.get(6)?;

            Ok(Employee {
                id: row.get(0)?,
                client_id: row.get(1)?,
                name: row.get(2)?,
                title: row.get(3)?,
                salary: row.get(4)?,
                status: row.get(5)?,
                paid: paid_int == 1,
                created_at,
            })
        })?;

        let mut employees = Vec::new();
        for emp in employee_iter {
            employees.push(emp?);
        }
        Ok(employees)
    }

    pub fn delete_employee(&self, id: &str, client_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM employees WHERE id = ?1 AND client_id = ?2", params![id, client_id])
    }

    pub fn update_employee_paid_status(&self, id: &str, client_id: &str, paid: bool) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let val = if paid { 1 } else { 0 };
        conn.execute("UPDATE employees SET paid = ?1 WHERE id = ?2 AND client_id = ?3", params![val, id, client_id])
    }

    pub fn create_task(&self, task: &Task) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tasks (id, client_id, title, priority, done, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task.id,
                task.client_id,
                task.title,
                task.priority,
                if task.done { 1 } else { 0 },
                task.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_tasks(&self, client_id: &str) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM tasks WHERE client_id = ?1 ORDER BY created_at DESC")?;
        
        let task_iter = stmt.query_map(params![client_id], |row| {
            let created_str: String = row.get(5)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc);
            
            let done_int: i32 = row.get(4)?;

            Ok(Task {
                id: row.get(0)?,
                client_id: row.get(1)?,
                title: row.get(2)?,
                priority: row.get(3)?,
                done: done_int == 1,
                created_at,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn update_task_status(&self, id: &str, client_id: &str, done: bool) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let val = if done { 1 } else { 0 };
        conn.execute("UPDATE tasks SET done = ?1 WHERE id = ?2 AND client_id = ?3", params![val, id, client_id])
    }

    pub fn delete_task(&self, id: &str, client_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE id = ?1 AND client_id = ?2", params![id, client_id])
    }

    pub fn create_event(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events (id, client_id, title, description, start_date, start_time, end_date, end_time, color, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.id,
                event.client_id,
                event.title,
                event.description,
                event.start_date,
                event.start_time,
                event.end_date,
                event.end_time,
                event.color,
                event.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_events(&self, client_id: &str) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM events WHERE client_id = ?1 ORDER BY start_date ASC")?;
        
        let event_iter = stmt.query_map(params![client_id], |row| {
            let created_str: String = row.get(9)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc);

            Ok(Event {
                id: row.get(0)?,
                client_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                start_date: row.get(4)?,
                start_time: row.get(5)?,
                end_date: row.get(6)?,
                end_time: row.get(7)?,
                color: row.get(8)?,
                created_at,
            })
        })?;

        let mut events = Vec::new();
        for ev in event_iter {
            events.push(ev?);
        }
        Ok(events)
    }

    pub fn delete_event(&self, id: &str, client_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM events WHERE id = ?1 AND client_id = ?2", params![id, client_id])
    }

    pub fn get_dashboard_stats(&self, client_id: &str) -> Result<DashboardStats> {
        let conn = self.conn.lock().unwrap();
        
        let emp_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM employees WHERE client_id = ?",
            params![client_id],
            |row| row.get(0),
        )?;

        let salary_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(salary), 0.0) FROM employees WHERE client_id = ?",
            params![client_id],
            |row| row.get(0),
        )?;

        let active_tasks: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE client_id = ? AND done = 0",
            params![client_id],
            |row| row.get(0),
        )?;

        let event_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM events WHERE client_id = ?",
            params![client_id],
            |row| row.get(0),
        )?;

        Ok(DashboardStats {
            total_employees: emp_count,
            monthly_payroll: salary_total,
            active_tasks,
            total_events: event_count,
        })
    }

    pub fn check_health(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let result: i32 = conn.query_row("SELECT 1", [], |r| r.get(0))?;
        Ok(result == 1)
    }

    pub fn backup_db(&self, backup_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.backup(rusqlite::DatabaseName::Main, Path::new(backup_path), None)?;
        Ok(())
    }

    pub fn restore_db(&self, backup_path: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.restore(rusqlite::DatabaseName::Main, Path::new(backup_path), None)?;
        Ok(())
    }

    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("VACUUM", [])?;
        Ok(())
    }

    pub fn execute_raw(&self, query: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute(query, [])
    }
}
