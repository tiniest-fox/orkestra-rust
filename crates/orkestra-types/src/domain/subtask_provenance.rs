//! Subtask provenance extracted from git commit trailers.

/// Structured provenance data parsed from commit trailers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SubtaskProvenance {
    pub parent_id: String,
    pub subtask_id: String,
    pub title: String,
}

/// Parse subtask trailers from a commit body.
///
/// Scans only the final paragraph (after the last blank line) to avoid
/// false positives from AI-generated body text. Returns `None` if any
/// of the three required trailers is missing.
pub fn parse_subtask_trailers(body: &str) -> Option<SubtaskProvenance> {
    let last_paragraph = body
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .last()?;

    let mut parent_id: Option<String> = None;
    let mut subtask_id: Option<String> = None;
    let mut title: Option<String> = None;

    for line in last_paragraph.lines() {
        if let Some(val) = line.strip_prefix("Subtrak-of: ") {
            parent_id = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("Subtrak-id: ") {
            subtask_id = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("Subtrak-title: ") {
            title = Some(val.trim().to_string());
        }
    }

    Some(SubtaskProvenance {
        parent_id: parent_id?,
        subtask_id: subtask_id?,
        title: title?,
    })
}

/// Format subtask provenance as a git trailer block.
pub fn format_subtask_trailers(parent_id: &str, subtask_id: &str, title: &str) -> String {
    format!("Subtrak-of: {parent_id}\nSubtrak-id: {subtask_id}\nSubtrak-title: {title}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_body(parent: &str, id: &str, title: &str) -> String {
        format!("Subtrak-of: {parent}\nSubtrak-id: {id}\nSubtrak-title: {title}")
    }

    #[test]
    fn happy_path_all_three_trailers() {
        let body = make_body("parent-1", "subtask-1", "Add feature");
        let result = parse_subtask_trailers(&body).unwrap();
        assert_eq!(result.parent_id, "parent-1");
        assert_eq!(result.subtask_id, "subtask-1");
        assert_eq!(result.title, "Add feature");
    }

    #[test]
    fn missing_parent_id_returns_none() {
        let body = "Subtrak-id: subtask-1\nSubtrak-title: Add feature";
        assert!(parse_subtask_trailers(body).is_none());
    }

    #[test]
    fn missing_subtask_id_returns_none() {
        let body = "Subtrak-of: parent-1\nSubtrak-title: Add feature";
        assert!(parse_subtask_trailers(body).is_none());
    }

    #[test]
    fn missing_title_returns_none() {
        let body = "Subtrak-of: parent-1\nSubtrak-id: subtask-1";
        assert!(parse_subtask_trailers(body).is_none());
    }

    #[test]
    fn trailers_in_non_final_paragraph_returns_none() {
        // Trailers in body text above a blank line — final paragraph has unrelated content
        let body = "Subtrak-of: parent-1\nSubtrak-id: subtask-1\nSubtrak-title: Add feature\n\nSome unrelated text after the trailers";
        assert!(parse_subtask_trailers(body).is_none());
    }

    #[test]
    fn title_with_special_characters() {
        let body = make_body(
            "parent-1",
            "subtask-1",
            "Add: feature \"quoted\" — unicode ñ",
        );
        let result = parse_subtask_trailers(&body).unwrap();
        assert_eq!(result.title, "Add: feature \"quoted\" — unicode ñ");
    }

    #[test]
    fn empty_body_returns_none() {
        assert!(parse_subtask_trailers("").is_none());
        assert!(parse_subtask_trailers("   ").is_none());
    }

    #[test]
    fn body_with_only_trailers() {
        let body = "Subtrak-of: p\nSubtrak-id: s\nSubtrak-title: T";
        let result = parse_subtask_trailers(body).unwrap();
        assert_eq!(result.parent_id, "p");
        assert_eq!(result.subtask_id, "s");
        assert_eq!(result.title, "T");
    }

    #[test]
    fn format_subtask_trailers_roundtrips() {
        let formatted = format_subtask_trailers("parent-1", "subtask-1", "My Feature");
        let parsed = parse_subtask_trailers(&formatted).unwrap();
        assert_eq!(parsed.parent_id, "parent-1");
        assert_eq!(parsed.subtask_id, "subtask-1");
        assert_eq!(parsed.title, "My Feature");
    }
}
