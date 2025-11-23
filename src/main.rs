use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use crate::models::*;
use crate::storage::Storage;

mod models;
mod storage;

struct AppState {
    storage: Arc<Storage>,
    sessions: Mutex<std::collections::HashMap<String, String>>,
}

async fn onboard_client(data: web::Data<AppState>, req: web::Json<OnboardingRequest>) -> impl Responder {
    match req.validate() {
        Ok(_) => {},
        Err(e) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }

    let new_client = Client::new(
        req.business_name.clone(),
        req.business_website.clone(),
        req.business_sector.clone(),
        req.revenue.clone(),
        req.goals.clone(),
        req.email.clone(),
        req.job_title.clone(),
        req.generated_username.clone(),
        req.generated_password.clone(),
    );

    match data.storage.create_client(&new_client) {
        Ok(_) => HttpResponse::Created().json(ApiResponse::success(new_client, "Client created successfully")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Database error: {}", e))),
    }
}

async fn login_client(data: web::Data<AppState>, req: web::Json<LoginRequest>) -> impl Responder {
    match req.validate() {
        Ok(_) => {},
        Err(e) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }

    match data.storage.get_client_by_username(&req.username) {
        Ok(Some(client)) => {
            if client.password_hash == req.password {
                let session_id = uuid::Uuid::new_v4().to_string();
                let mut sessions = data.sessions.lock().unwrap();
                sessions.insert(session_id.clone(), client.id.clone());
                
                let response = LoginResponse {
                    session_id,
                    client_name: client.business_name,
                };
                HttpResponse::Ok().json(ApiResponse::success(response, "Login successful"))
            } else {
                HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"))
            }
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Database error: {}", e))),
    }
}

fn get_client_id_from_header(req: &actix_web::HttpRequest, sessions: &Mutex<std::collections::HashMap<String, String>>) -> Option<String> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");
                let sessions = sessions.lock().unwrap();
                return sessions.get(token).cloned();
            }
        }
    }
    None
}

async fn get_employees(data: web::Data<AppState>, req: actix_web::HttpRequest) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    match data.storage.get_employees(&client_id) {
        Ok(employees) => HttpResponse::Ok().json(ApiResponse::success(employees, "Employees retrieved")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn create_employee(data: web::Data<AppState>, req: actix_web::HttpRequest, body: web::Json<CreateEmployeeRequest>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e));
    }

    let new_emp = Employee::new(
        client_id,
        body.name.clone(),
        body.title.clone(),
        body.salary,
        body.status.clone()
    );

    match data.storage.create_employee(&new_emp) {
        Ok(_) => HttpResponse::Created().json(ApiResponse::success(new_emp, "Employee created")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn delete_employee(data: web::Data<AppState>, req: actix_web::HttpRequest, path: web::Path<String>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };
    let emp_id = path.into_inner();

    match data.storage.delete_employee(&emp_id, &client_id) {
        Ok(deleted) => {
            if deleted > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Employee deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Employee not found"))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn update_employee_payment(data: web::Data<AppState>, req: actix_web::HttpRequest, path: web::Path<String>, body: web::Json<UpdateEmployeePaymentRequest>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };
    let emp_id = path.into_inner();

    match data.storage.update_employee_paid_status(&emp_id, &client_id, body.paid) {
        Ok(updated) => {
            if updated > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Payment status updated"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Employee not found"))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn get_tasks(data: web::Data<AppState>, req: actix_web::HttpRequest) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    match data.storage.get_tasks(&client_id) {
        Ok(tasks) => HttpResponse::Ok().json(ApiResponse::success(tasks, "Tasks retrieved")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn create_task(data: web::Data<AppState>, req: actix_web::HttpRequest, body: web::Json<CreateTaskRequest>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    let new_task = Task::new(client_id, body.title.clone(), body.priority.clone());

    match data.storage.create_task(&new_task) {
        Ok(_) => HttpResponse::Created().json(ApiResponse::success(new_task, "Task created")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn update_task_status(data: web::Data<AppState>, req: actix_web::HttpRequest, path: web::Path<String>, body: web::Json<UpdateTaskRequest>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };
    let task_id = path.into_inner();

    match data.storage.update_task_status(&task_id, &client_id, body.done) {
        Ok(updated) => {
            if updated > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Task updated"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Task not found"))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn delete_task(data: web::Data<AppState>, req: actix_web::HttpRequest, path: web::Path<String>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };
    let task_id = path.into_inner();

    match data.storage.delete_task(&task_id, &client_id) {
        Ok(deleted) => {
            if deleted > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Task deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Task not found"))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn get_events(data: web::Data<AppState>, req: actix_web::HttpRequest) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    match data.storage.get_events(&client_id) {
        Ok(events) => HttpResponse::Ok().json(ApiResponse::success(events, "Events retrieved")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn create_event(data: web::Data<AppState>, req: actix_web::HttpRequest, body: web::Json<CreateEventRequest>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e));
    }

    let new_event = Event::new(
        client_id,
        body.title.clone(),
        body.description.clone().unwrap_or_default(),
        body.start_date.clone(),
        body.start_time.clone().unwrap_or_default(),
        body.end_date.clone(),
        body.end_time.clone().unwrap_or_default(),
        body.color.clone(),
    );

    match data.storage.create_event(&new_event) {
        Ok(_) => HttpResponse::Created().json(ApiResponse::success(new_event, "Event created")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn delete_event(data: web::Data<AppState>, req: actix_web::HttpRequest, path: web::Path<String>) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };
    let event_id = path.into_inner();

    match data.storage.delete_event(&event_id, &client_id) {
        Ok(deleted) => {
            if deleted > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Event deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Event not found"))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn get_dashboard(data: web::Data<AppState>, req: actix_web::HttpRequest) -> impl Responder {
    let client_id = match get_client_id_from_header(&req, &data.sessions) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid session")),
    };

    match data.storage.get_dashboard_stats(&client_id) {
        Ok(stats) => HttpResponse::Ok().json(ApiResponse::success(stats, "Dashboard stats retrieved")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error: {}", e))),
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "OK".to_string(),
        uptime: 0, 
        database_connected: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(ApiResponse::<()>::error("Route not found"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = "qads.db";
    let storage = match Storage::new(db_path) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let app_state = web::Data::new(AppState {
        storage: storage.clone(),
        sessions: Mutex::new(std::collections::HashMap::new()),
    });

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/onboarding", web::post().to(onboard_client))
            .route("/login", web::post().to(login_client))
            .service(
                web::scope("/api")
                    .route("/dashboard", web::get().to(get_dashboard))
                    .route("/employees", web::get().to(get_employees))
                    .route("/employees", web::post().to(create_employee))
                    .route("/employees/{id}", web::delete().to(delete_employee))
                    .route("/employees/{id}/payment", web::put().to(update_employee_payment))
                    .route("/tasks", web::get().to(get_tasks))
                    .route("/tasks", web::post().to(create_task))
                    .route("/tasks/{id}", web::put().to(update_task_status))
                    .route("/tasks/{id}", web::delete().to(delete_task))
                    .route("/events", web::get().to(get_events))
                    .route("/events", web::post().to(create_event))
                    .route("/events/{id}", web::delete().to(delete_event))
            )
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
