//! Blocked word/pattern lists for content moderation.
//!
//! These lists are intentionally kept short and focused on the most harmful
//! patterns. The system prompt handles nuance; this layer catches obvious
//! violations quickly and cheaply.

/// Patterns that indicate violent or weapons-related content.
pub const VIOLENCE_PATTERNS: &[&str] = &[
    "kill", "murder", "stab", "shoot", "bomb", "explode", "torture",
    "убить", "убийство", "взорвать", "бомба", "оружие", "пистолет",
    "вбити", "зброя", "matar", "asesinar", "bomba", "arma",
];

/// Patterns that indicate sexual or adult content.
pub const SEXUAL_PATTERNS: &[&str] = &[
    "porn", "xxx", "nude", "naked", "sex",
    "порно", "секс", "голый", "голая",
    "порно", "сексу", "porno", "sexo", "desnudo",
];

/// Patterns that indicate drug, alcohol, or tobacco content.
pub const SUBSTANCE_PATTERNS: &[&str] = &[
    "cocaine", "heroin", "meth", "marijuana", "weed",
    "кокаин", "героин", "наркотик", "марихуана",
    "cocaína", "heroína", "droga", "marihuana",
];

/// Patterns that indicate hacking or exploitation (blocked unless parent-approved).
pub const HACKING_PATTERNS: &[&str] = &[
    "hack", "exploit", "crack password", "sql injection", "xss",
    "взлом", "хакер", "взломать",
    "hackear", "explotar",
];

/// Patterns that indicate self-harm or suicide.
pub const SELF_HARM_PATTERNS: &[&str] = &[
    "suicide", "kill myself", "self-harm", "cut myself",
    "суицид", "покончить с собой", "порезать себя",
    "suicidio", "hacerme daño",
];

/// Patterns that indicate hate speech or bullying.
pub const HATE_PATTERNS: &[&str] = &[
    "hate you", "stupid", "ugly", "loser", "worthless",
    "ненавижу тебя", "тупой", "уродливый", "лузер",
    "te odio", "estúpido", "feo", "perdedor",
];

/// All pattern lists grouped by category for iteration.
pub fn all_categories() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        ("violence", VIOLENCE_PATTERNS),
        ("sexual", SEXUAL_PATTERNS),
        ("substances", SUBSTANCE_PATTERNS),
        ("hacking", HACKING_PATTERNS),
        ("self_harm", SELF_HARM_PATTERNS),
        ("hate_speech", HATE_PATTERNS),
    ]
}
