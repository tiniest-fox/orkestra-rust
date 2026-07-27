//! Git log (most recent commits on current branch).

use std::path::Path;
use std::process::Command;

use crate::types::{CommitInfo, GitError};

/// Format string for `git log` output: record-separated fields per commit.
///
/// Fields (null-delimited): hash, subject, author name, author ISO date, body.
pub(super) const LOG_FORMAT: &str = "--format=%x1e%h%x00%s%x00%an%x00%aI%x00%b";

/// Get the N most recent commits on the current branch.
pub fn execute(repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
    // Use record separator (0x1e) between commits to handle multi-line bodies
    let output = Command::new("git")
        .args(["log", &format!("-{limit}"), LOG_FORMAT])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::IoError(format!("Failed to run git log: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::IoError(format!("git log failed: {stderr}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_log_output(&stdout))
}

/// Parse the output of `git log` formatted with [`LOG_FORMAT`] into [`CommitInfo`] records.
pub(super) fn parse_log_output(stdout: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();

    for record in stdout.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let parts: Vec<&str> = record.splitn(5, '\0').collect();
        if parts.len() == 5 {
            let body_text = parts[4].trim();
            let body = if body_text.is_empty() {
                None
            } else {
                Some(body_text.to_string())
            };
            let from_subtask = body
                .as_ref()
                .and_then(|b| orkestra_types::parse_subtask_trailers(b));
            commits.push(CommitInfo {
                hash: parts[0].to_string(),
                message: parts[1].to_string(),
                body,
                author: parts[2].to_string(),
                timestamp: parts[3].to_string(),
                file_count: None,
                from_subtask,
            });
        }
    }

    commits
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn record(hash: &str, subject: &str, author: &str, date: &str, body: &str) -> String {
        format!("\x1e{hash}\x00{subject}\x00{author}\x00{date}\x00{body}")
    }

    #[test]
    fn test_commit_with_subtask_trailers_populates_from_subtask() {
        let body = "Subtrak-of: parent-1\nSubtrak-id: child-1\nSubtrak-title: Add auth layer";
        let stdout = record(
            "abc1234",
            "Add auth layer",
            "Alice",
            "2024-01-01T10:00:00Z",
            body,
        );
        let commits = parse_log_output(&stdout);
        assert_eq!(commits.len(), 1);
        let provenance = commits[0]
            .from_subtask
            .as_ref()
            .expect("from_subtask should be Some");
        assert_eq!(provenance.parent_id, "parent-1");
        assert_eq!(provenance.subtask_id, "child-1");
        assert_eq!(provenance.title, "Add auth layer");
        assert_eq!(commits[0].body.as_deref(), Some(body));
    }

    #[test]
    fn test_commit_without_trailers_has_none_from_subtask() {
        let body = "This is a regular commit body with no trailers.";
        let stdout = record(
            "def5678",
            "Fix the bug",
            "Bob",
            "2024-01-02T12:00:00Z",
            body,
        );
        let commits = parse_log_output(&stdout);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].from_subtask.is_none());
        assert_eq!(commits[0].body.as_deref(), Some(body));
    }

    #[test]
    fn test_commit_with_empty_body_has_none_body_and_from_subtask() {
        let stdout = record(
            "fed9876",
            "Initial commit",
            "Carol",
            "2024-01-03T08:00:00Z",
            "",
        );
        let commits = parse_log_output(&stdout);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].body.is_none());
        assert!(commits[0].from_subtask.is_none());
    }
}
