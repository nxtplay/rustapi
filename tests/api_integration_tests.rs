use reqwest;
use tokio;
use serde_json;

// Mark the test function as async
#[tokio::test]
async fn test_videos_endpoint() {
    let client = reqwest::Client::new();
    let res = client.get("http://localhost:8080/api/videos")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Expected success status, got {}", res.status());

    // Parse the response body as JSON
    let videos: Vec<serde_json::Value> = res.json().await.expect("Failed to parse JSON");

    // Check if the response is an array and not empty
    assert!(!videos.is_empty(), "Expected non-empty list of videos");

    // Optionally, check the structure of the first video item
    if let Some(first_video) = videos.get(0) {
        assert!(first_video["title"].is_string(), "Expected video title to be a string");
        assert!(first_video["video_uid"].is_string(), "Expected video_uid to be a string");
    } else {
        panic!("Expected at least one video in response");
    }
}
