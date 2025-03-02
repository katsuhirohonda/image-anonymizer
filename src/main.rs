pub mod ocr;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

use ocr::detection::detect_text_with_api;
use ocr::masking::mask_text;

#[derive(Parser, Debug)]
#[command(author, version, about = "A tool to mask sensitive content in images")]
struct Args {
    #[arg(required = true)]
    input_file: PathBuf,

    #[arg(short, long, default_value = "./output")]
    output_dir: PathBuf,

    #[arg(short, long)]
    mask_texts: Option<String>,

    #[arg(short, long)]
    api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MaskText {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SensitiveTextsResponse {
    sensitive_texts: Vec<String>,
}

fn main() -> Result<()> {
    // Try to load .env file for environment variables (optional)
    match dotenv::dotenv() {
        Ok(_) => info!("Loaded environment from .env file"),
        Err(_) => info!("No .env file found, using environment variables"),
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(false)
        .with_level(true)
        .try_init()
        .expect("Failed to initialize logger");

    let args = Args::parse();

    if std::env::var("GCP_API_KEY").is_err() {
        error!("GCP_API_KEY environment variable is not set");
        anyhow::bail!("GCP_API_KEY environment variable is not set");
    }

    if !args.input_file.exists() {
        error!("Input file does not exist: {:?}", args.input_file);
        anyhow::bail!("Input file does not exist: {:?}", args.input_file);
    }

    if !args.output_dir.exists() {
        debug!("Creating output directory: {:?}", args.output_dir);
        fs::create_dir_all(&args.output_dir).context("Failed to create output directory")?;
    }

    process_image(
        &args.input_file,
        &args.output_dir,
        args.mask_texts.as_deref(),
    )
    .context("Failed to process image")?;

    info!("Image processing completed successfully");
    Ok(())
}

/// Process an image to mask sensitive text
///
/// # Arguments
///
/// * `input_path` - The path to the input image
/// * `output_dir` - The directory to save the output image
/// * `mask_texts` - The texts to mask
///
/// # Returns
///
/// * `Result<()>` - The result of the image processing
///
/// # Errors
///
/// * `anyhow::Error` - If the image processing fails
///
fn process_image(input_path: &Path, output_dir: &Path, mask_texts: Option<&str>) -> Result<()> {
    info!("Image processing started");
    info!("Reading input image: {:?}", input_path);
    let mut img = image::open(input_path).context("Failed to open input image")?;

    let file_name = input_path
        .file_name()
        .context("Invalid input filename")?
        .to_str()
        .context("Non-UTF8 filename")?;

    let output_path = output_dir.join(format!("masked_{}", file_name));

    let additional_masks = if let Some(texts) = mask_texts {
        texts
            .split(',')
            .map(|text| text.trim().to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let annotations = detect_text_with_api(input_path).context("Failed to detect text in image")?;

    if annotations.is_empty() {
        debug!("No text detected in the image");
    } else {
        debug!("Detected {} text annotations", annotations.len());

        mask_text(&mut img, &annotations, &additional_masks).context("Failed to mask text")?;
    }

    img.save(&output_path)
        .context("Failed to save output image")?;

    info!("Saved processed image to: {:?}", output_path);
    Ok(())
}
