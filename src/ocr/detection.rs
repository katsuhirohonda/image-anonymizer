use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct TextDetectionResponse {
    pub responses: Vec<Response>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(default)]
    #[serde(rename = "textAnnotations")]
    pub text_annotations: Vec<TextAnnotation>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextAnnotation {
    pub description: String,
    #[serde(default)]
    #[serde(rename = "boundingPoly")]
    pub bounding_poly: Option<BoundingPoly>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BoundingPoly {
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Vertex {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
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

/// Detect text in an image using the Google Cloud Vision API
///
/// # Arguments
///
/// * `image_path` - The path to the image file
///
/// # Returns
///
/// * `Result<Vec<TextAnnotation>>` - The detected text annotations
///
/// # Errors
///
/// * `anyhow::Error` - If the image processing fails
///
pub fn detect_text_with_api(image_path: &Path) -> Result<Vec<TextAnnotation>> {
    let api_key = env::var("GCP_API_KEY").context("GCP_API_KEY environment variable not set")?;
    debug!("image_path: {}", image_path.display());

    let image_data = std::fs::read(image_path).context("Failed to read image file")?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    let request = TextDetectionRequest {
        requests: vec![Request {
            image: Image {
                content: base64_image,
            },
            features: vec![Feature {
                feature_type: "TEXT_DETECTION".to_string(),
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

    let response_text = response.text().context("Failed to get response text")?;

    if response_text.len() > 1000 {
        debug!(
            "Response text (first 1000 chars): {}",
            &response_text[..1000]
        );
        debug!("Response text length: {}", response_text.len());
    } else {
        debug!("Response text: {}", &response_text);
    }

    let response_body: TextDetectionResponse = serde_json::from_str(&response_text)
        .context("Failed to parse Google Cloud Vision API response")?;

    if response_body.responses.is_empty() {
        error!("No responses from Google Cloud Vision API");
        anyhow::bail!("No responses from Google Cloud Vision API");
    }

    let annotations = response_body.responses[0].text_annotations.clone();
    debug!("Detected {} text annotations", annotations.len());

    Ok(annotations)
}
