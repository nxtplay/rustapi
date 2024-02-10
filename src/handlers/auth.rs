use tera::{Tera, Context};
use firebase_auth::FirebaseUser;
use actix_web::{web, HttpResponse, Responder, get, http::StatusCode};
use crate::models::user::UserCredentials;
#[get("/hello")]
pub async fn greet(user: FirebaseUser) -> impl Responder {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("Hello {}!", email)
}

#[get("/public")]
pub async fn public() -> impl Responder {
    "ok"
}


pub async fn login(credentials: web::Json<UserCredentials>) -> impl Responder {
    let url = "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=[AIzaSyAR_9FrpHM22VDTwhEHiXfapDk1k5IfiF4]";
    let client = reqwest::Client::new();
    let res = client.post(url)
        .json(&credentials)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                HttpResponse::Ok().json("Login successful")
            } else {
                HttpResponse::Unauthorized().json("Invalid credentials")
            }
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
pub async fn login_page(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    // You can add any context variables here if needed

    match tera.render("login.html", &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

