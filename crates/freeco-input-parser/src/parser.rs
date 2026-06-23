use crate::{
    error::ParseError,
    types::{ShoppingItem, ShoppingList},
};

/// Parses free-form text into a [`ShoppingList`].
///
/// Handles the most common real-world formats users type or say:
/// - Comma-separated: `"oat milk, vegan cheese, olive oil"`
/// - Newline-separated: one item per line
/// - Numbered lists: `"1. Oat milk\n2. Vegan cheese"`
/// - Bullet lists: `"- Oat milk\n• Vegan cheese"`
/// - Mixed quantity/unit: `"2L oat milk"`, `"500g vegan cheese"`
pub struct TextParser;

impl TextParser {
    /// Parse a free-form text string into a [`ShoppingList`].
    pub fn parse_text(text: &str) -> Result<ShoppingList, ParseError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Decide splitting strategy: if there are newlines, split on lines;
        // otherwise split on commas or semicolons.
        let raw_items: Vec<&str> = if trimmed.contains('\n') {
            trimmed.lines().collect()
        } else if trimmed.contains(',') {
            trimmed.split(',').collect()
        } else if trimmed.contains(';') {
            trimmed.split(';').collect()
        } else {
            // Single item or space-separated — treat as one item.
            vec![trimmed]
        };

        let items: Vec<ShoppingItem> = raw_items
            .into_iter()
            .filter_map(|raw| parse_single_item(raw.trim()))
            .collect();

        if items.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        Ok(ShoppingList::new(items, "text"))
    }

    /// Parse CSV text. The first column is treated as the item name; an
    /// optional second column is quantity; an optional third column is unit.
    /// A header row is skipped if the first cell is non-numeric text that
    /// doesn't look like a product name (contains "name", "item", "product").
    pub fn parse_csv(csv: &str) -> Result<ShoppingList, ParseError> {
        let trimmed = csv.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let mut items = Vec::new();
        for (line_idx, line) in trimmed.lines().enumerate() {
            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            if cols.is_empty() {
                continue;
            }
            let name_col = cols[0];
            // Skip header row
            if line_idx == 0 {
                let lower = name_col.to_lowercase();
                if lower.contains("name") || lower.contains("item") || lower.contains("product") {
                    continue;
                }
            }
            if name_col.is_empty() {
                continue;
            }
            let raw = line.to_string();
            let mut item = ShoppingItem::new(normalize_name(name_col), raw);

            if let Some(qty_str) = cols.get(1) {
                if let Ok(qty) = qty_str.parse::<f32>() {
                    item.quantity = qty;
                }
            }
            if let Some(unit_str) = cols.get(2) {
                item.unit = unit_str.to_string();
            }
            items.push(item);
        }

        if items.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        Ok(ShoppingList::new(items, "csv"))
    }
}

/// Parse a single text fragment (one line or one comma-split segment) into a
/// [`ShoppingItem`], or `None` if the fragment is junk.
fn parse_single_item(raw: &str) -> Option<ShoppingItem> {
    if raw.is_empty() {
        return None;
    }

    // Strip leading bullet / list marker: "- ", "• ", "* ", "1. ", "12) " etc.
    let stripped = strip_list_marker(raw);
    if stripped.is_empty() {
        return None;
    }

    // Try to extract quantity + unit from the front: "2L oat milk", "500g cheese"
    let (quantity, unit, name_raw) = extract_quantity_unit(stripped);

    let name = normalize_name(name_raw);
    if name.is_empty() {
        return None;
    }

    let mut item = ShoppingItem::new(name, raw.to_string());
    item.quantity = quantity;
    item.unit = unit;
    Some(item)
}

/// Strip leading list markers: bullets, numbers, dashes.
fn strip_list_marker(s: &str) -> &str {
    // Numbered: "1. ", "12) ", "3- "
    let s = if let Some(pos) = s.find(|c: char| c == '.' || c == ')' || (c == '-' && !s.starts_with('-'))) {
        let before = &s[..pos];
        if before.chars().all(|c| c.is_ascii_digit()) && pos < 4 {
            s[pos + 1..].trim_start()
        } else {
            s
        }
    } else {
        s
    };
    // Bullets: "- ", "• ", "* ", "– "
    let s = s.trim_start_matches(['-', '•', '*', '–', '·']).trim_start();
    s
}

