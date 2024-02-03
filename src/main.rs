use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer};
use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv::dotenv;
use serde::Serialize;
use std::env;
use tera::{Context, Tera};
use tokio_postgres::NoTls;
#[derive(Serialize)]
struct Video {
    title: String,
    video_uid: String,
}

/* 
 * This function is used to get the upload URL from the Cloudflare Stream api
 * It is used to upload the video to the cloudflare Stream
 * The function is called when the user uploads a video to the server
 * The function returns the upload URL
 */
async fn get_upload_url(req: HttpRequest,  pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>) ->Result<HttpResponse, actix_web::Error>{
    print!("TEST");

    let cloudflare_account_id = env::var("cloudflare_account_id").expect("cloudflare_account_id must be set");
    let cloudflare_api_token = env::var("cloudflare_api_token").expect("cloudflare_api_token must be set");
    let endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/stream?direct_user=true",
        cloudflare_account_id
        );

    let client = reqwest::Client::new();
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", cloudflare_api_token))
        .header("Tus-Resumable", "1.0.0")
        .header("Upload-Length", req.headers().get("Upload-Length").unwrap().to_str().unwrap())
        .header("Upload-Metadata", req.headers().get("Upload-Metadata").unwrap().to_str().unwrap())
        .send()
        .await
        .unwrap();

    let video_uid = response.headers().get("stream-media-id").map(|v| v.to_str().unwrap());
    println!("Video UID: {:?}", video_uid);


    let destination = response.headers().get("Location").unwrap().to_str().unwrap();

    //let video_uid = response.headers().get("stream-media-id").map(|v| v.to_str().unwrap());

    // Get a connection from the pool
    let conn = pool.get().await.map_err(actix_web::error::ErrorInternalServerError)?;

    // Insert video UID into the database
   if let Some(uid) = video_uid {
        let team_id: i32 = 1; // Assuming teamid is an integer
        let uploaded_by: i32 = 1; // Assuming uploadedby is an integer
        let angle: &str = "default_angle";
        let description: &str = "Default description"; // Replace with actual description
        let visibility: &str = "public"; // Replace with actual visibility setting
        let game: &str = "default_game"; // Replace with actual game
        let play: i32 = 1; // Replace with actual play

        conn.execute(
            "INSERT INTO Videos (teamid, videouid, angle, uploadedby, description, visibility, game, play) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[&team_id, &uid, &angle, &uploaded_by, &description, &visibility, &game, &play],
            ).await.map_err(actix_web::error::ErrorInternalServerError)?;    
   }
    Ok(HttpResponse::Ok()
        .insert_header(("Access-Control-Expose-Headers", "Location"))
        .insert_header(("Access-Control-Allow-Headers", "*"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Location", destination))
        .finish())

}
/* 
 * This function is used to render the index page
 * The function is called when the user visits the index page
 * The function returns the index page
 */
async fn index(
    // The pool is used to get a connection to the database
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,

    // The tera is used to render the HTML template
    tera: web::Data<Tera>,
    ) -> Result<HttpResponse, actix_web::Error> {

    // Create a new context to hold the data that will be used to render the template
    let mut context = Context::new();
    
    // Get a connection to the database
    let conn = pool
        .get()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;


    // Query the database for the videos
    let rows = conn
        .query("SELECT description, videouid FROM Videos", &[])
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    println!("{}", rows.len());

    // Create a list to hold the videos
    let mut video_list: Vec<Video> = vec![];

    // Iterate over the rows and create a Video struct for each row
    for row in rows {
        let video = Video {
            title: row.get(0),
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

    // Insert the videos_data into the context
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
