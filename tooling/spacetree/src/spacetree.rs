//! Resolution of the enclosing spacetree: the bindings every command reports.
//!
//! A consumer resolves once through [`SpacetreeContext`] and stops; no call site reconstructs
//! a path from another one.

mod config;
mod display;
mod label;
mod machine_state;
mod marker;
#[cfg(test)]
pub(crate) mod test_support;

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde::Serialize;

use config::load_spacetree_config;
use machine_state::load_machine_state_name;
use marker::{get_shared_root, get_spacetree_repo, get_spacetree_root};

/// Everything the tree resolves to, gathered so a command asks once.
///
/// Construction enforces what the spec treats as invariants: the markers exist, are symlinks,
/// sit together, the shared marker reaches exactly the repo's `tree-shared/` directory, and the
/// machine-state label is recorded and well formed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SpacetreeContext {
    /// Directory holding the markers, which is what "the spacetree root" means.
    root: PathBuf,
    /// Directory the repo marker points at.
    repo: PathBuf,
    /// Directory the shared marker points at, already agreed to be `<repo>/tree-shared`.
    ///
    /// Unset when the tree has no shared marker, or when the marker present cannot be
    /// honoured.
    shared_root: Option<PathBuf>,
    /// Name the repo's config declares; never a directory basename.
    name: String,
    /// Label naming this machine's state branch, read from the tree root; never the OS login.
    machine_state_name: String,
}

impl SpacetreeContext {
    /// Resolve the spacetree enclosing `start`, failing explicitly rather than falling back.
    pub(crate) fn resolve(start: &Path) -> Result<Self> {
        let root = get_spacetree_root(start)?;
        let repo = get_spacetree_repo(&root)?;
        let shared_root = get_shared_root(start, &root, &repo);
        let config = load_spacetree_config(&repo)?;
        let machine_state_name = load_machine_state_name(&root)?;

        Ok(Self {
            root,
            repo,
            shared_root,
            name: config.name,
            machine_state_name,
        })
    }

    /// Render the `info --json` payload for this context.
    ///
    /// `root` stays absolute because it is the path every other answer hangs off; paths inside
    /// it are spelled `./…`, so the answer reads the same on any machine.
    /// `shared_root` is `null` when the tree shares nothing, so a consumer must handle it
    /// rather than assume a path.
    pub(crate) fn render_json(&self) -> Result<String> {
        let payload = InfoPayload {
            spacetree: ResolvedBindings {
                name: &self.name,
                repo: render_path(&self.repo, &self.root),
                root: self.root.display().to_string(),
                shared_root: self
                    .shared_root
                    .as_deref()
                    .map(|path| render_path(path, &self.root)),
                machine_state_name: &self.machine_state_name,
            },
        };

        serde_json::to_string(&payload)
            .map_err(|error| anyhow!("cannot render the info payload as JSON: {error}"))
    }
}

/// Spell `path` relative to `root` when it lives inside it, and absolute otherwise.
///
/// A path inside the root becomes `./…`, which survives a move or a second machine. A checkout
/// outside the root has no such spelling, so it keeps its absolute path rather than an `..`
/// chain whose meaning depends on the depth it is read from.
fn render_path(path: &Path, root: &Path) -> String {
    match path.strip_prefix(root) {
        // The root itself, which no marker target should be, reads as the directory it is.
        Ok(inside) if inside.as_os_str().is_empty() => ".".to_string(),
        Ok(inside) => format!("./{}", inside.display()),
        Err(_) => path.display().to_string(),
    }
}

/// Payload `info --json` emits: every resolved binding, under one key.
#[derive(Debug, Serialize)]
struct InfoPayload<'a> {
    spacetree: ResolvedBindings<'a>,
}

