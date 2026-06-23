use serde::{Deserialize, Serialize};

use crate::language::Language;

/// A single item on a shopping list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShoppingItem {
    /// Normalised product name (lowercase, trimmed).
    pub name: String,
    /// Quantity requested (1.0 if not specified).
    pub quantity: f32,
    /// Unit (e.g. "L", "kg", "pcs") or empty string.
    pub unit: String,
    /// The raw text fragment this item was parsed from.
    pub raw_text: String,
    /// Detected language of the raw text.
    pub language: Language,
}

impl ShoppingItem {
    pub fn new(name: impl Into<String>, raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let language = Language::detect(&raw);
        Self {
            name: name.into(),
            quantity: 1.0,
            unit: String::new(),
            raw_text: raw,
            language,
        }
    }
}

/// The parsed output from any input source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingList {
    pub items: Vec<ShoppingItem>,
    /// Where the list came from: "text", "voice", "image_ocr", "csv", "file".
    pub source: String,
    /// Dominant language across all items.
    pub language: Language,
}

impl ShoppingList {
    pub fn new(items: Vec<ShoppingItem>, source: impl Into<String>) -> Self {
        let language = dominant_language(&items);
        Self {
            items,
            source: source.into(),
            language,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn dominant_language(items: &[ShoppingItem]) -> Language {
    let fr_count = items.iter().filter(|i| i.language == Language::French).count();
    if fr_count * 2 >= items.len() && !items.is_empty() {
        Language::French
    } else {
        Language::English
    }
}

/// All input formats accepted by [`crate::InputParser::parse`].
pub enum InputSource {
    /// Free-form text (typed by user).
    Text(String),
    /// Web Speech API transcript.
    Voice(String),
    /// Raw image bytes (JPEG/PNG).
    ImageBytes(Vec<u8>),
    /// Base64-encoded image string.
    ImageBase64(String),
    /// CSV text (header optional).
    CsvText(String),
    /// Plain text file contents.
    PlainFileText(String),
}
