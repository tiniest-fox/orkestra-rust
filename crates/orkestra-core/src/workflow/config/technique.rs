//! Technique, check-script, and model-registry parsing and resolution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use orkestra_types::config::{GateConfig, ToolRestriction};

// ============================================================================
// Public Types
// ============================================================================

/// Resolved configuration from a set of techniques.
#[derive(Debug, Clone)]
pub struct ResolvedTechniques {
    /// Highest-ranked model from the technique set, if any technique specifies one.
    pub model: Option<String>,
    /// Merged, deduplicated tool restrictions from all techniques.
    pub disallowed_tools: Vec<ToolRestriction>,
    /// Sorted, deduplicated check script names from all techniques.
    pub checks: Vec<String>,
    /// Concatenated technique bodies joined by `"\n\n"`.
    pub prompt_body: String,
}

/// A parsed technique ready for use in stage configuration.
#[derive(Debug, Clone)]
pub struct Technique {
    /// Derived from filename: "red-green.md" → "red-green".
    pub name: String,
    pub title: String,
    pub description: String,
    pub check: Option<String>,
    pub disallowed_tools: Vec<ToolRestriction>,
    pub model: Option<String>,
    /// Prompt content after frontmatter.
    pub body: String,
}

/// Metadata parsed from a check script's embedded YAML block.
#[derive(Debug, Clone)]
pub struct CheckMetadata {
    pub title: String,
    pub description: String,
    pub timeout_seconds: u64,
}

/// Registry of model identifiers ranked by preference.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelRegistry {
    pub default: String,
    pub ranked: Vec<String>,
}

// ============================================================================
// Error Type
// ============================================================================

/// Errors that can occur when loading or parsing technique files.
#[derive(Debug, thiserror::Error)]
pub enum TechniqueLoadError {
    #[error("technique file not found: {0}")]
    NotFound(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse technique frontmatter: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("invalid technique: {0}")]
    Validation(String),
}

// ============================================================================
// Private Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct TechniqueFrontmatter {
    title: String,
    description: String,
    check: Option<String>,
    #[serde(default)]
    disallowed_tools: Vec<ToolRestriction>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CheckMetadataRaw {
    title: String,
    description: String,
    timeout_seconds: u64,
}

// ============================================================================
// Public Functions
// ============================================================================

/// Parse a technique file from the given path.
///
/// Returns `TechniqueLoadError::NotFound` if the file does not exist.
/// Derives `name` from the filename stem (e.g., "red-green.md" → "red-green").
pub fn parse_technique(path: &Path) -> Result<Technique, TechniqueLoadError> {
    if !path.exists() {
        return Err(TechniqueLoadError::NotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path)?;

    let (frontmatter_str, body) = super::split_frontmatter(&content).ok_or_else(|| {
        TechniqueLoadError::Validation(
            "missing YAML frontmatter (expected --- delimiters)".to_string(),
        )
    })?;

    let frontmatter: TechniqueFrontmatter = serde_yaml::from_str(frontmatter_str)?;

    let name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    Ok(Technique {
        name,
        title: frontmatter.title,
        description: frontmatter.description,
        check: frontmatter.check,
        disallowed_tools: frontmatter.disallowed_tools,
        model: frontmatter.model,
        body: body.to_string(),
    })
}

/// Parse check metadata from check script content.
///
/// Expects the script to contain a `# ---` delimited YAML metadata block
/// after the shebang line. Strips leading `# ` from each line before parsing.
pub fn parse_check_metadata(content: &str) -> Result<CheckMetadata, TechniqueLoadError> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the first `# ---` delimiter
    let start = lines.iter().position(|l| *l == "# ---").ok_or_else(|| {
        TechniqueLoadError::Validation(
            "check script missing '# ---' metadata delimiters".to_string(),
        )
    })?;

    // Find the closing `# ---` delimiter after the opening one
    let end = lines[start + 1..]
        .iter()
        .position(|l| *l == "# ---")
        .ok_or_else(|| {
            TechniqueLoadError::Validation(
                "check script missing closing '# ---' delimiter".to_string(),
            )
        })?
        + start
        + 1;

    // Strip leading `# ` from each line between the delimiters
    let yaml_lines: Vec<&str> = lines[start + 1..end]
        .iter()
        .map(|l| l.strip_prefix("# ").unwrap_or(l))
        .collect();
    let yaml = yaml_lines.join("\n");

    let raw: CheckMetadataRaw = serde_yaml::from_str(&yaml).map_err(|e| {
        TechniqueLoadError::Validation(format!("failed to parse check metadata YAML: {e}"))
    })?;

    Ok(CheckMetadata {
        title: raw.title,
        description: raw.description,
        timeout_seconds: raw.timeout_seconds,
    })
}

