use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("input is empty")]
    EmptyInput,

    #[error("OCR request failed: {0}")]
    OcrFailed(String),

    #[error("HTTP error during OCR: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to decode base64 image: {0}")]
    Base64Decode(String),

    #[error("invalid CSV: {0}")]
    InvalidCsv(String),
}
