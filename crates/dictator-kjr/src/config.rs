//! Configuration for Kim Jong Rails decree
//!
//! The Party sets parameters. The people obey.

/// Configuration for Kim Jong Rails decree
#[derive(Debug, Clone)]
pub struct KjrConfig {
    pub min_emojis: usize,
    pub min_function_lines: usize,
    pub max_imports: usize,
    pub max_identifier_length: usize,
    pub banned_words: Vec<String>,
    pub praise_keywords: Vec<String>,
}

impl Default for KjrConfig {
    fn default() -> Self {
        Self {
            min_emojis: 2,
            min_function_lines: 10,
            max_imports: 5,
            max_identifier_length: 12,
            banned_words: vec![
                "profit".into(),
                "revenue".into(),
                "cost".into(),
                "market".into(),
                "budget".into(),
                "capital".into(),
                "stock".into(),
            ],
            praise_keywords: vec![
                "kim".into(),
                "supreme".into(),
                "glorious".into(),
                "dictator".into(),
                "motherland".into(),
                "party".into(),
                "glory".into(),
            ],
        }
    }
}
