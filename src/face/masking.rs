use anyhow::Result;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use tracing::{debug, info};

use super::detection::FaceAnnotation;

/// Mask faces in an image with pixelation (mosaic effect)
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
    // Fixed mosaic pixel size (bigger = more pixelated)
    let pixel_size = 16;

    info!(
        "Masking {} faces in image with mosaic effect",
        face_annotations.len()
    );

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

            // Make sure we don't go out of bounds
            let max_x = max_x.min(width - 1);
            let max_y = max_y.min(height - 1);

            // Calculate how many mosaic blocks we'll have
            let blocks_x = ((max_x - min_x) / pixel_size).max(1);
            let blocks_y = ((max_y - min_y) / pixel_size).max(1);

            // Apply mosaic effect by creating pixelated blocks
            for block_y in 0..blocks_y {
                for block_x in 0..blocks_x {
                    // Calculate block boundaries
                    let block_start_x = min_x + block_x * pixel_size;
                    let block_start_y = min_y + block_y * pixel_size;
                    let block_end_x = (block_start_x + pixel_size).min(max_x);
                    let block_end_y = (block_start_y + pixel_size).min(max_y);

                    // Calculate average color for the block
                    let mut r_sum = 0u32;
                    let mut g_sum = 0u32;
                    let mut b_sum = 0u32;
                    let mut a_sum = 0u32;
                    let mut pixel_count = 0u32;

                    for y in block_start_y..block_end_y {
                        for x in block_start_x..block_end_x {
                            if x < width && y < height {
                                let pixel = image.get_pixel(x, y);
                                r_sum += pixel[0] as u32;
                                g_sum += pixel[1] as u32;
                                b_sum += pixel[2] as u32;
                                a_sum += pixel[3] as u32;
                                pixel_count += 1;
                            }
                        }
                    }

                    if pixel_count > 0 {
                        let avg_pixel = Rgba([
                            (r_sum / pixel_count) as u8,
                            (g_sum / pixel_count) as u8,
                            (b_sum / pixel_count) as u8,
                            (a_sum / pixel_count) as u8,
                        ]);

                        // Fill the block with the average color
                        for y in block_start_y..block_end_y {
                            for x in block_start_x..block_end_x {
                                if x < width && y < height {
                                    image.put_pixel(x, y, avg_pixel);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Face masking with mosaic effect completed");
    Ok(())
}
