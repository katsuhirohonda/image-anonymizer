use anyhow::Result;
use image::{DynamicImage, GenericImageView, Rgba};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Helper function to create a test image with sample text
fn create_test_image(path: &Path) -> Result<()> {
    let mut img = DynamicImage::new_rgba8(200, 100);
    
    // Fill with white
    for y in 0..100 {
        for x in 0..200 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
    
    // Save the image
    fs::create_dir_all(path.parent().unwrap())?;
    img.save(path)?;
    
    Ok(())
}

// Set up test environment
fn setup() -> Result<PathBuf> {
    // Create a temporary test directory
    let test_dir = PathBuf::from("./test_output");
    fs::create_dir_all(&test_dir)?;
    
    // Create test image
    let test_image_path = test_dir.join("test_image.png");
    create_test_image(&test_image_path)?;
    
    // Set up environment variables if they don't exist
    if env::var("GCP_API_KEY").is_err() {
        env::set_var("GCP_API_KEY", "dummy_api_key_for_testing");
    }
    
    Ok(test_dir)
}

// Cleanup test environment
fn teardown(test_dir: &Path) -> Result<()> {
    fs::remove_dir_all(test_dir)?;
    Ok(())
}

// This test can be run with cargo test -- --ignored
// since it requires API access and can't be run in CI
#[test]
#[ignore]
fn test_process_image_end_to_end() -> Result<()> {
    let test_dir = setup()?;
    let image_path = test_dir.join("test_image.png");
    let output_dir = test_dir.join("output");
    
    // This is now calling the actual function from image-anonymizer
    let result = image_anonymizer::process_image(&image_path, &output_dir, Some("test"));
    
    // Check that processing completed successfully
    assert!(result.is_ok(), "Image processing failed: {:?}", result.err());
    
    // Verify output file exists
    let expected_output = output_dir.join("masked_test_image.png");
    assert!(expected_output.exists(), "Output file does not exist");
    
    // Cleanup
    teardown(&test_dir)?;
    
    Ok(())
}

// Test with invalid input path
#[test]
fn test_process_image_nonexistent_input() -> Result<()> {
    let test_dir = setup()?;
    let nonexistent_path = test_dir.join("nonexistent.png");
    let output_dir = test_dir.join("output");
    
    let result = image_anonymizer::process_image(&nonexistent_path, &output_dir, None);
    
    // Should fail because input doesn't exist
    assert!(result.is_err(), "Should fail with nonexistent input");
    
    // Cleanup
    teardown(&test_dir)?;
    
    Ok(())
}