/// Extract a leading quantity and unit from text like "2L oat milk".
/// Returns (quantity, unit, remaining_text).
fn extract_quantity_unit(s: &str) -> (f32, String, &str) {
    // Pattern: optional digits, optional decimal, optional unit chars, then space
    let mut num_end = 0;
    let chars: Vec<char> = s.chars().collect();

    // Collect digits (and optional single decimal point)
    let mut seen_dot = false;
    for &c in &chars {
        if c.is_ascii_digit() {
            num_end += c.len_utf8();
        } else if c == '.' && !seen_dot {
            seen_dot = true;
            num_end += c.len_utf8();
        } else {
            break;
        }
    }

    if num_end == 0 {
        return (1.0, String::new(), s);
    }

    let qty: f32 = s[..num_end].parse().unwrap_or(1.0);
    let unit_end;

    // Collect letters immediately after the number (unit like "L", "kg", "g", "pcs")
    let remainder = &s[num_end..];
    let unit_chars: String = remainder
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect();

    if !unit_chars.is_empty() && unit_chars.len() <= 4 {
        unit_end = num_end + unit_chars.len();
    } else {
        // No attached unit — number was a standalone quantity
        return (qty, String::new(), s[num_end..].trim_start());
    }

    let after_unit = s[unit_end..].trim_start();
    (qty, unit_chars, after_unit)
}

/// Lowercase and trim a product name.
fn normalize_name(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '(' || c == ')')
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── text parsing ─────────────────────────────────────────────────────────

    #[test]
    fn comma_separated_list() {
        let list = TextParser::parse_text("oat milk, vegan cheese, olive oil").unwrap();
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.items[0].name, "oat milk");
        assert_eq!(list.items[1].name, "vegan cheese");
        assert_eq!(list.items[2].name, "olive oil");
    }

    #[test]
    fn newline_separated_list() {
        let list = TextParser::parse_text("oat milk\nvegan cheese\nolive oil").unwrap();
        assert_eq!(list.items.len(), 3);
    }

    #[test]
    fn numbered_list() {
        let text = "1. Oat milk\n2. Vegan cheese\n3. Olive oil";
        let list = TextParser::parse_text(text).unwrap();
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.items[0].name, "oat milk");
    }

    #[test]
    fn bullet_list_with_dashes() {
        let text = "- oat milk\n- vegan cheese\n- olive oil";
        let list = TextParser::parse_text(text).unwrap();
        assert_eq!(list.items.len(), 3);
    }

    #[test]
    fn bullet_list_with_unicode_bullets() {
        let text = "• oat milk\n• vegan cheese";
        let list = TextParser::parse_text(text).unwrap();
        assert_eq!(list.items.len(), 2);
    }

    #[test]
    fn quantity_and_unit_extracted() {
        let list = TextParser::parse_text("2L oat milk, 500g vegan cheese").unwrap();
        assert_eq!(list.items[0].quantity, 2.0);
        assert_eq!(list.items[0].unit, "L");
        assert_eq!(list.items[0].name, "oat milk");
        assert_eq!(list.items[1].quantity, 500.0);
        assert_eq!(list.items[1].unit, "g");
    }

    #[test]
    fn single_item_no_separator() {
        let list = TextParser::parse_text("organic oat milk").unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].name, "organic oat milk");
        assert_eq!(list.items[0].quantity, 1.0);
    }

    #[test]
    fn empty_input_errors() {
        assert!(TextParser::parse_text("").is_err());
        assert!(TextParser::parse_text("   ").is_err());
    }

    #[test]
    fn french_list() {
        let list =
            TextParser::parse_text("du lait d'avoine, fromage vegan, de l'huile d'olive").unwrap();
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.language, crate::Language::French);
    }

    #[test]
    fn semicolon_separated() {
        let list = TextParser::parse_text("oat milk; vegan cheese; olive oil").unwrap();
        assert_eq!(list.items.len(), 3);
    }

    #[test]
    fn names_are_lowercase() {
        let list = TextParser::parse_text("Organic Oat Milk").unwrap();
        assert_eq!(list.items[0].name, "organic oat milk");
    }

    // ── CSV parsing ──────────────────────────────────────────────────────────

    #[test]
    fn csv_basic() {
        let csv = "oat milk,2,L\nvegan cheese,1,pcs";
        let list = TextParser::parse_csv(csv).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].quantity, 2.0);
        assert_eq!(list.items[0].unit, "L");
    }

    #[test]
    fn csv_skips_header() {
        let csv = "product name,quantity,unit\noat milk,1,L";
        let list = TextParser::parse_csv(csv).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].name, "oat milk");
    }

    #[test]
    fn csv_empty_errors() {
        assert!(TextParser::parse_csv("").is_err());
    }

    #[test]
    fn csv_name_only_column() {
        let csv = "oat milk\nvegan cheese";
        let list = TextParser::parse_csv(csv).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].quantity, 1.0);
    }
}
