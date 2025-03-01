use anyhow::Result;
use image::{DynamicImage, GenericImage, Rgba};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::detection::TextAnnotation;

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
    _criteria: &SensitiveTextCriteria,
    additional_texts: &[String],
) -> bool {
    if additional_texts.iter().any(|t| text.contains(t)) {
        return true;
    }

    // TODO: add more criteria

    false
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
        if is_sensitive_text(&annotation.description, &criteria, additional_masks) {
            mask_annotation(image, annotation)?;
            masked_count += 1;
        }
    }

    info!("Masked {} sensitive text regions", masked_count);
    Ok(())
}

fn mask_annotation(image: &mut DynamicImage, annotation: &TextAnnotation) -> Result<()> {
    let vertices = &annotation.bounding_poly.vertices;

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
