pub mod ocr;

use anyhow::{Context, Result};
use clap::Parser;
use image_anonymizer::process_image;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info};

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

/// Main function
///
/// # Returns
///
/// * `Result<()>` - The result of the program
///
/// # Errors
///
/// * `anyhow::Error` - If the program fails
///
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
