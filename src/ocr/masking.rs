use crate::ocr::detection::BoundingPoly;
use anyhow::Result;
use image::{DynamicImage, GenericImage, Rgba};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use super::detection::TextAnnotation;
use super::gemini::analyze_text_sensitivity;

#[derive(Debug, Serialize, Deserialize)]
pub struct SensitiveTextCriteria {
    pub api_keys: bool,
    pub emails: bool,
    pub phone_numbers: bool,
    pub credit_cards: bool,
    pub personal_names: bool,
    pub company_names: bool,
}

/// Default criteria for sensitive text
///
/// # Returns
///
/// * `SensitiveTextCriteria` - The default criteria
///
impl Default for SensitiveTextCriteria {
    fn default() -> Self {
        Self {
            api_keys: true,
            emails: true,
            phone_numbers: true,
            credit_cards: true,
            personal_names: true,
            company_names: true,
        }
    }
}

/// Check if a text is sensitive
///
/// # Arguments
///
/// * `text` - The text to check
/// * `criteria` - The criteria for sensitive text
/// * `additional_texts` - Additional texts to check
///
/// # Returns
///
/// * `bool` - True if the text is sensitive, false otherwise
///
/// # Errors
///
/// * `anyhow::Error` - If the text analysis fails
///
fn is_sensitive_text(
    text: &str,
    criteria: &SensitiveTextCriteria,
    additional_texts: &[String],
) -> bool {
    // First check additional_texts for direct matches (this is fast and doesn't require API calls)
    if additional_texts.iter().any(|t| text.contains(t)) {
        debug!("Text matched additional mask pattern: {}", text);
        return true;
    }

    if text.len() < 3 {
        return false;
    }

    if criteria.api_keys
        && text.len() > 20
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '@')
    {
        debug!("Detected potential API key: {}", text);
        return true;
    }

    // Call Gemini API to analyze the text
    match analyze_text_sensitivity(text) {
        Ok(is_sensitive) => {
            if is_sensitive {
                debug!("Gemini identified sensitive text: {}", text);
                true
            } else {
                false
            }
        }
        Err(err) => {
            error!(
                "Error calling Gemini API, defaulting to non-sensitive: {}",
                err
            );
            // If API fails, fall back to safety and consider it sensitive if it looks like
            // an email, phone number, or contains numeric sequences that might be cards/IDs
            text.contains('@')
                || text.contains('-')
                || (text.chars().filter(|c| c.is_numeric()).count() > 8)
        }
    }
}

/// Mask sensitive text in an image
///
/// # Arguments
///
/// * `image` - The image to mask
/// * `annotations` - The annotations to mask
/// * `additional_masks` - Additional masks to check
///
/// # Returns
///
/// * `Result<()>` - The result of the image processing
///
/// # Errors
///
/// * `anyhow::Error` - If the image processing fails
pub fn mask_text(
    image: &mut DynamicImage,
    annotations: &[TextAnnotation],
    additional_masks: &[String],
) -> Result<()> {
    let criteria = SensitiveTextCriteria::default();

    info!("Masking sensitive text in image");

    // skip first annotation because it's usually the whole image text
    // if there is only one annotation, process it
    let annotations_to_process = if annotations.len() > 1 {
        &annotations[1..]
    } else {
        annotations
    };

    // check sensitivity in parallel and collect sensitive annotations
    let sensitive_annotations: Vec<&TextAnnotation> = annotations_to_process
        .par_iter() // parallel iteration
        .filter(|&annotation| {
            is_sensitive_text(&annotation.description, &criteria, additional_masks)
        })
        .collect();

    let masked_count = sensitive_annotations.len();

    // apply mask to sensitive annotations
    // because it's writing to the image, we avoid parallelization and process sequentially
    for annotation in sensitive_annotations {
        mask_annotation(image, annotation)?;
    }

    info!("Masked {} sensitive text regions", masked_count);
    Ok(())
}

/// Mask a text annotation in an image
///
/// # Arguments
///
/// * `image` - The image to mask
/// * `annotation` - The annotation to mask
///
/// # Returns
///
/// * `Result<()>` - The result of the image processing
///
/// # Errors
///
/// * `anyhow::Error` - If the image processing fails
fn mask_annotation(image: &mut DynamicImage, annotation: &TextAnnotation) -> Result<()> {
    let empty_poly = BoundingPoly { vertices: vec![] };
    let vertices = &annotation
        .bounding_poly
        .as_ref()
        .unwrap_or(&empty_poly)
        .vertices;

    if vertices.is_empty() {
        debug!("Skipping annotation with empty bounding polygon");
        return Ok(());
    }

    let min_x = vertices.iter().map(|v| v.x).min().unwrap_or(0).max(0) as u32;
    let min_y = vertices.iter().map(|v| v.y).min().unwrap_or(0).max(0) as u32;
    let max_x = vertices.iter().map(|v| v.x).max().unwrap_or(0).max(0) as u32;
    let max_y = vertices.iter().map(|v| v.y).max().unwrap_or(0).max(0) as u32;

    let (width, height) = (image.width(), image.height());

    let max_x = max_x.min(width - 1);
    let max_y = max_y.min(height - 1);

    let box_width = max_x.saturating_sub(min_x);
    let box_height = max_y.saturating_sub(min_y);

    if box_width > width / 2 || box_height > height / 2 {
        debug!(
            "Skipping oversized bounding box: {}x{}",
            box_width, box_height
        );
        return Ok(());
    }

    let black = Rgba([0, 0, 0, 128]);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if x < width && y < height {
                image.put_pixel(x, y, black);
            }
        }
    }

    Ok(())
}
