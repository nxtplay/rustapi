use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use tera::{Tera, Context};
use std::error::Error;
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, ListObjectsV2Request};  
use std::env;
use s3_presign::{Credentials, Presigner, Bucket, put};
use url::Url;
use chrono::{Utc, Duration as ChronoDuration};
use tokio_postgres::{NoTls, Error as PostgresError};
use dotenv::dotenv;
use anyhow::Result;
#[derive(Serialize)]
struct Video {
    title: String,
    video_uid: String,
}

#[derive(Debug)]
struct CloudFlareVideo {
    VideoId: i32,
    VideoPath: String,
    // Add other fields as necessary
}


//Allows User to upload videos
// This handler returns a list of videos with links to CloudFront URLs.

async fn index(tera: web::Data<Tera>) -> Result<HttpResponse, actix_web::Error> {

    let mut context = Context::new();

     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to the database
   
  let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT VideoID, videouid FROM Videos", &[])
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    println!("{}",rows.len());
    let mut video_list: Vec<Video> = vec![];

     for row in rows {
        let video = Video {
            title: row.get(1),
            video_uid: row.get(1),
        };
        video_list.push(video);
//        println!("{:?}", video.VideoPath);
    }

 
    // Insert the stringified URL into the context
   // let video_list = list_videos_from_s3(bucket_name, prefix).await.unwrap_or_else(|_| vec![]);  // Ensure this is correct

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
    Ok::<HttpResponse, actix_web::Error>(HttpResponse::Ok().content_type("text/html").body(rendered))
}
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the Tera templating engine and point it to the templates directory
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to the database
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let rows = client.query("SELECT VideoID, videouid FROM Videos", &[]).await?;
    println!("{}",rows.len());
     for row in rows {
        let video = CloudFlareVideo {
            VideoId: row.get(0),
            VideoPath: row.get(1),
        };
        println!("{:?}", video.VideoPath);
    }


    let tera = Tera::new("templates/**/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?;

    Ok(())
}
