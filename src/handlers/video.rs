
// videos.rs
use actix_web::{web, HttpResponse, HttpRequest, Responder};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use std::env;
use crate::models::video::{Video, VideoData}; // This imports the Video struct for use


pub async fn fetch_videos(
    pool: web::Data<Pool<PostgresConnectionManager<NoTls>>>,
) -> impl Responder {
    let conn = pool.get().await.expect("Failed to get DB connection from pool");
    let rows = conn.query("SELECT description, videouid FROM Videos", &[])
        .await
        .expect("Failed to execute query");

    let videos: Vec<Video> = rows.iter().map(|row| Video {
        title: row.get("description"),
        video_uid: row.get("videouid"),
    }).collect();

    HttpResponse::Ok().json(videos) // Respond with JSON
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

