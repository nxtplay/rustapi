use actix_cors::Cors;
use actix_web::{web, App,HttpServer};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use serde::Serialize;
use std::env;
use tera::Tera;
use tokio_postgres::NoTls;
#[derive(Serialize)]
struct Video {
    title: String,
    video_uid: String,
}


mod handlers;

// ... other mods ...

use handlers::{index, get_upload_url};
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

    let tera = Tera::new("templates/**/*").unwrap();
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/api/get-upload-url", web::post().to(get_upload_url))
    })
    .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
