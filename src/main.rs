use actix_cors::Cors;
use actix_web::{web, http, App, HttpServer};
use actix_web::http::header;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;
use firebase_auth::FirebaseAuth;
mod models; // This line imports the models directory module, thanks to models/mod.rs
mod handlers;
use crate::handlers::video::{get_upload_url, fetch_videos};
use crate::handlers::auth::login_page;

// The main function is the entry point for the program
// It is used to start the server and listen for incoming requests
// The server listens for incoming requests and routes them to the appropriate handler
// The server is started by calling the run method on the HttpServer struct
// The run method is an asynchronous method and is called within an async block
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the Tera templating engine and point it to the templates directory
    env_logger::init();

    println!("Starting the Rust API server...");
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to the database
    println!("Connecting to the database...");
    println!("Database URL: {}", database_url);
   let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)?;
   let pool = bb8::Pool::builder().build(manager).await.expect("Failed to create pool.");   

    //Initialize Firebase Auth
    let firebase_auth = FirebaseAuth::new("nxtplay-9dbae").await;
    println!("Firebase Auth initialized...");
    println!("Server running at http:// ");
    HttpServer::new(move || {
        let cors = Cors::permissive()
        .allowed_origin("http://localhost:3000") // Adjust this to your client's origin
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST"]) // Specifies which methods are allowed
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
        .max_age(3600);

       App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(firebase_auth.clone())) // Add Firebase Auth as app data
            .route("/login", web::get().to(login_page))
            .route("/api/get-upload-url", web::post().to(get_upload_url))
            .route("/api/videos", web::get().to(fetch_videos))
                   // Example Firebase Auth protected route
    })
    .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}
