use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use reqwest::header;
use serde::Serialize;
use std::env;
use tera::{Context, Tera};
use tokio_postgres::NoTls;
#[derive(Serialize)]
struct Video {
    title: String,
    video_uid: String,
}

//#[derive(Debug)]

//#[post("/api/get-upload-url")]
async fn get_upload_url() -> impl Responder {
    print!("TEST");
    let cloudflare_account_id = "40f4a0b828f1555fa46730f248b7614f";
    let cloudflare_api_token = "FzYQRPc4BcJumALyRpLS3QIC6ziiRKUr9u_WLTlo";
    let endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/stream?direct_user=true",
        cloudflare_account_id
        );

    let client = reqwest::Client::new();
    let response = match client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", cloudflare_api_token))
        .header("Tus-Resumable", "1.0.0")
        .header("Upload-Length", "your_upload_length") // Set this appropriately
        .header("Upload-Metadata", "your_upload_metadata") // Set this as needed
        .send()
        .await
        {
            Ok(res) => res,
            Err(e) => {
                return HttpResponse::InternalServerError().body(format!("API request failed: {}", e))
            }
        };
    println!("{:?}", response);

    // Check if the response is a success
    if !response.status().is_success() {
        return HttpResponse::InternalServerError().body("API request did not succeed");
    }

    let destination = match response.headers().get(header::LOCATION) {
        Some(loc) => match loc.to_str() {
            Ok(url) => url,
            Err(_) => {
                return HttpResponse::InternalServerError().body("Failed to parse LOCATION header")
            }
        },
        None => {
            return HttpResponse::InternalServerError().body("No LOCATION header found in response")
        }
    };

    HttpResponse::Ok()
        .append_header(("Access-Control-Expose-Headers", "Location"))
        .append_header(("Access-Control-Allow-Headers", "*"))
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Location", destination))
        .finish()
}

async fn index(
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,
    tera: web::Data<Tera>,
    ) -> Result<HttpResponse, actix_web::Error> {
    let mut context = Context::new();

    let conn = pool
        .get()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let rows = conn
        .query("SELECT VideoID, videouid FROM Videos", &[])
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    println!("{}", rows.len());
    let mut video_list: Vec<Video> = vec![];

    for row in rows {
        let video = Video {
            title: row.get(1),
            video_uid: row.get(1),
        };
        video_list.push(video);
    }
    // Insert the stringified URL into the context
    context.insert("videos", &video_list);

    // Assuming you have a struct that represents the data for videos_data
    // Let's say it's something like this
    #[derive(Serialize)]
    struct VideoData {
        play: String,
        playtype: String,
        result: String,
        // ... other fields as needed
    }

    // Create some example data for videos_data
    let videos_data = vec![
        VideoData {
            play: "Play 1".to_string(),
            playtype: "Type 1".to_string(),
            result: "Result 1".to_string(),
            // ... other fields as needed
        },
        // ... add more video data as needed
    ];

    context.insert("videos_data", &videos_data); // The list used for the table data

    // Render the template and handle any errors that may occur
    let rendered = match tera.get_ref().render("videos.html", &context) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Template render error: {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // Return the rendered template wrapped in an HttpResponse
    Ok::<HttpResponse, actix_web::Error>(
        HttpResponse::Ok().content_type("text/html").body(rendered),
        )
}
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
    .bind("127.0.0.1:5000")?
        .run()
        .await?;

    Ok(())
}
