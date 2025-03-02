use std::env;
use std::path::Path;
use anyhow::Result;

// Note: Since detect_text_with_api makes external API calls,
// we'll focus on testing the response parsing logic with mocked data.

// Create a directory for test fixtures
#[test]
fn test_parse_api_response() {
    let sample_response = r#"{
        "responses": [
            {
                "textAnnotations": [
                    {
                        "description": "Sample Text",
                        "boundingPoly": {
                            "vertices": [
                                { "x": 10, "y": 10 },
                                { "x": 50, "y": 10 },
                                { "x": 50, "y": 40 },
                                { "x": 10, "y": 40 }
                            ]
                        }
                    }
                ]
            }
        ]
    }"#;

    // Test parsing logic
    let response: image_anonymizer::ocr::detection::TextDetectionResponse = 
        serde_json::from_str(sample_response).expect("Failed to parse sample JSON");
    
    assert_eq!(response.responses.len(), 1);
    assert_eq!(response.responses[0].text_annotations.len(), 1);
    assert_eq!(response.responses[0].text_annotations[0].description, "Sample Text");
    
    let vertices = &response.responses[0].text_annotations[0].bounding_poly.as_ref().unwrap().vertices;
    assert_eq!(vertices.len(), 4);
    assert_eq!(vertices[0].x, 10);
    assert_eq!(vertices[0].y, 10);
    assert_eq!(vertices[1].x, 50);
    assert_eq!(vertices[1].y, 10);
}

#[test]
fn test_api_key_environment_variable() {
    // Save original value to restore later
    let original = env::var("GCP_API_KEY").ok();
    
    // Unset the variable
    env::remove_var("GCP_API_KEY");
    
    // Test that our function properly checks for the environment variable
    let result = image_anonymizer::ocr::detection::detect_text_with_api(Path::new("nonexistent.jpg"));
    assert!(result.is_err(), "Function should error when GCP_API_KEY is not set");
    
    // Check specific error message
    let err = result.unwrap_err().to_string();
    assert!(err.contains("GCP_API_KEY"), "Error should mention missing API key");
    
    // Restore original value if it existed
    if let Some(value) = original {
        env::set_var("GCP_API_KEY", value);
    }
}