/// Parse a model registry from YAML content.
///
/// Validates that `default` and `ranked` are non-empty.
pub fn parse_model_registry(content: &str) -> Result<ModelRegistry, TechniqueLoadError> {
    let registry: ModelRegistry = serde_yaml::from_str(content)?;

    if registry.default.is_empty() {
        return Err(TechniqueLoadError::Validation(
            "model registry 'default' must not be empty".to_string(),
        ));
    }
    if registry.ranked.is_empty() {
        return Err(TechniqueLoadError::Validation(
            "model registry 'ranked' must not be empty".to_string(),
        ));
    }

    Ok(registry)
}

/// Resolve the highest-ranked model across a set of techniques.
///
/// Returns the model with the lowest index in `registry.ranked` (highest rank).
/// Falls back to `registry.default` if no technique specifies a model or no
/// specified model appears in the ranked list.
pub fn resolve_model(techniques: &[&Technique], registry: &ModelRegistry) -> String {
    techniques
        .iter()
        .filter_map(|t| t.model.as_deref())
        .filter_map(|model| {
            registry
                .ranked
                .iter()
                .position(|r| r == model)
                .map(|idx| (idx, model))
        })
        .min_by_key(|(idx, _)| *idx)
        .map_or_else(|| registry.default.clone(), |(_, model)| model.to_string())
}

/// Collect, sort, and deduplicate check script names across a set of techniques.
pub fn resolve_checks(techniques: &[&Technique]) -> Vec<String> {
    let mut checks: Vec<String> = techniques.iter().filter_map(|t| t.check.clone()).collect();
    checks.sort();
    checks.dedup();
    checks
}

/// Collect and deduplicate tool restrictions across a set of techniques.
///
/// Deduplication is by pattern only — if two techniques restrict the same
/// pattern with different messages, the first one wins. Output is sorted by
/// pattern for deterministic ordering.
pub fn resolve_disallowed_tools(techniques: &[&Technique]) -> Vec<ToolRestriction> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut tools: Vec<ToolRestriction> = techniques
        .iter()
        .flat_map(|t| t.disallowed_tools.iter().cloned())
        .filter(|t| seen.insert(t.pattern.clone()))
        .collect();
    tools.sort_by(|a, b| a.pattern.cmp(&b.pattern));
    tools
}

// -- Stage Resolution --

