use anyhow::Result;
use image::{DynamicImage, GenericImage, Rgba};
use tracing::{debug, info};

use super::detection::FaceAnnotation;

/// Mask faces in an image
///
/// # Arguments
///
/// * `image` - The image to mask
/// * `face_annotations` - The face annotations from Vision API
///
/// # Returns
///
/// * `Result<()>` - The result of the image processing
///
/// # Errors
///
/// * `anyhow::Error` - If the image processing fails
///
pub fn mask_faces(image: &mut DynamicImage, face_annotations: &[FaceAnnotation]) -> Result<()> {
    info!("Masking {} faces in image", face_annotations.len());

    for (idx, annotation) in face_annotations.iter().enumerate() {
        if let Some(bounding_poly) = &annotation.bounding_poly {
            debug!("Masking face #{}", idx + 1);
            
            let vertices = &bounding_poly.vertices;
            if vertices.is_empty() {
                debug!("Skipping face with empty bounding polygon");
                continue;
            }

            let min_x = vertices.iter().map(|v| v.x).min().unwrap_or(0).max(0) as u32;
            let min_y = vertices.iter().map(|v| v.y).min().unwrap_or(0).max(0) as u32;
            let max_x = vertices.iter().map(|v| v.x).max().unwrap_or(0).max(0) as u32;
            let max_y = vertices.iter().map(|v| v.y).max().unwrap_or(0).max(0) as u32;

            let (width, height) = (image.width(), image.height());

            let max_x = max_x.min(width - 1);
            let max_y = max_y.min(height - 1);

            // Use semi-transparent black for masking
            let mask_color = Rgba([0, 0, 0, 180]);

            // Apply pixelation or solid mask
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x < width && y < height {
                        image.put_pixel(x, y, mask_color);
                    }
                }
            }
        }
    }

    info!("Face masking completed");
    Ok(())
}