/// The resolved bindings, declared in the order the spec lists them.
#[derive(Debug, Serialize)]
struct ResolvedBindings<'a> {
    name: &'a str,
    repo: String,
    root: String,
    shared_root: Option<String>,
    machine_state_name: &'a str,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use test_support::{
        FIXTURE_MACHINE_STATE_NAME, canonical_member, make_spacetree, write_spacetree_config,
    };

    #[test]
    fn resolve_carries_every_binding() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"work\"\n");

        let context =
            SpacetreeContext::resolve(&temp.path().join("tree")).expect("resolve context");

        // Comparing whole contexts pins every binding at once, and fails if a field is added
        // without being resolved.
        assert_eq!(
            context,
            SpacetreeContext {
                root: temp.path().join("tree"),
                repo: canonical_member(&temp, "spacetree-home"),
                shared_root: Some(canonical_member(&temp, "spacetree-home/tree-shared")),
                name: "work".to_string(),
                machine_state_name: FIXTURE_MACHINE_STATE_NAME.to_string(),
            }
        );
    }

    #[test]
    fn resolve_tolerates_a_disagreeing_shared_marker() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"work\"\n");
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(marker::SHARED_MARKER_FILENAME)).expect("remove shared marker");
        std::os::unix::fs::symlink(
            "../spacetree-home",
            tree.join(marker::SHARED_MARKER_FILENAME),
        )
        .expect("create disagreeing shared marker");

        let context =
            SpacetreeContext::resolve(&tree).expect("a broken shared marker must not fail");

        assert_eq!(
            context.shared_root, None,
            "a disagreeing marker leaves the tree unbound rather than failing"
        );
    }

    #[test]
    fn resolve_leaves_a_tree_without_a_shared_marker_unbound() {
        let temp = make_spacetree();
        write_spacetree_config(&temp, "[spacetree]\nname = \"work\"\n");
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(marker::SHARED_MARKER_FILENAME)).expect("remove shared marker");

        let context = SpacetreeContext::resolve(&tree).expect("the shared marker is optional");

        assert_eq!(context.shared_root, None);
    }

    #[test]
    fn render_json_spells_a_repo_inside_the_root_relative_to_it() {
        let context = SpacetreeContext {
            root: PathBuf::from("/trees/work-stuff"),
            repo: PathBuf::from("/trees/work-stuff/spacetree-home"),
            shared_root: Some(PathBuf::from(
                "/trees/work-stuff/spacetree-home/tree-shared",
            )),
            name: "work".to_string(),
            machine_state_name: "foo".to_string(),
        };

        // The raw string pins the wire contract: field order, values, and absence of extras.
        assert_eq!(
            context.render_json().expect("render the info payload"),
            r#"{"spacetree":{"name":"work","repo":"./spacetree-home","root":"/trees/work-stuff","shared_root":"./spacetree-home/tree-shared","machine_state_name":"foo"}}"#
        );
    }

    #[test]
    fn render_json_keeps_a_repo_outside_the_root_absolute() {
        let context = SpacetreeContext {
            root: PathBuf::from("/trees/a tree"),
            repo: PathBuf::from("/trees/spacetree-home"),
            shared_root: Some(PathBuf::from("/trees/spacetree-home/tree-shared")),
            name: "work".to_string(),
            machine_state_name: "foo".to_string(),
        };

        assert_eq!(
            context.render_json().expect("render the info payload"),
            r#"{"spacetree":{"name":"work","repo":"/trees/spacetree-home","root":"/trees/a tree","shared_root":"/trees/spacetree-home/tree-shared","machine_state_name":"foo"}}"#
        );
    }

    #[test]
    fn render_json_reports_a_null_shared_root() {
        let context = SpacetreeContext {
            root: PathBuf::from("/trees/a tree"),
            repo: PathBuf::from("/trees/spacetree-home"),
            shared_root: None,
            name: "work".to_string(),
            machine_state_name: "foo".to_string(),
        };

        assert_eq!(
            context.render_json().expect("render the info payload"),
            r#"{"spacetree":{"name":"work","repo":"/trees/spacetree-home","root":"/trees/a tree","shared_root":null,"machine_state_name":"foo"}}"#
        );
    }
}
