use actix_cors::Cors;
use actix_web::{web, HttpResponse, App,HttpServer, Responder, get, http::StatusCode};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use std::env;
use tera::{Tera, Context};
use tokio_postgres::NoTls;
use firebase_auth::{FirebaseAuth, FirebaseUser};
use serde::{Deserialize, Serialize};
use reqwest; // Add `reqwest` to your Cargo.toml dependencies

#[derive(Serialize, Deserialize)] 
struct UserCredentials {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct Video {
    title: String,
    video_uid: String,
}


mod handlers;

// ... other mods ...

use handlers::{index, get_upload_url};

#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("Hello {}!", email)
}

#[get("/public")]
async fn public() -> impl Responder {
    "ok"
}


async fn login(credentials: web::Json<UserCredentials>) -> impl Responder {
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
async fn login_page(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    // You can add any context variables here if needed

    match tera.render("login.html", &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
// The main function is the entry point for the program
// It is used to start the server and listen for incoming requests
// The server is started on port 5000
// The server listens for incoming requests and routes them to the appropriate handler
// The server is started by calling the run method on the HttpServer struct
// The run method is an asynchronous method and is called within an async block
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the Tera templating engine and point it to the templates directory
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to the database
    let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)?;
    let pool = Pool::builder().build(manager).await?;
    
    //Initialize Firebase Auth
    let firebase_auth = FirebaseAuth::new("nxtplay-9dbae").await;
    let tera = Tera::new("templates/**/*").unwrap();
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(firebase_auth.clone())) // Add Firebase Auth as app data
            .route("/login", web::get().to(login_page))
            .route("/", web::get().to(index))
            .route("/api/get-upload-url", web::post().to(get_upload_url))
                   // Example Firebase Auth protected route
            .service(greet)
            .service(public)
    })
    .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
