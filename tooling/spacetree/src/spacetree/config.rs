//! The config a spacetree repo holds, and the label grammar it shares with the tree's names.

use std::path::Path;

use anyhow::{Result, anyhow, ensure};
use serde::Deserialize;

use super::display::display_path;

/// Filename of the config that a spacetree repo holds, declaring the spacetree's identity.
pub(crate) const CONFIG_FILENAME: &str = "spacetree.toml";

/// Identity a spacetree repo declares for its tree, as the `[spacetree]` table holds it.
///
/// Every field is parsed so that a malformed value fails at load, even where no consumer
/// reads it yet; `name` is the one field that resolution itself uses.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpacetreeConfig {
    /// Name the tree reports, and the one the `wkg` spacetree axis keys on.
    pub(crate) name: String,
    /// Free prose for humans describing what the tree is for; never changes resolution.
    #[allow(dead_code)]
    description: Option<String>,
    /// Kind of spacetree, descriptive metadata only; unset unless the config declares it.
    #[allow(dead_code)]
    variant: Option<String>,
}

/// Whole config document: one `[spacetree]` table and nothing else.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpacetreeConfigDocument {
    spacetree: SpacetreeConfig,
}

/// Load and validate the config the spacetree repo at `spacetree_repo` holds.
///
/// Fails when the config file is missing or unreadable, when its TOML does not match the
/// declared shape, when `name` is absent, and when `name` is not a short label.
/// The name never falls back to a directory name.
pub(crate) fn load_spacetree_config(spacetree_repo: &Path) -> Result<SpacetreeConfig> {
    let config_path = spacetree_repo.join(CONFIG_FILENAME);
    let config_text = std::fs::read_to_string(&config_path)
        .map_err(|error| anyhow!("{} cannot be read: {error}", display_path(&config_path)))?;

    let config_doc: SpacetreeConfigDocument = toml::from_str(&config_text)
        .map_err(|error| anyhow!("{} does not parse: {error}", display_path(&config_path)))?;

    let config = config_doc.spacetree;
    check_short_label(&config.name, &config_path, "name")?;

    Ok(config)
}

/// Check that `label`, declared as `field` in `source`, matches the short-label grammar.
///
/// Fails naming both the offending value and where it came from.
fn check_short_label(label: &str, source: &Path, field: &str) -> Result<()> {
    ensure!(
        is_short_label(label),
        "{} has {field} = {label:?}, which is not a short label: expected [a-z0-9][a-z0-9-]*",
        display_path(source)
    );
    Ok(())
}

/// Report whether `label` matches the short-label grammar.
///
/// A short label starts with a lowercase letter or digit, and holds lowercase letters,
/// digits, and hyphens after that.
fn is_short_label(label: &str) -> bool {
    let mut characters = label.chars();
    let Some(first_character) = characters.next() else {
        return false;
    };

    let first_is_allowed = first_character.is_ascii_lowercase() || first_character.is_ascii_digit();
    let rest_are_allowed = characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    });

    first_is_allowed && rest_are_allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetree::test_support::{make_spacetree, spacetree_repo, write_spacetree_config};

    /// Config text declaring every field, for the tests that assert the whole struct loads.
    const CONFIG_WITH_EVERY_FIELD: &str = "\
[spacetree]
name = \"work\"
description = \"Work trees for the greenhouse\"
variant = \"devtree\"
";

    #[test]
    fn config_load_carries_every_declared_field() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, CONFIG_WITH_EVERY_FIELD);

        let config = load_spacetree_config(&spacetree_repo(&temp)).expect("load config");

        assert_eq!(config.name, "work");
        assert_eq!(
            config.description.as_deref(),
            Some("Work trees for the greenhouse")
        );
        assert_eq!(config.variant.as_deref(), Some("devtree"));
    }

    #[test]
    fn config_load_leaves_absent_descriptive_fields_unset() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"work\"\n");

        let config = load_spacetree_config(&spacetree_repo(&temp)).expect("load config");

        assert_eq!(config.name, "work");
        assert!(config.description.is_none());
        assert!(config.variant.is_none());
    }

    #[test]
    fn config_load_names_the_missing_file() {
        let temp = make_spacetree();

        let error =
            load_spacetree_config(&spacetree_repo(&temp)).expect_err("missing config must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains("spacetree.toml cannot be read"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn config_load_names_the_missing_name_field() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\ndescription = \"no name here\"\n");

        let error = load_spacetree_config(&spacetree_repo(&temp)).expect_err("name is required");

        let message = format!("{error:#}");
        assert!(
            message.contains("spacetree.toml") && message.contains("missing field `name`"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn config_load_rejects_an_unknown_field() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"work\"\ncolour = \"red\"\n");

        let error = load_spacetree_config(&spacetree_repo(&temp)).expect_err("typo must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains("unknown field `colour`"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn config_load_rejects_a_name_that_is_not_a_short_label() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"Work Tree\"\n");

        let error = load_spacetree_config(&spacetree_repo(&temp)).expect_err("bad name must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains("not a short label"),
            "unexpected message: {message}"
        );
    }

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
}
