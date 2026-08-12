//! Finding sensitive values in text.
//!
//! Deliberately written without a regex engine. The formats that matter here —
//! card numbers, IBANs, provider keys — are defined by structure and checksums,
//! not by patterns: a regex matches sixteen digits, but only Luhn tells you
//! whether they are a card number or an order reference. Scanning explicitly
//! also keeps the false-positive rate low enough to mask by default, which is
//! the only setting that protects anyone.

use serde::{Deserialize, Serialize};

/// A class of sensitive data, so policy can differ per kind.
///
/// A credit card and an email address both matter, but not equally: masking
/// every email would break ordinary work like "draft a reply to this address",
/// whereas a card number has no business reaching a model at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataKind {
    /// Provider API keys and tokens.
    ApiKey,
    /// Payment card numbers (Luhn-valid).
    CreditCard,
    /// International bank account numbers.
    Iban,
    /// Email addresses.
    Email,
    /// IPv4 addresses.
    IpAddress,
    /// Private key material (PEM blocks).
    PrivateKey,
}

impl DataKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DataKind::ApiKey => "api_key",
            DataKind::CreditCard => "credit_card",
            DataKind::Iban => "iban",
            DataKind::Email => "email",
            DataKind::IpAddress => "ip_address",
            DataKind::PrivateKey => "private_key",
        }
    }
}

/// One sensitive value located in a piece of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: DataKind,
    /// Byte range in the scanned text.
    pub start: usize,
    pub end: usize,
    /// The matched text itself.
    pub value: String,
}

/// Luhn checksum, the thing that separates a card number from any other digits.
fn luhn_valid(digits: &[u8]) -> bool {
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let mut sum = 0u32;
    for (i, d) in digits.iter().rev().enumerate() {
        let mut v = u32::from(*d);
        if i % 2 == 1 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    sum.is_multiple_of(10)
}

/// Known provider key prefixes, and how long the key runs after them.
///
/// Prefix-anchored rather than "any long random string", because the latter
/// also matches commit hashes, UUIDs and base64 payloads — and a gateway that
/// masks those makes the product unusable while looking like it is working.
const KEY_PREFIXES: &[&str] = &[
    "sk-or-v1-", // OpenRouter
    "sk-ant-",   // Anthropic
    "sk-proj-",  // OpenAI project keys
    "sk-",       // OpenAI and compatible
    "ghp_",      // GitHub personal
    "gho_",      // GitHub OAuth
    "github_pat_",
    "gsk_",   // Groq
    "AKIA",   // AWS access key id
    "AIza",   // Google
    "xoxb-",  // Slack bot
    "xoxp-",  // Slack user
    "hf_",    // Hugging Face
    "r8_",    // Replicate
    "csk-",   // Cerebras
    "pplx-",  // Perplexity
    "glpat-", // GitLab
];

fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Locate every sensitive value in `text`.
///
/// Findings never overlap: the longest match at a given position wins, so a
/// card number inside a longer token is not reported twice.
pub fn scan(text: &str) -> Vec<Finding> {
    let bytes = text.as_bytes();
    let mut out: Vec<Finding> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if !text.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let rest = &text[i..];

        // PEM private key blocks — matched whole, because half a key is still
        // a leak and the header alone tells you the rest follows.
        if rest.starts_with("-----BEGIN") {
            if let Some(hdr_end) = rest.find("-----END") {
                if rest[..hdr_end].contains("PRIVATE KEY") {
                    let end_marker = rest[hdr_end..]
                        .find("-----\n")
                        .or_else(|| rest[hdr_end..].find("-----"))
                        .map(|p| hdr_end + p + 5)
                        .unwrap_or(rest.len());
                    out.push(Finding {
                        kind: DataKind::PrivateKey,
                        start: i,
                        end: i + end_marker,
                        value: rest[..end_marker].to_string(),
                    });
                    i += end_marker;
                    continue;
                }
            }
        }

        // Provider API keys.
        if let Some(prefix) = KEY_PREFIXES.iter().find(|p| rest.starts_with(**p)) {
            let mut end = prefix.len();
            while end < rest.len() && rest[end..].chars().next().is_some_and(is_key_char) {
                end += rest[end..].chars().next().unwrap().len_utf8();
            }
            // Require real entropy after the prefix, or "sk-" in prose matches.
            if end - prefix.len() >= 16 {
                out.push(Finding {
                    kind: DataKind::ApiKey,
                    start: i,
                    end: i + end,
                    value: rest[..end].to_string(),
                });
                i += end;
                continue;
            }
        }

        let c = rest.chars().next().unwrap();

        // Email addresses.
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '%' || c == '+' || c == '-' {
            if let Some(f) = scan_email(text, i) {
                let adv = f.end - f.start;
                out.push(f);
                i += adv;
                continue;
            }
        }

        // IBAN: two letters, two check digits, then up to 30 alphanumerics.
        if c.is_ascii_uppercase() {
            if let Some(f) = scan_iban(text, i) {
                let adv = f.end - f.start;
                out.push(f);
                i += adv;
                continue;
            }
        }

        // Digit runs: card numbers (Luhn) or IPv4.
        if c.is_ascii_digit() {
            if let Some(f) = scan_ipv4(text, i) {
                let adv = f.end - f.start;
                out.push(f);
                i += adv;
                continue;
            }
            if let Some(f) = scan_card(text, i) {
                let adv = f.end - f.start;
                out.push(f);
                i += adv;
                continue;
            }
        }

        i += c.len_utf8();
    }
    out
}