/// Load and resolve a set of techniques into a merged configuration.
///
/// Loads technique files from `project_root/.orkestra/techniques/{name}.md`,
/// resolves model via `models.yaml` (if present), merges tool restrictions and
/// checks, and concatenates prompt bodies.
pub fn load_and_resolve_techniques(
    project_root: &Path,
    technique_names: &[String],
) -> Result<ResolvedTechniques, TechniqueLoadError> {
    let techniques_dir = project_root.join(".orkestra/techniques");
    let techniques: Vec<Technique> = technique_names
        .iter()
        .map(|name| parse_technique(&techniques_dir.join(format!("{name}.md"))))
        .collect::<Result<_, _>>()?;

    let tech_refs: Vec<&Technique> = techniques.iter().collect();

    // Load model registry if present; fall back to first technique's raw model value.
    let registry_path = project_root.join(".orkestra/models.yaml");
    let model = if registry_path.exists() {
        let content = std::fs::read_to_string(&registry_path)?;
        let registry = parse_model_registry(&content)?;
        Some(resolve_model(&tech_refs, &registry))
    } else {
        tech_refs.iter().find_map(|t| t.model.clone())
    };

    let disallowed_tools = resolve_disallowed_tools(&tech_refs);
    let checks = resolve_checks(&tech_refs);
    let prompt_body = techniques
        .iter()
        .map(|t| t.body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(ResolvedTechniques {
        model,
        disallowed_tools,
        checks,
        prompt_body,
    })
}

/// Build an automated gate from a list of check script names.
///
/// Reads each check from `project_root/.orkestra/checks/{name}.sh`, parses its
/// metadata for the timeout, builds a compound shell command that runs them in
/// sequence, and returns `GateConfig::Automated`. Returns `Ok(None)` when
/// `check_names` is empty.
pub fn build_gate_from_checks(
    project_root: &Path,
    check_names: &[String],
) -> Result<Option<GateConfig>, TechniqueLoadError> {
    if check_names.is_empty() {
        return Ok(None);
    }

    let checks_dir = project_root.join(".orkestra/checks");
    let mut timeout_total: u64 = 0;
    let mut commands: Vec<String> = Vec::new();

    for name in check_names {
        let path = checks_dir.join(format!("{name}.sh"));
        let content = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TechniqueLoadError::NotFound(path.clone())
            } else {
                TechniqueLoadError::Io(e)
            }
        })?;
        let meta = parse_check_metadata(&content)?;
        timeout_total += meta.timeout_seconds;
        commands.push(format!("$ORKESTRA_PROJECT_ROOT/.orkestra/checks/{name}.sh"));
    }

    let command = commands.join(" && ");
    Ok(Some(GateConfig::Automated {
        command,
        timeout_seconds: timeout_total,
    }))
}

