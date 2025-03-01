use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    top_p: f32,
    top_k: i32,
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Debug, Deserialize)]
struct CandidatePart {
    text: String,
}

/// Analyzes text for sensitive information using Google Gemini API
pub fn analyze_text_sensitivity(text: &str) -> Result<bool> {
    let api_key = env::var("GCP_API_KEY").context("GCP_API_KEY environment variable not set")?;

    info!("Analyzing text sensitivity with Gemini: {}", text);

    let prompt = format!(
        "Analyze the following text and determine if it contains sensitive information such as: \
        API keys, credentials, email addresses, phone numbers, credit card numbers, \
        personal names, or any other confidential data. \
        Do not explain. Respond with only 'true' if it contains sensitive information, or 'false' if it doesn't.\n\n\
        Text to analyze: \"{}\"",
        text
    );

    let request = GeminiRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part { text: prompt }],
        }],
        generation_config: GenerationConfig {
            temperature: 0.0,
            top_p: 0.1,
            top_k: 1,
            max_output_tokens: 5,
        },
    };

    let client = Client::new();
    let response = client
        .post(&format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
            api_key
        ))
        .json(&request)
        .send()
        .context("Failed to send request to Google Gemini API")?;

    let response_status = response.status();
    if !response_status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Could not read error response".to_string());
        error!(
            "Gemini API request failed with status {}: {}",
            response_status, error_text
        );
        anyhow::bail!("Gemini API request failed with status {}", response_status);
    }

    let response_body: GeminiResponse = response
        .json()
        .context("Failed to parse Google Gemini API response")?;

    if response_body.candidates.is_empty() {
        error!("No candidates in Gemini API response");
        anyhow::bail!("No candidates in Gemini API response");
    }

    let result_text = response_body.candidates[0]
        .content
        .parts
        .first()
        .map(|part| part.text.trim().to_lowercase())
        .unwrap_or_default();

    info!("Gemini sensitivity analysis result: {}", result_text);

    match result_text.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => {
            info!("Unexpected response from Gemini API: {}", result_text);
            // Default to treating as sensitive if we get an unexpected response
            Ok(true)
        }
    }
}
