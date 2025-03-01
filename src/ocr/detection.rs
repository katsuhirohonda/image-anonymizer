use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct TextDetectionResponse {
    pub responses: Vec<Response>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub text_annotations: Vec<TextAnnotation>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextAnnotation {
    pub description: String,
    pub bounding_poly: BoundingPoly,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BoundingPoly {
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize)]
struct TextDetectionRequest {
    requests: Vec<Request>,
}

#[derive(Debug, Serialize)]
struct Request {
    image: Image,
    features: Vec<Feature>,
}

#[derive(Debug, Serialize)]
struct Image {
    content: String,
}

#[derive(Debug, Serialize)]
struct Feature {
    #[serde(rename = "type")]
    feature_type: String,
    max_results: i32,
}

pub fn detect_text_with_api(image_path: &Path) -> Result<Vec<TextAnnotation>> {
    let api_key = env::var("GCP_API_KEY").context("GCP_API_KEY environment variable not set")?;

    info!("image_path: {}", image_path.display());
    let image_data = std::fs::read(image_path).context("Failed to read image file")?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    let request = TextDetectionRequest {
        requests: vec![Request {
            image: Image {
                content: base64_image,
            },
            features: vec![Feature {
                feature_type: "DOCUMENT_TEXT_DETECTION".to_string(),
                max_results: 100,
            }],
        }],
    };

    let client = Client::new();
    let response = client
        .post(&format!(
            "https://vision.googleapis.com/v1/images:annotate?key={}",
            api_key
        ))
        .json(&request)
        .send()
        .context("Failed to send request to Google Cloud Vision API")?;

    let response_body: TextDetectionResponse = response
        .json()
        .context("Failed to parse Google Cloud Vision API response")?;

    info!("Response: {:?}", response_body);

    if response_body.responses.is_empty() {
        error!("No responses from Google Cloud Vision API");
        anyhow::bail!("No responses from Google Cloud Vision API");
    }

    let annotations = response_body.responses[0].text_annotations.clone();
    info!("Detected {} text annotations", annotations.len());

    Ok(annotations)
}
