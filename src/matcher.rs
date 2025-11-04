/// Keyword matching module
///
/// Searches for keywords in content and returns matches with context

#[derive(Debug, Clone)]
pub struct KeywordMatch {
    pub keyword: String,
    pub context: String,
}

/// Search for keywords in content (case-insensitive)
/// Returns matches with surrounding context (up to 100 chars before/after)
pub fn find_keywords(content: &str, keywords: &[String]) -> Vec<KeywordMatch> {
    let mut matches = Vec::new();
    let content_lower = content.to_lowercase();

    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();

        // Find all occurrences of this keyword
        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&keyword_lower) {
            let absolute_pos = start + pos;

            // Extract context around the match
            let context_start = absolute_pos.saturating_sub(100);
            let context_end = (absolute_pos + keyword.len() + 100).min(content.len());
            let context = &content[context_start..context_end];

            // Clean up the context (remove extra whitespace, newlines)
            let context_cleaned = context
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            // Truncate if too long and add ellipsis
            let context_display = if context_cleaned.len() > 200 {
                format!("...{}...", &context_cleaned[..197])
            } else if context_start > 0 && context_end < content.len() {
                format!("...{}...", context_cleaned)
            } else if context_start > 0 {
                format!("{}...", context_cleaned)
            } else if context_end < content.len() {
                format!("...{}", context_cleaned)
            } else {
                context_cleaned
            };

            matches.push(KeywordMatch {
                keyword: keyword.clone(),
                context: context_display,
            });

            // Move past this match to find next occurrence
            start = absolute_pos + keyword.len();
        }
    }

    matches
}
