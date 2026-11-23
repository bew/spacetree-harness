//! Fixtures shared by the unit tests of the resolution modules.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use tempfile::TempDir;

use super::config::CONFIG_FILENAME;
use super::machine_state::{MACHINE_STATE_NAME_FILENAME, SPACETREE_DIRNAME};
use super::marker::{MARKER_FILENAME, SHARED_MARKER_FILENAME, TREE_SHARED_DIRNAME};

/// Label a `make_spacetree` fixture records for the machine it stands for.
pub(crate) const FIXTURE_MACHINE_STATE_NAME: &str = "fixture-box";

/// Build a temporary spacetree: `<tmp>/tree` holds a `.spacetree-repo` symlink to
/// `<tmp>/spacetree-home`, a sibling directory playing the repo, and a
/// `.spacetree-shared-root` symlink to that repo's `tree-shared/` directory.
/// It also records a machine-state label, so resolution has every binding to read.
///
/// Both markers hold relative links on purpose, the form a real tree uses.
pub(crate) fn make_spacetree() -> TempDir {
    let temp = TempDir::new().expect("create temp dir");
    let tree = temp.path().join("tree");
    let spacetree_repo = temp.path().join("spacetree-home");
    fs::create_dir(&tree).expect("create tree dir");
    fs::create_dir(&spacetree_repo).expect("create repo dir");
    fs::create_dir(spacetree_repo.join(TREE_SHARED_DIRNAME)).expect("create tree-shared dir");
    symlink("../spacetree-home", tree.join(MARKER_FILENAME)).expect("create marker");
    symlink(
        format!("../spacetree-home/{TREE_SHARED_DIRNAME}"),
        tree.join(SHARED_MARKER_FILENAME),
    )
    .expect("create shared marker");
    fs::create_dir(tree.join(SPACETREE_DIRNAME)).expect("create state dir");
    write_machine_state_name(&temp, FIXTURE_MACHINE_STATE_NAME);
    temp
}

/// Path of the spacetree root inside a `make_spacetree` fixture.
pub(crate) fn spacetree_root(temp: &TempDir) -> PathBuf {
    temp.path().join("tree")
}

/// Write `text` as the machine-state label the tree of a `make_spacetree` fixture records.
pub(crate) fn write_machine_state_name(temp: &TempDir, text: &str) {
    fs::write(
        spacetree_root(temp)
            .join(SPACETREE_DIRNAME)
            .join(MACHINE_STATE_NAME_FILENAME),
        text,
    )
    .expect("write the machine-state label");
}

/// Canonical form of `temp/<name>`: `TempDir` hands out the non-canonical macOS path
/// (`/var/…`), while the tool resolves symlinks (`/private/var/…`).
pub(crate) fn canonical_member(temp: &TempDir, name: &str) -> PathBuf {
    temp.path()
        .canonicalize()
        .expect("canonicalize temp dir")
        .join(name)
}

/// Path of the repo directory inside a `make_spacetree` fixture.
pub(crate) fn spacetree_repo(temp: &TempDir) -> PathBuf {
    temp.path().join("spacetree-home")
}

/// Write `text` as the config the repo of a `make_spacetree` fixture holds.
pub(crate) fn write_spacetree_config(temp: &TempDir, text: &str) {
    fs::write(spacetree_repo(temp).join(CONFIG_FILENAME), text).expect("write config");
}