fn word_boundary_before(text: &str, at: usize) -> bool {
    if at == 0 {
        return true;
    }
    match text[..at].chars().next_back() {
        Some(p) => !p.is_ascii_alphanumeric() && p != '_' && p != '.' && p != '-' && p != '+',
        None => true,
    }
}

fn scan_email(text: &str, start: usize) -> Option<Finding> {
    if !word_boundary_before(text, start) {
        return None;
    }
    let rest = &text[start..];
    let at = rest.find('@')?;
    if at == 0 || at > 64 {
        return None;
    }
    let local = &rest[..at];
    if !local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c))
    {
        return None;
    }
    let after = &rest[at + 1..];
    let mut dlen = 0usize;
    for ch in after.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
            dlen += ch.len_utf8();
        } else {
            break;
        }
    }
    let domain = &after[..dlen];
    // Must have a dot and a plausible TLD, or "user@localhost" style noise matches.
    let tld = domain.rsplit('.').next().unwrap_or("");
    if !domain.contains('.') || tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let domain = domain.trim_end_matches('.');
    let end = start + at + 1 + domain.len();
    Some(Finding {
        kind: DataKind::Email,
        start,
        end,
        value: text[start..end].to_string(),
    })
}

fn scan_iban(text: &str, start: usize) -> Option<Finding> {
    if !word_boundary_before(text, start) {
        return None;
    }
    let rest = &text[start..];
    let mut chars = rest.chars();
    let c0 = chars.next()?;
    let c1 = chars.next()?;
    let d0 = chars.next()?;
    let d1 = chars.next()?;
    if !(c0.is_ascii_uppercase()
        && c1.is_ascii_uppercase()
        && d0.is_ascii_digit()
        && d1.is_ascii_digit())
    {
        return None;
    }
    let mut end = 4usize;
    while end < rest.len() {
        let ch = rest[end..].chars().next().unwrap();
        if ch.is_ascii_alphanumeric() && end < 34 {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end < 15 {
        return None;
    }
    Some(Finding {
        kind: DataKind::Iban,
        start,
        end: start + end,
        value: rest[..end].to_string(),
    })
}

fn scan_ipv4(text: &str, start: usize) -> Option<Finding> {
    if !word_boundary_before(text, start) {
        return None;
    }
    let rest = &text[start..];
    let mut end = 0usize;
    let mut octets = 0usize;
    let mut cur: u32 = 0;
    let mut cur_len = 0usize;
    for ch in rest.chars() {
        if ch.is_ascii_digit() {
            cur = cur * 10 + ch.to_digit(10).unwrap();
            cur_len += 1;
            if cur > 255 || cur_len > 3 {
                return None;
            }
            end += 1;
        } else if ch == '.' && cur_len > 0 && octets < 3 {
            octets += 1;
            cur = 0;
            cur_len = 0;
            end += 1;
        } else {
            break;
        }
    }
    if octets != 3 || cur_len == 0 {
        return None;
    }
    // A trailing dot belongs to the sentence, not the address.
    let value = rest[..end].trim_end_matches('.');
    Some(Finding {
        kind: DataKind::IpAddress,
        start,
        end: start + value.len(),
        value: value.to_string(),
    })
}

fn scan_card(text: &str, start: usize) -> Option<Finding> {
    if !word_boundary_before(text, start) {
        return None;
    }
    let rest = &text[start..];
    let mut digits: Vec<u8> = Vec::new();
    let mut end = 0usize;
    for ch in rest.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch.to_digit(10).unwrap() as u8);
            end += 1;
        } else if (ch == ' ' || ch == '-') && !digits.is_empty() && digits.len() < 19 {
            // Cards are commonly written in groups; stop if the separator does
            // not lead back into digits.
            let next = rest[end + ch.len_utf8()..].chars().next();
            if next.is_some_and(|n| n.is_ascii_digit()) {
                end += ch.len_utf8();
            } else {
                break;
            }
        } else {
            break;
        }
        if digits.len() > 19 {
            return None;
        }
    }
    if !luhn_valid(&digits) {
        return None;
    }
    Some(Finding {
        kind: DataKind::CreditCard,
        start,
        end: start + end,
        value: rest[..end].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<DataKind> {
        scan(text).into_iter().map(|f| f.kind).collect()
    }

    #[test]
    fn finds_provider_keys() {
        let t = "use sk-or-v1-abcdef0123456789abcdef and ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ012345";
        assert_eq!(kinds(t), vec![DataKind::ApiKey, DataKind::ApiKey]);
    }

    /// "sk-" appears in ordinary prose. Masking it would be worse than useless:
    /// the user would see mangled text and stop trusting the feature.
    #[test]
    fn ignores_short_lookalikes() {
        assert!(scan("the sk-1 label and gh_x").is_empty());
    }

    #[test]
    fn card_numbers_need_a_valid_checksum() {
        // Luhn-valid test number.
        assert_eq!(
            kinds("pay with 4242 4242 4242 4242"),
            vec![DataKind::CreditCard]
        );
        // Same shape, checksum deliberately wrong -> an order number, not a card.
        assert!(scan("order 4242 4242 4242 4243").is_empty());
    }

    #[test]
    fn finds_emails_but_not_every_at_sign() {
        assert_eq!(
            kinds("write to a.b+c@example.com now"),
            vec![DataKind::Email]
        );
        assert!(scan("mention @someone in chat").is_empty());
    }

    #[test]
    fn finds_ipv4_and_rejects_impossible_octets() {
        assert_eq!(kinds("host 192.168.1.10 up"), vec![DataKind::IpAddress]);
        assert!(scan("version 999.1.1.1 here").is_empty());
    }

    #[test]
    fn finds_iban() {
        assert_eq!(
            kinds("send to DE89370400440532013000 today"),
            vec![DataKind::Iban]
        );
    }

    #[test]
    fn finds_private_key_blocks() {
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIBOgIBAAJBAK\n-----END RSA PRIVATE KEY-----";
        assert_eq!(kinds(pem), vec![DataKind::PrivateKey]);
    }

    #[test]
    fn clean_text_produces_nothing() {
        assert!(scan("Please summarise the attached quarterly report.").is_empty());
    }

    #[test]
    fn findings_do_not_overlap() {
        let t = "card 4242424242424242 mail a@b.co ip 10.0.0.1";
        let f = scan(t);
        for w in f.windows(2) {
            assert!(w[0].end <= w[1].start, "overlapping findings: {:?}", f);
        }
    }
}
