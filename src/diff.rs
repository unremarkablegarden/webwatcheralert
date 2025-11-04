/// Content diffing module
///
/// Compares new content with cached version to detect meaningful changes

use similar::{ChangeTag, TextDiff};

/// Check if content has meaningfully changed
/// Returns true if there are actual content differences (ignoring minor whitespace)
pub fn has_changed(old_content: &str, new_content: &str) -> bool {
    // If strings are exactly equal, no change
    if old_content == new_content {
        return false;
    }

    // Normalize whitespace for comparison
    let old_normalized = normalize_whitespace(old_content);
    let new_normalized = normalize_whitespace(new_content);

    // Check if normalized versions differ
    old_normalized != new_normalized
}

/// Get a human-readable diff summary
#[allow(dead_code)]
pub fn get_diff(old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut changes = Vec::new();
    let mut added_lines = 0;
    let mut removed_lines = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                removed_lines += 1;
                if removed_lines <= 3 {
                    changes.push(format!("- {}", change.value().trim()));
                }
            }
            ChangeTag::Insert => {
                added_lines += 1;
                if added_lines <= 3 {
                    changes.push(format!("+ {}", change.value().trim()));
                }
            }
            ChangeTag::Equal => {}
        }
    }

    if changes.is_empty() {
        return String::from("Content changed (whitespace only)");
    }

    let summary = format!(
        "{} lines added, {} lines removed\n{}",
        added_lines,
        removed_lines,
        changes.join("\n")
    );

    if added_lines > 3 || removed_lines > 3 {
        format!("{}\n... (showing first 3 changes)", summary)
    } else {
        summary
    }
}

/// Normalize whitespace for comparison
fn normalize_whitespace(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