// -- Helpers --

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_technique_valid() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("red-green.md");
        std::fs::write(
            &path,
            "---\ntitle: Red Green\ndescription: TDD cycle\ncheck: checks.sh\ndisallowed_tools:\n  - pattern: 'Bash(cargo *)'\n    message: Use the gate\nmodel: claudecode/sonnet\n---\nWrite failing tests first.",
        )
        .unwrap();

        let t = parse_technique(&path).unwrap();
        assert_eq!(t.name, "red-green");
        assert_eq!(t.title, "Red Green");
        assert_eq!(t.description, "TDD cycle");
        assert_eq!(t.check, Some("checks.sh".to_string()));
        assert_eq!(t.disallowed_tools.len(), 1);
        assert_eq!(t.disallowed_tools[0].pattern, "Bash(cargo *)");
        assert_eq!(t.model, Some("claudecode/sonnet".to_string()));
        assert_eq!(t.body, "Write failing tests first.");
    }

    #[test]
    fn test_parse_technique_missing_title() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-title.md");
        std::fs::write(&path, "---\ndescription: No title here\n---\nBody").unwrap();

        let result = parse_technique(&path);
        assert!(matches!(result, Err(TechniqueLoadError::Parse(_))));
    }

    #[test]
    fn test_parse_technique_not_found() {
        let path = Path::new("/nonexistent/path/technique.md");
        let result = parse_technique(path);
        assert!(matches!(result, Err(TechniqueLoadError::NotFound(_))));
    }

    #[test]
    fn test_parse_technique_no_frontmatter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("plain.md");
        std::fs::write(&path, "Just plain text without frontmatter").unwrap();

        let result = parse_technique(&path);
        assert!(matches!(result, Err(TechniqueLoadError::Validation(_))));
    }

    #[test]
    fn test_parse_technique_minimal() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("minimal.md");
        std::fs::write(
            &path,
            "---\ntitle: Minimal\ndescription: Bare minimum\n---\nBody content",
        )
        .unwrap();

        let t = parse_technique(&path).unwrap();
        assert_eq!(t.name, "minimal");
        assert_eq!(t.title, "Minimal");
        assert_eq!(t.description, "Bare minimum");
        assert!(t.check.is_none());
        assert!(t.disallowed_tools.is_empty());
        assert!(t.model.is_none());
        assert_eq!(t.body, "Body content");
    }

    #[test]
    fn test_parse_check_metadata_valid() {
        let content = "#!/bin/bash\n# ---\n# title: Run Tests\n# description: Runs the full test suite\n# timeout_seconds: 300\n# ---\necho 'running'";
        let meta = parse_check_metadata(content).unwrap();
        assert_eq!(meta.title, "Run Tests");
        assert_eq!(meta.description, "Runs the full test suite");
        assert_eq!(meta.timeout_seconds, 300);
    }

    #[test]
    fn test_parse_check_metadata_no_delimiters() {
        let content = "#!/bin/bash\necho 'no metadata here'";
        let result = parse_check_metadata(content);
        assert!(matches!(result, Err(TechniqueLoadError::Validation(_))));
    }

    #[test]
    fn test_parse_model_registry_valid() {
        let yaml = "default: claudecode/sonnet\nranked:\n  - claudecode/opus\n  - claudecode/sonnet\n  - claudecode/haiku\n";
        let registry = parse_model_registry(yaml).unwrap();
        assert_eq!(registry.default, "claudecode/sonnet");
        assert_eq!(registry.ranked.len(), 3);
        assert_eq!(registry.ranked[0], "claudecode/opus");
    }

    #[test]
    fn test_parse_model_registry_empty_ranked() {
        let yaml = "default: claudecode/sonnet\nranked: []\n";
        let result = parse_model_registry(yaml);
        assert!(matches!(result, Err(TechniqueLoadError::Validation(_))));
    }

    fn test_technique(name: &str) -> Technique {
        Technique {
            name: name.to_string(),
            title: name.to_string(),
            description: String::new(),
            check: None,
            disallowed_tools: Vec::new(),
            model: None,
            body: String::new(),
        }
    }

    fn test_registry() -> ModelRegistry {
        ModelRegistry {
            default: "claudecode/haiku".to_string(),
            ranked: vec![
                "claudecode/opus".to_string(),
                "claudecode/sonnet".to_string(),
                "claudecode/haiku".to_string(),
            ],
        }
    }

    #[test]
    fn test_resolve_model_picks_highest_ranked() {
        let mut t1 = test_technique("a");
        t1.model = Some("claudecode/sonnet".to_string());
        let mut t2 = test_technique("b");
        t2.model = Some("claudecode/opus".to_string());
        let registry = test_registry();

        let result = resolve_model(&[&t1, &t2], &registry);
        assert_eq!(result, "claudecode/opus");
    }

    #[test]
    fn test_resolve_model_falls_back_to_default() {
        let t1 = test_technique("a");
        let t2 = test_technique("b");
        let registry = test_registry();

        let result = resolve_model(&[&t1, &t2], &registry);
        assert_eq!(result, "claudecode/haiku");
    }

    #[test]
    fn test_resolve_model_unknown_model_ignored() {
        let mut t1 = test_technique("a");
        t1.model = Some("unknown/model".to_string());
        let registry = test_registry();

        let result = resolve_model(&[&t1], &registry);
        assert_eq!(result, "claudecode/haiku");
    }

    #[test]
    fn test_resolve_checks_deduplicates() {
        let mut t1 = test_technique("a");
        t1.check = Some("lint".to_string());
        let mut t2 = test_technique("b");
        t2.check = Some("lint".to_string());

        let result = resolve_checks(&[&t1, &t2]);
        assert_eq!(result, vec!["lint"]);
    }

    #[test]
    fn test_resolve_checks_sorts() {
        let mut t1 = test_technique("a");
        t1.check = Some("zebra".to_string());
        let mut t2 = test_technique("b");
        t2.check = Some("alpha".to_string());
        let mut t3 = test_technique("c");
        t3.check = Some("middle".to_string());

        let result = resolve_checks(&[&t1, &t2, &t3]);
        assert_eq!(result, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_resolve_checks_empty() {
        let t1 = test_technique("a");
        let t2 = test_technique("b");

        let result = resolve_checks(&[&t1, &t2]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_disallowed_tools_deduplicates() {
        let restriction = ToolRestriction {
            pattern: "Edit".to_string(),
            message: None,
        };
        let mut t1 = test_technique("a");
        t1.disallowed_tools = vec![restriction.clone()];
        let mut t2 = test_technique("b");
        t2.disallowed_tools = vec![restriction];

        let result = resolve_disallowed_tools(&[&t1, &t2]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "Edit");
    }

    #[test]
    fn test_resolve_disallowed_tools_unions() {
        let mut t1 = test_technique("a");
        t1.disallowed_tools = vec![ToolRestriction {
            pattern: "Edit".to_string(),
            message: None,
        }];
        let mut t2 = test_technique("b");
        t2.disallowed_tools = vec![ToolRestriction {
            pattern: "Write".to_string(),
            message: None,
        }];

        let result = resolve_disallowed_tools(&[&t1, &t2]);
        assert_eq!(result.len(), 2);
        // sorted by pattern
        assert_eq!(result[0].pattern, "Edit");
        assert_eq!(result[1].pattern, "Write");
    }

    #[test]
    fn test_resolve_disallowed_tools_same_pattern_different_message() {
        let mut t1 = test_technique("a");
        t1.disallowed_tools = vec![ToolRestriction {
            pattern: "Edit".to_string(),
            message: Some("read-only".to_string()),
        }];
        let mut t2 = test_technique("b");
        t2.disallowed_tools = vec![ToolRestriction {
            pattern: "Edit".to_string(),
            message: None,
        }];

        let result = resolve_disallowed_tools(&[&t1, &t2]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "Edit");
        // First writer wins
        assert_eq!(result[0].message, Some("read-only".to_string()));
    }

    #[test]
    fn test_resolve_disallowed_tools_empty() {
        let t1 = test_technique("a");
        let t2 = test_technique("b");

        let result = resolve_disallowed_tools(&[&t1, &t2]);
        assert!(result.is_empty());
    }

    // -- load_and_resolve_techniques --

    fn write_technique(dir: &std::path::Path, name: &str, content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{name}.md")), content).unwrap();
    }

    fn write_check(dir: &std::path::Path, name: &str, content: &str, executable: bool) {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{name}.sh"));
        std::fs::write(&path, content).unwrap();
        if executable {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
    }

    #[test]
    fn test_load_and_resolve_techniques_basic() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let techniques_dir = root.join(".orkestra/techniques");

        write_technique(
            &techniques_dir,
            "alpha",
            "---\ntitle: Alpha\ndescription: First\nmodel: opus\n---\nAlpha body",
        );
        write_technique(
            &techniques_dir,
            "beta",
            "---\ntitle: Beta\ndescription: Second\ndisallowed_tools:\n  - pattern: Edit\n---\nBeta body",
        );

        // Write models.yaml
        let models_path = root.join(".orkestra/models.yaml");
        std::fs::create_dir_all(models_path.parent().unwrap()).unwrap();
        std::fs::write(
            &models_path,
            "default: haiku\nranked:\n  - opus\n  - sonnet\n  - haiku\n",
        )
        .unwrap();

        let names = vec!["alpha".to_string(), "beta".to_string()];
        let resolved = load_and_resolve_techniques(root, &names).unwrap();

        assert_eq!(resolved.model, Some("opus".to_string()));
        assert_eq!(resolved.disallowed_tools.len(), 1);
        assert_eq!(resolved.disallowed_tools[0].pattern, "Edit");
        assert!(resolved.checks.is_empty());
        assert_eq!(resolved.prompt_body, "Alpha body\n\nBeta body");
    }

    #[test]
    fn test_load_and_resolve_techniques_no_models_yaml() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let techniques_dir = root.join(".orkestra/techniques");

        write_technique(
            &techniques_dir,
            "solo",
            "---\ntitle: Solo\ndescription: Only one\nmodel: sonnet\n---\nSolo body",
        );

        let names = vec!["solo".to_string()];
        let resolved = load_and_resolve_techniques(root, &names).unwrap();

        // Falls back to first technique's raw model value
        assert_eq!(resolved.model, Some("sonnet".to_string()));
        assert_eq!(resolved.prompt_body, "Solo body");
    }

    #[test]
    fn test_load_and_resolve_techniques_missing_file() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let names = vec!["nonexistent".to_string()];
        let result = load_and_resolve_techniques(root, &names);
        assert!(matches!(result, Err(TechniqueLoadError::NotFound(_))));
    }

    #[test]
    fn test_load_and_resolve_techniques_with_checks() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let techniques_dir = root.join(".orkestra/techniques");

        write_technique(
            &techniques_dir,
            "checked",
            "---\ntitle: Checked\ndescription: Has a check\ncheck: my-check\n---\nChecked body",
        );

        let names = vec!["checked".to_string()];
        let resolved = load_and_resolve_techniques(root, &names).unwrap();

        assert_eq!(resolved.checks, vec!["my-check".to_string()]);
    }

    // -- build_gate_from_checks --

    #[test]
    fn test_build_gate_from_checks_empty() {
        let dir = tempdir().unwrap();
        let result = build_gate_from_checks(dir.path(), &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_gate_from_checks_single() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let checks_dir = root.join(".orkestra/checks");

        write_check(
            &checks_dir,
            "lint",
            "#!/bin/bash\n# ---\n# title: Lint\n# description: Run linter\n# timeout_seconds: 60\n# ---\nexit 0",
            true,
        );

        let names = vec!["lint".to_string()];
        let gate = build_gate_from_checks(root, &names).unwrap();

        assert!(gate.is_some());
        if let Some(GateConfig::Automated {
            command,
            timeout_seconds,
        }) = gate
        {
            assert!(command.contains("$ORKESTRA_PROJECT_ROOT/.orkestra/checks/lint.sh"));
            assert_eq!(timeout_seconds, 60);
        } else {
            panic!("Expected Automated gate");
        }
    }

    #[test]
    fn test_build_gate_from_checks_multiple() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let checks_dir = root.join(".orkestra/checks");

        write_check(
            &checks_dir,
            "lint",
            "#!/bin/bash\n# ---\n# title: Lint\n# description: Run linter\n# timeout_seconds: 60\n# ---\nexit 0",
            true,
        );
        write_check(
            &checks_dir,
            "tests",
            "#!/bin/bash\n# ---\n# title: Tests\n# description: Run tests\n# timeout_seconds: 120\n# ---\nexit 0",
            true,
        );

        let names = vec!["lint".to_string(), "tests".to_string()];
        let gate = build_gate_from_checks(root, &names).unwrap();

        if let Some(GateConfig::Automated {
            command,
            timeout_seconds,
        }) = gate
        {
            assert!(command.contains("lint.sh"));
            assert!(command.contains("tests.sh"));
            assert!(command.contains(" && "));
            assert_eq!(timeout_seconds, 180); // 60 + 120
        } else {
            panic!("Expected Automated gate");
        }
    }

    #[test]
    fn test_build_gate_from_checks_missing_file() {
        let dir = tempdir().unwrap();
        let result = build_gate_from_checks(dir.path(), &["missing".to_string()]);
        assert!(matches!(result, Err(TechniqueLoadError::NotFound(_))));
    }
}
