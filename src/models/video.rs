// models/video.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub title: String,
    pub video_uid: String,
}
// Assuming you have a struct that represents the data for videos_data
// Let's say it's something like this
#[derive(Serialize)]
pub struct VideoData {
    play: String,
    playtype: String,
    result: String,
    // ... other fields as needed
}




