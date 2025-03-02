use anyhow::Result;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use image_anonymizer::ocr::detection::{BoundingPoly, TextAnnotation, Vertex};
use image_anonymizer::ocr::masking::mask_text;

#[test]
fn test_mask_text_with_empty_annotations() -> Result<()> {
    // Create a test image
    let mut img = DynamicImage::new_rgba8(100, 100);
    // Make it all white
    for y in 0..100 {
        for x in 0..100 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // No annotations
    let annotations = vec![];
    let additional_masks = vec![];

    // Apply masking
    mask_text(&mut img, &annotations, &additional_masks)?;

    // Image should remain unchanged
    for y in 0..100 {
        for x in 0..100 {
            let pixel = img.get_pixel(x, y);
            assert_eq!(pixel, Rgba([255, 255, 255, 255]));
        }
    }

    Ok(())
}

#[test]
fn test_mask_text_with_annotations() -> Result<()> {
    // Create a test image
    let mut img = DynamicImage::new_rgba8(100, 100);
    // Make it all white
    for y in 0..100 {
        for x in 0..100 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Create a test annotation (that would normally be detected as sensitive)
    let annotations = vec![TextAnnotation {
        description: "test@example.com".to_string(),
        bounding_poly: Some(BoundingPoly {
            vertices: vec![
                Vertex { x: 10, y: 10 },
                Vertex { x: 50, y: 10 },
                Vertex { x: 50, y: 40 },
                Vertex { x: 10, y: 40 },
            ],
        }),
    }];

    // Force masking with additional masks that match our text
    let additional_masks = vec!["test@example.com".to_string()];

    // Apply masking
    mask_text(&mut img, &annotations, &additional_masks)?;

    // Check that pixels in the bounding box are masked (black with alpha)
    let expected_color = Rgba([0, 0, 0, 128]);
    for y in 10..=40 {
        for x in 10..=50 {
            let pixel = img.get_pixel(x, y);
            assert_eq!(
                pixel, expected_color,
                "Pixel at ({}, {}) should be masked",
                x, y
            );
        }
    }

    // Check that pixels outside the bounding box are unchanged
    let white = Rgba([255, 255, 255, 255]);
    assert_eq!(img.get_pixel(9, 10), white);
    assert_eq!(img.get_pixel(51, 10), white);
    assert_eq!(img.get_pixel(10, 9), white);
    assert_eq!(img.get_pixel(10, 41), white);

    Ok(())
}
