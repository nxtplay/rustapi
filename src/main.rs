use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use tera::{Tera, Context};
use std::error::Error;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3Client, S3, ListObjectsV2Request};  
use std::env;
use s3_presign::{Credentials, Presigner, Bucket, put};
use url::Url;
use chrono::{Utc, Duration as ChronoDuration};
#[derive(Serialize)]
struct Video {
    title: String,
    cloudfront_url: String,
}   

async fn list_videos_from_s3(bucket_name: &str, prefix: &str) -> Result<Vec<Video>, Box<dyn Error>> {
    let s3_client = S3Client::new(Region::UsWest1); // Change the region as needed
    let mut videos = Vec::new();
    let cloudfront_domain = "d2rk3jlkj5wdvt.cloudfront.net"; // Your CloudFront domain

    let mut continuation_token: Option<String> = None;
    loop {
        let list_obj_req = ListObjectsV2Request {
            bucket: bucket_name.to_string(),
            prefix: Some(prefix.to_string()),
            continuation_token: continuation_token.clone(),
            ..Default::default()
        };

        let result = s3_client.list_objects_v2(list_obj_req).await?;
        if let Some(contents) = result.contents {
            for object in contents {
                if let Some(key) = object.key {
                    println!("{}", key);
                    if key.ends_with(".mp4"){
                    let video_url = format!("https://{}/{}", cloudfront_domain, key);
                    videos.push(Video {
                        title: key.clone(),
                        cloudfront_url: video_url,
                        });
                    }
                }
            }
        }

        match result.is_truncated {
            Some(true) => continuation_token = result.next_continuation_token,
            Some(false) | None => break,
        }
    }

    Ok(videos)
}


//Allows User to upload videos

async fn create_presigned_put_url(bucket_name: &str, object_key: &str, expiration_in_sec: i64) -> Result<Url, Box<dyn std::error::Error>> {
    // Fetch AWS credentials from environment variables
    println!("Testin");
    let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not found in environment");
    let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not found in environment");

    // Your AWS credentials
    let credentials = Credentials::new(
        aws_access_key_id,
        aws_secret_access_key,
        None,  // for session_token if it's optional
    );

    // Initialize Presigner
    let presigner = Presigner::new(credentials, bucket_name, "us-west-1");  // Adjust region as needed

    // Calculate expiration time
    let expiration_time = Utc::now() + ChronoDuration::seconds(expiration_in_sec as i64);

    // Initialize Bucket
    let bucket = Bucket::new("us-west-1", bucket_name);  // Adjust region as needed

    // Generate presigned PUT URL
    let presigned_url_str = presigner.put(object_key, expiration_time.timestamp())
        .ok_or("Failed to generate presigned URL")?;
    
    // Convert the presigned_url_str (String) to a Url
    let presigned_url = Url::parse(&presigned_url_str)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    println!("TESTING TESTING");
    Ok(presigned_url)
}

// This handler returns a list of videos with links to CloudFront URLs.

async fn index(tera: web::Data<Tera>) -> impl Responder {
let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not found in environment");
    let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not found in environment");

    // Your AWS credentials
    let credentials = Credentials::new(
        aws_access_key_id,
        aws_secret_access_key,
        None,  // for session_token if it's optional
    );

   let bucket_name = "testbucketnxtplay";
    let prefix = "";
    let expiration_in_sec = 3600; // Presigned URL validity duration in seconds
    let object_key = "result1.mp4";
    
    let bucket = Bucket::new("us-west-1", bucket_name);  // Adjust region as needed
    let presigned_url = put(&credentials, &bucket, object_key, expiration_in_sec);

    let mut context = Context::new();

    // Insert the stringified URL into the context
    context.insert("presigned_url", &presigned_url);
    
    let video_list = list_videos_from_s3(bucket_name, prefix).await.unwrap_or_else(|_| vec![]);  // Ensure this is correct

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
async fn main() -> std::io::Result<()> {
    // Initialize the Tera templating engine and point it to the templates directory
    let tera = Tera::new("templates/**/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
