mod detection;
mod masking;

pub use detection::{BoundingPoly, TextAnnotation, Vertex, detect_text_with_api};
pub use masking::mask_text;
