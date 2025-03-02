use anyhow::Result;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use image_anonymizer::face::detection::{BoundingPoly, FaceAnnotation, Vertex};
use image_anonymizer::face::masking::mask_faces;

#[test]
fn test_mask_faces_with_empty_annotations() -> Result<()> {
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

    // Apply masking
    mask_faces(&mut img, &annotations)?;

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
fn test_mask_faces_with_annotations() -> Result<()> {
    // Create a test image
    let mut img = DynamicImage::new_rgba8(100, 100);
    // Make it all white
    for y in 0..100 {
        for x in 0..100 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Create a test annotation simulating a face
    let annotations = vec![FaceAnnotation {
        bounding_poly: Some(BoundingPoly {
            vertices: vec![
                Vertex { x: 20, y: 20 },
                Vertex { x: 60, y: 20 },
                Vertex { x: 60, y: 80 },
                Vertex { x: 20, y: 80 },
            ],
        }),
        landmarks: None,
        detection_confidence: Some(0.95),
    }];

    // Apply masking
    mask_faces(&mut img, &annotations)?;

    // Check that pixels in the bounding box are masked (black with alpha)
    let expected_color = Rgba([0, 0, 0, 180]);
    for y in 20..=80 {
        for x in 20..=60 {
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
    assert_eq!(img.get_pixel(19, 20), white);
    assert_eq!(img.get_pixel(61, 20), white);
    assert_eq!(img.get_pixel(20, 19), white);
    assert_eq!(img.get_pixel(20, 81), white);

    Ok(())
}

#[test]
fn test_parse_face_api_response() {
    let sample_response = r#"{
        "responses": [
            {
                "faceAnnotations": [
                    {
                        "boundingPoly": {
                            "vertices": [
                                { "x": 10, "y": 10 },
                                { "x": 100, "y": 10 },
                                { "x": 100, "y": 100 },
                                { "x": 10, "y": 100 }
                            ]
                        },
                        "detectionConfidence": 0.95
                    }
                ]
            }
        ]
    }"#;

    // Test parsing logic
    let response: image_anonymizer::face::detection::FaceDetectionResponse =
        serde_json::from_str(sample_response).expect("Failed to parse sample JSON");

    assert_eq!(response.responses.len(), 1);
    assert_eq!(response.responses[0].face_annotations.len(), 1);
    assert!(response.responses[0].face_annotations[0].detection_confidence.unwrap() > 0.9);

    let vertices = &response.responses[0].face_annotations[0]
        .bounding_poly
        .as_ref()
        .unwrap()
        .vertices;
    assert_eq!(vertices.len(), 4);
    assert_eq!(vertices[0].x, 10);
    assert_eq!(vertices[0].y, 10);
    assert_eq!(vertices[1].x, 100);
    assert_eq!(vertices[1].y, 10);
}
