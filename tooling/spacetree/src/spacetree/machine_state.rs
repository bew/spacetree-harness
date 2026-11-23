//! The label naming this machine's state branch, read from the tree root rather than derived.

use std::path::Path;

use anyhow::{Result, anyhow};

use super::display::display_path;
use super::label::ensure_short_label;

/// Directory holding the tree's unversioned state, at the spacetree root.
pub(crate) const SPACETREE_DIRNAME: &str = ".spacetree";

/// Filename of the label naming this machine's state, inside [`SPACETREE_DIRNAME`].
pub(crate) const MACHINE_STATE_NAME_FILENAME: &str = "machine-state-name";

/// Read and validate the machine-state label the tree at `root` records.
///
/// The label reaches resolution only through this file: `init` records the choice once, so a
/// changed OS login cannot silently map to a different state branch.
/// Surrounding whitespace is trimmed, since the trailing newline an editor leaves is not part of
/// the label; an empty file still fails, because the trimmed value is not a short label.
pub(crate) fn load_machine_state_name(root: &Path) -> Result<String> {
    let label_path = root
        .join(SPACETREE_DIRNAME)
        .join(MACHINE_STATE_NAME_FILENAME);
    let label_text = std::fs::read_to_string(&label_path)
        .map_err(|error| anyhow!("{} cannot be read: {error}", display_path(&label_path)))?;
    let label = label_text.trim();

    ensure_short_label(label, &display_path(&label_path))?;

    Ok(label.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::spacetree::test_support::{
        FIXTURE_MACHINE_STATE_NAME, make_spacetree, spacetree_root, write_machine_state_name,
    };

    #[test]
    fn label_load_trims_the_recorded_line() {
        let temp = make_spacetree();
        write_machine_state_name(&temp, "foo\n");

        let label = load_machine_state_name(&spacetree_root(&temp)).expect("load the label");

        assert_eq!(label, "foo");
    }

    #[test]
    fn label_load_names_the_missing_file() {
        let temp = make_spacetree();
        fs::remove_file(
            spacetree_root(&temp)
                .join(SPACETREE_DIRNAME)
                .join(MACHINE_STATE_NAME_FILENAME),
        )
        .expect("remove the label file");

        let error =
            load_machine_state_name(&spacetree_root(&temp)).expect_err("a missing label must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains(MACHINE_STATE_NAME_FILENAME) && message.contains("cannot be read"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn label_load_rejects_a_value_that_is_not_a_short_label() {
        let temp = make_spacetree();
        write_machine_state_name(&temp, "Foo Bar");

        let error =
            load_machine_state_name(&spacetree_root(&temp)).expect_err("a bad label must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains(MACHINE_STATE_NAME_FILENAME)
                && message.contains("not a short label")
                && message.contains("[a-z0-9][a-z0-9-]*"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn label_load_rejects_an_empty_file() {
        let temp = make_spacetree();
        write_machine_state_name(&temp, "\n");

        let error =
            load_machine_state_name(&spacetree_root(&temp)).expect_err("an empty label must fail");

        let message = format!("{error:#}");
        assert!(
            message.contains("not a short label"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn label_load_reads_the_fixture_value() {
        let temp = make_spacetree();

        let label = load_machine_state_name(&spacetree_root(&temp)).expect("load the label");

        assert_eq!(label, FIXTURE_MACHINE_STATE_NAME);
    }
}
