//! Tests for SKILL.md compliance with Anthropic's Agent Skills specification.
//! See: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices

use std::fs;
use std::path::Path;

const SKILL_PATH: &str = "skills/interminai/SKILL.md";

// Limits from Anthropic's Agent Skills specification
const NAME_MAX_CHARS: usize = 64;
const DESCRIPTION_MAX_CHARS: usize = 1024;
const BODY_MAX_LINES: usize = 500;
// Token limit is 5000; roughly 1 token per word, ~5 chars per word
const BODY_MAX_WORDS: usize = 5000;
const BODY_MAX_CHARS: usize = 25000;

#[test]
fn test_skill_name_within_limit() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let name = extract_yaml_field(&content, "name")
        .expect("Failed to find 'name' field in SKILL.md");

    assert!(
        name.len() <= NAME_MAX_CHARS,
        "Skill name exceeds {} char limit: {} chars ('{}')",
        NAME_MAX_CHARS, name.len(), name
    );
}

#[test]
fn test_skill_description_within_limit() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let description = extract_yaml_field(&content, "description")
        .expect("Failed to find 'description' field in SKILL.md");

    assert!(
        description.len() <= DESCRIPTION_MAX_CHARS,
        "Skill description exceeds {} char limit: {} chars",
        DESCRIPTION_MAX_CHARS, description.len()
    );
}

#[test]
fn test_skill_body_within_limit() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    // Find the end of YAML frontmatter (second ---)
    let body = extract_body(&content)
        .expect("Failed to extract body from SKILL.md");

    let line_count = body.lines().count();

    assert!(
        line_count <= BODY_MAX_LINES,
        "Skill body exceeds {} line limit: {} lines",
        BODY_MAX_LINES, line_count
    );
}

#[test]
fn test_skill_body_words_within_limit() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let body = extract_body(&content)
        .expect("Failed to extract body from SKILL.md");

    let word_count = body.split_whitespace().count();

    assert!(
        word_count <= BODY_MAX_WORDS,
        "Skill body exceeds {} word limit: {} words",
        BODY_MAX_WORDS, word_count
    );
}

#[test]
fn test_skill_body_chars_within_limit() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let body = extract_body(&content)
        .expect("Failed to extract body from SKILL.md");

    let char_count = body.len();

    assert!(
        char_count <= BODY_MAX_CHARS,
        "Skill body exceeds {} char limit: {} chars",
        BODY_MAX_CHARS, char_count
    );
}

#[test]
fn test_skill_name_format() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let name = extract_yaml_field(&content, "name")
        .expect("Failed to find 'name' field in SKILL.md");

    // Name must contain only lowercase letters, numbers, and hyphens
    assert!(
        name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
        "Skill name must contain only lowercase letters, numbers, and hyphens: '{}'",
        name
    );

    // Name cannot contain reserved words
    let reserved = ["anthropic", "claude"];
    for word in reserved {
        assert!(
            !name.contains(word),
            "Skill name cannot contain reserved word '{}': '{}'",
            word, name
        );
    }
}

#[test]
fn test_skill_description_not_empty() {
    let content = fs::read_to_string(SKILL_PATH)
        .expect("Failed to read SKILL.md");

    let description = extract_yaml_field(&content, "description")
        .expect("Failed to find 'description' field in SKILL.md");

    assert!(
        !description.trim().is_empty(),
        "Skill description must not be empty"
    );
}

#[test]
fn test_skill_file_exists() {
    assert!(
        Path::new(SKILL_PATH).exists(),
        "SKILL.md not found at {}",
        SKILL_PATH
    );
}

/// Extract a field value from YAML frontmatter
fn extract_yaml_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with(&format!("{}: ", field)) {
            return Some(line[field.len() + 2..].to_string());
        }
    }
    None
}

/// Extract the body content after YAML frontmatter
fn extract_body(content: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut body_start = None;

    for (i, line) in content.lines().enumerate() {
        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                body_start = Some(i + 1);
                break;
            }
        }
    }

    body_start.map(|start| {
        content.lines()
            .skip(start)
            .collect::<Vec<_>>()
            .join("\n")
    })
}
