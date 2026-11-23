//! The short-label grammar the tree's names share: the spacetree name, related-tree keys, and
//! the machine-state name.
//!
//! One module owns it so every source of a label — a config field, a file, a registry key — is
//! judged by the same rule and reports the same expectation.

use anyhow::{Result, ensure};

/// Shape a short label takes, spelled out in messages so a reader learns the rule from the error.
pub(crate) const SHORT_LABEL_GRAMMAR: &str = "[a-z0-9][a-z0-9-]*";

/// Report whether `candidate` matches the short-label grammar.
///
/// A short label starts with a lowercase letter or digit, and holds lowercase letters, digits,
/// and hyphens after that.
pub(crate) fn is_short_label(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first_character) = characters.next() else {
        return false;
    };

    let first_is_allowed = first_character.is_ascii_lowercase() || first_character.is_ascii_digit();
    let rest_are_allowed = characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    });

    first_is_allowed && rest_are_allowed
}

/// Fail when `value` is not a short label, naming `holder` — the file or field it came from —
/// so the report says where to look.
pub(crate) fn ensure_short_label(value: &str, holder: &str) -> Result<()> {
    ensure!(
        is_short_label(value),
        "{holder} holds {value:?}, which is not a short label: expected {SHORT_LABEL_GRAMMAR}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_label_check_accepts_valid_labels() {
        let valid_labels = ["work", "w", "0", "work-2", "a-b-c", "work-"];

        for label in valid_labels {
            assert!(is_short_label(label), "{label:?} must be accepted");
        }
    }

    #[test]
    fn short_label_check_rejects_invalid_labels() {
        let invalid_labels = ["", "-work", "Work", "work_tree", "work tree", "wörk", "a.b"];

        for label in invalid_labels {
            assert!(!is_short_label(label), "{label:?} must be rejected");
        }
    }

    #[test]
    fn label_check_names_the_holder_and_the_grammar() {
        let error = ensure_short_label("Foo Bar", "some-file").expect_err("must be rejected");

        let message = format!("{error:#}");
        assert!(
            message.contains("some-file") && message.contains(SHORT_LABEL_GRAMMAR),
            "unexpected message: {message}"
        );
    }
}
