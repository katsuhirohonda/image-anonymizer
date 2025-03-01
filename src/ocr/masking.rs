use crate::ocr::detection::BoundingPoly;
use anyhow::Result;
use image::{DynamicImage, GenericImage, Rgba};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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

fn is_sensitive_text(
    text: &str,
    criteria: &SensitiveTextCriteria,
    additional_texts: &[String],
) -> bool {
    // First check additional_texts for direct matches (this is fast and doesn't require API calls)
    if additional_texts.iter().any(|t| text.contains(t)) {
        info!("Text matched additional mask pattern: {}", text);
        return true;
    }

    // Skip very short text as they're unlikely to be sensitive
    if text.len() < 3 {
        return false;
    }

    if criteria.api_keys
        && text.len() > 20
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '@')
    {
        info!("Detected potential API key: {}", text);
        return true;
    }

    return false;

    //// Call Gemini API to analyze the text
    //match analyze_text_sensitivity(text) {
    //    Ok(is_sensitive) => {
    //        if is_sensitive {
    //            info!("Gemini identified sensitive text: {}", text);
    //            true
    //        } else {
    //            false
    //        }
    //    }
    //    Err(err) => {
    //        warn!(
    //            "Error calling Gemini API, defaulting to non-sensitive: {}",
    //            err
    //        );
    //        // If API fails, fall back to safety and consider it sensitive if it looks like
    //        // an email, phone number, or contains numeric sequences that might be cards/IDs
    //        text.contains('@')
    //            || text.contains('-')
    //            || (text.chars().filter(|c| c.is_numeric()).count() > 8)
    //    }
    //}
}

pub fn mask_text(
    image: &mut DynamicImage,
    annotations: &[TextAnnotation],
    additional_masks: &[String],
) -> Result<()> {
    let criteria = SensitiveTextCriteria::default();

    info!("Masking sensitive text in image");
    let mut masked_count = 0;

    for annotation in annotations {
        // Using is_sensitive_text function requires async context
        // For compatibility with existing code, we can use reqwest's blocking API in analyze_text_sensitivity
        // Or refactor the entire codebase to use async Rust
        let is_sensitive = is_sensitive_text(&annotation.description, &criteria, additional_masks);

        if is_sensitive {
            mask_annotation(image, annotation)?;
            masked_count += 1;
        }
    }

    info!("Masked {} sensitive text regions", masked_count);
    Ok(())
}

fn mask_annotation(image: &mut DynamicImage, annotation: &TextAnnotation) -> Result<()> {
    let empty_poly = BoundingPoly { vertices: vec![] };
    let vertices = &annotation
        .bounding_poly
        .as_ref()
        .unwrap_or(&empty_poly)
        .vertices;

    let min_x = vertices.iter().map(|v| v.x).min().unwrap_or(0).max(0) as u32;
    let min_y = vertices.iter().map(|v| v.y).min().unwrap_or(0).max(0) as u32;
    let max_x = vertices.iter().map(|v| v.x).max().unwrap_or(0).max(0) as u32;
    let max_y = vertices.iter().map(|v| v.y).max().unwrap_or(0).max(0) as u32;

    let (width, height) = (image.width(), image.height());

    let max_x = max_x.min(width - 1);
    let max_y = max_y.min(height - 1);

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
