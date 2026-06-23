//! # input-parser
//!
//! Converts any user input format into a structured [`ShoppingList`].
//!
//! Handles text, voice transcripts, photos of handwritten lists (via Google
//! Vision OCR), receipts, and uploaded CSV/TXT files — all without an LLM
//! call.  Pure Rust parsing logic keeps this fast, cheap, and deterministic.
//!
//! # Example
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() {
//! use freeco_input_parser::{InputParser, InputSource};
//!
//! let parser = InputParser::new("google-vision-api-key");
//!
//! // Plain text
//! let list = parser.parse(InputSource::Text("oat milk, vegan cheese, olive oil".into())).await.unwrap();
//! assert_eq!(list.items.len(), 3);
//!
//! // Voice transcript (same pipeline as text)
//! let list = parser.parse(InputSource::Voice("buy oat milk and vegan cheese".into())).await.unwrap();
//! assert_eq!(list.items.len(), 2);
//! # }
//! ```

pub mod error;
pub mod language;
pub mod ocr;
pub mod parser;
pub mod types;

pub use error::ParseError;
pub use language::Language;
pub use ocr::OcrClient;
pub use parser::TextParser;
pub use types::{InputSource, ShoppingItem, ShoppingList};

use reqwest::Client;

/// Entry point for all input formats.
pub struct InputParser {
    ocr: OcrClient,
}

impl InputParser {
    pub fn new(google_vision_api_key: impl Into<String>) -> Self {
        Self {
            ocr: OcrClient::new(google_vision_api_key.into(), Client::new()),
        }
    }

    /// Override the Google Vision base URL (for testing).
    pub fn with_ocr_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.ocr = self.ocr.with_base_url(base_url.into());
        self
    }

    /// Parse any supported input source into a [`ShoppingList`].
    pub async fn parse(&self, source: InputSource) -> Result<ShoppingList, ParseError> {
        match source {
            InputSource::Text(text) => {
                tracing::debug!("parsing text input ({} chars)", text.len());
                TextParser::parse_text(&text)
            }
            InputSource::Voice(transcript) => {
                tracing::debug!("parsing voice transcript ({} chars)", transcript.len());
                // Voice transcripts are already text — same pipeline, note the source
                let mut list = TextParser::parse_text(&transcript)?;
                list.source = "voice".into();
                Ok(list)
            }
            InputSource::ImageBytes(bytes) => {
                tracing::debug!("parsing image via OCR ({} bytes)", bytes.len());
                let text = self.ocr.extract_text_from_bytes(&bytes).await?;
                let mut list = TextParser::parse_text(&text)?;
                list.source = "image_ocr".into();
                Ok(list)
            }
            InputSource::ImageBase64(b64) => {
                tracing::debug!("parsing base64 image via OCR");
                let text = self.ocr.extract_text_from_base64(&b64).await?;
                let mut list = TextParser::parse_text(&text)?;
                list.source = "image_ocr".into();
                Ok(list)
            }
            InputSource::CsvText(csv) => {
                tracing::debug!("parsing CSV input");
                let mut list = TextParser::parse_csv(&csv)?;
                list.source = "csv".into();
                Ok(list)
            }
            InputSource::PlainFileText(text) => {
                tracing::debug!("parsing plain file text");
                let mut list = TextParser::parse_text(&text)?;
                list.source = "file".into();
                Ok(list)
            }
        }
    }
}
