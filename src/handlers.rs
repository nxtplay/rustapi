
// handlers.rs

use actix_web::{web, HttpResponse, HttpRequest};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::Serialize;
use tera::{Context, Tera};
use tokio_postgres::NoTls;
use std::env;
use crate::Video; // Assuming Video struct is defined in the main module or another module


/// Renders the index page of the application.
/// 
/// This function is responsible for querying the database to retrieve video data,
/// and then using the Tera templating engine to render the HTML for the index page.
/// It demonstrates how to integrate database operations with web templating in Actix-web.
pub async fn index(
    // Database connection pool for executing queries.
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,

    // The tera is used to render the HTML template
    tera: web::Data<Tera>,
    ) -> Result<HttpResponse, actix_web::Error> {

    // Context for the Tera template, holding the data to be displayed on the page.
    let mut context = Context::new();
    
    // Retrieves a connection from the pool and queries the database for video data.
    // The query specifically fetches the description and UID of each video,
    // which are essential for displaying them on the index page.
    let conn = pool
        .get()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let rows = conn
        .query("SELECT description, videouid FROM Videos", &[])
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;


    // Iterates over the query results, converting each row into a Video object.
    // This transformation is necessary to match the structure expected by the HTML template.
    let mut video_list: Vec<Video> = vec![];
    for row in rows {
        let video = Video {
            title: row.get(0),
            video_uid: row.get(1),
        };
        video_list.push(video);
    }
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

    // Render the index page using the 'videos.html' Tera template.
    // The context provides the data needed to fill out the template.
    // Any rendering errors are handled gracefully.
    let rendered = match tera.get_ref().render("videos.html", &context) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Template render error: {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // Returns the rendered HTML wrapped in an HttpResponse.
    // This is the final output that will be displayed to the user.
    Ok::<HttpResponse, actix_web::Error>(
        HttpResponse::Ok().content_type("text/html").body(rendered),
        )
}


/* 
 * This function is used to get the upload URL from the Cloudflare Stream api
 * It is used to upload the video to the cloudflare Stream
 * The function is called when the user uploads a video to the server
 * The function returns the upload URL
 */
pub async fn get_upload_url(req: HttpRequest,  pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>) ->Result<HttpResponse, actix_web::Error>{
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

