# Image Anonymizer

A command-line tool to detect and mask sensitive content in images.

## Features

- Detects and masks sensitive content in images:
  - Text detection via OCR
- Identifies sensitive textual information like:
  - API keys
  - Email addresses
  - Phone numbers
  - Credit card numbers
  - Personal names
  - Company or service names
- Masks detected content with colored rectangles
- Outputs processed images to a specified directory

## Installation

```
cargo install image-anonymizer
```

## Configuration

The application requires API keys to access Google Cloud Platform services. Create a `.env` file in the root directory based on the `.env.template` file:

```
# You need to set these environment variables
# GCP API Key for Google Cloud Platform need to access Gemini API and Cloud Vision API
GCP_API_KEY=
GEMINI_MODEL=gemini-2.0-flash-lite
```

Fill in your GCP API key to enable text detection capabilities.

## Usage

```
privacy-masker [OPTIONS] <INPUT_FILE>
Options:
  -o, --output-dir <DIR>     Output directory for processed images [default: ./output]
  -m, --mask-texts <TEXTS>   Additional texts to mask, comma separated
  -h, --help                 Print help
  -V, --version              Print version
```

## Examples

```
# Process a single image 
image-anonymizer screenshot.png
# Process an image and specify output directory
image-anonymizer --output-dir ./masked_images screenshot.png
# Process an image and mask additional text
image-anonymizer --mask-texts "secret,confidential" screenshot.png
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.