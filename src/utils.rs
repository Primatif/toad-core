/// Simple token estimator based on characters / 4 approximation.
/// [IMPORTANT] This is a heuristic for token usage and NOT a precise 
/// count for any specific model. It is intended for budget-aware truncation.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

/// Truncates text to stay within a token limit.
pub fn truncate_by_tokens(text: &str, limit: usize) -> String {
    let char_limit = limit * 4;
    if text.chars().count() > char_limit {
        let mut truncated: String = text.chars().take(char_limit).collect();
        truncated.push_str("\n... [Truncated due to token budget]");
        truncated
    } else {
        text.to_string()
    }
}
