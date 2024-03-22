use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;
use firebase_auth::FirebaseAuth;
mod models; // This line imports the models directory module, thanks to models/mod.rs
mod handlers;
use crate::handlers::video::{get_upload_url, fetch_videos};
use crate::handlers::auth::login_page;

use log::{debug, error, log_enabled, info, Level};
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

    //let database_url = "postgres://willmetz:Raventhree2020@host.docker.internal:5432/nxtplaydatabase";
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//    let database_url = "postgres://postgres:uXmbs3dNgEH0ACKntrMQ@nxtplaydatabase.cxee8am8a74x.us-west-1.rds.amazonaws.com:5432/nxtplaydatabase";
    // Connect to the database
    let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)?;
    println!("Connecting to the database...");
   // let pool = Pool::builder().max_size(16).build(manager).await?;
    let pool = Pool::builder()
    .max_size(16) // Example size
    .build(manager).await
    .map_err(|e| {
        println!("Failed to create connection pool: {}", e);
        e // You can also transform the error here
    })?;
    //Initialize Firebase Auth
    let firebase_auth = FirebaseAuth::new("nxtplay-9dbae").await;
    println!("Firebase Auth initialized...");
    println!("Server running at http:// ");
    HttpServer::new(move || {
        let cors = Cors::permissive()// Adjust according to your needs
            .allowed_origin("http://localhost:3000") // React app's origin
            .allowed_origin("http://134.173.248.4") // React app's origin
//            .allowed_methods(vec!["GET", "POST",  "OPTIONS"])
 //           .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
  //          .allowed_header(http::header::CONTENT_TYPE)
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
