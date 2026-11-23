//! The two markers a spacetree root holds, and the bindings they must agree on.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, ensure};

use super::display::display_path;

/// Filename of the marker that a spacetree root holds: a symlink to the spacetree repo.
pub(crate) const MARKER_FILENAME: &str = ".spacetree-repo";

/// Filename of the marker that a spacetree root holds: a symlink to the repo's shared namespace.
pub(crate) const SHARED_MARKER_FILENAME: &str = ".spacetree-shared-root";

/// Directory inside the spacetree repo holding the inherited content the shared marker reaches.
pub(crate) const TREE_SHARED_DIRNAME: &str = "tree-shared";

/// Return the nearest ancestor of `start`, `start` included, holding the repo marker.
///
/// Fails with the marker filename when no ancestor carries it.
pub(crate) fn get_spacetree_root(start: &Path) -> Result<PathBuf> {
    find_marker_directory(start, MARKER_FILENAME).ok_or_else(|| {
        anyhow!(
            "no {MARKER_FILENAME} found above {}; the directory is not in a spacetree",
            display_path(start)
        )
    })
}

/// Return the nearest ancestor of `start`, `start` included, holding `filename`.
///
/// A dangling symlink still marks its directory, so the walk looks at the entry itself rather
/// than at what it points to; resolution reports the broken target afterwards.
fn find_marker_directory(start: &Path, filename: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join(filename).symlink_metadata().is_ok())
        .map(Path::to_path_buf)
}

/// Resolve `<root>/.spacetree-repo` to the absolute path of the spacetree repo it points at.
///
/// The target is read relative to `root`, so a marker holding a relative link works from any
/// working directory.
pub(crate) fn get_spacetree_repo(root: &Path) -> Result<PathBuf> {
    resolve_marker_directory(&root.join(MARKER_FILENAME), "the spacetree repo")
}

/// Resolve the shared marker to the absolute path of the shared namespace it points at, when
/// the tree has one that agrees with the repo marker.
///
/// The shared marker is optional: a tree that shares nothing simply has no `shared_root`, and
/// consumers must not fall back to the tree root. Absence is silent.
/// A marker that is present but broken is just as unbound, and warns with its cause rather than
/// failing the command: it is not a symlink, it dangles, it sits in another directory than the
/// repo marker, or it points anywhere but `<repo>/tree-shared`.
/// The agreement between the markers is judged here, once, rather than on every read.
pub(crate) fn get_shared_root(start: &Path, root: &Path, spacetree_repo: &Path) -> Option<PathBuf> {
    let marker_directory = find_marker_directory(start, SHARED_MARKER_FILENAME)?;

    if marker_directory != root {
        warn_broken_shared_marker(&format!(
            "{} sits in {}, not in {}, where {MARKER_FILENAME} is; the markers must share a \
             directory",
            display_path(&root.join(SHARED_MARKER_FILENAME)),
            display_path(&marker_directory),
            display_path(root)
        ));
        return None;
    }

    let shared_root = match resolve_marker_directory(
        &root.join(SHARED_MARKER_FILENAME),
        "the spacetree repo's tree-shared directory",
    ) {
        Ok(shared_root) => shared_root,
        Err(error) => {
            warn_broken_shared_marker(&format!("{error:#}"));
            return None;
        }
    };

    // Both sides are compared resolved, so a relative target and a repo reached through a
    // symlinked path still agree. The expected directory may be absent, in which case the
    // target cannot match it anyway and the comparison reports the mismatch.
    let expected = spacetree_repo.join(TREE_SHARED_DIRNAME);
    let expected_resolved = expected.canonicalize().unwrap_or(expected);
    if shared_root != expected_resolved {
        warn_broken_shared_marker(&format!(
            "{} points at {}, not at {}, the repo's {TREE_SHARED_DIRNAME} directory; the markers \
             disagree",
            display_path(&root.join(SHARED_MARKER_FILENAME)),
            display_path(&shared_root),
            display_path(&expected_resolved)
        ));
        return None;
    }

    Some(shared_root)
}

/// Report a shared marker that is present but cannot be honoured, and name why.
///
/// The command still succeeds: the shared root is optional, so the answer carries `null`
/// instead, and this warning is the only trace of the broken binding.
fn warn_broken_shared_marker(cause: &str) {
    eprintln!("!! WARNING: {cause}; reporting no shared root");
}

/// Resolve the marker symlink at `marker_path` to the absolute directory it points at.
///
/// The target is read relative to the marker's directory, so a marker holding a relative link
/// works from any working directory.
/// `binding` names what the marker must point at, for the error that rejects a plain file.
/// Fails when the marker is not a symlink, or when its target does not exist or is not a
/// directory.
fn resolve_marker_directory(marker_path: &Path, binding: &str) -> Result<PathBuf> {
    let marker_metadata = marker_path.symlink_metadata()?;

    ensure!(
        marker_metadata.file_type().is_symlink(),
        "{} is not a symlink; it must point at {binding}",
        display_path(marker_path)
    );

    let link_target = std::fs::read_link(marker_path)?;
    // A marker travels with the tree, so an absolute target would pin it to one machine.
    // Resolve it anyway, and say so rather than rejecting a tree that already works.
    if link_target.is_absolute() {
        eprintln!(
            "!! WARNING: {} points at {}, an absolute path; the marker is meant to hold \
             a path relative to the spacetree root",
            display_path(marker_path),
            display_path(&link_target)
        );
    }
    // The link is stored relative to the marker's directory, so joining before resolving
    // keeps the answer independent of the working directory the tool was started from.
    let target = marker_path
        .parent()
        .map_or_else(|| link_target.clone(), |parent| parent.join(&link_target));
    let resolved = target.canonicalize().map_err(|error| {
        anyhow!(
            "{} points at {}, which does not resolve: {error}",
            display_path(marker_path),
            display_path(&link_target)
        )
    })?;

    ensure!(
        resolved.is_dir(),
        "{} points at {}, which is not a directory",
        display_path(marker_path),
        display_path(&resolved)
    );

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;

    use tempfile::TempDir;

    use super::*;
    use crate::spacetree::test_support::{canonical_member, make_spacetree, spacetree_repo};

    #[test]
    fn walk_up_finds_the_root_from_a_nested_directory() {
        let temp = make_spacetree();
        let nested = temp.path().join("tree/group/repo");
        fs::create_dir_all(&nested).expect("create nested dirs");

        let root = get_spacetree_root(&nested).expect("resolve root");

        assert_eq!(root, temp.path().join("tree"));
    }

    #[test]
    fn walk_up_prefers_the_nearest_root() {
        let temp = make_spacetree();
        let inner = temp.path().join("tree/inner");
        fs::create_dir_all(inner.join("sub")).expect("create inner dirs");
        fs::create_dir(temp.path().join("tree/inner-home")).expect("create inner repo");
        symlink("../inner-home", inner.join(MARKER_FILENAME)).expect("create inner marker");

        let root = get_spacetree_root(&inner.join("sub")).expect("resolve root");

        assert_eq!(root, inner);
    }

    #[test]
    fn walk_up_names_the_marker_when_none_exists() {
        let temp = TempDir::new().expect("create temp dir");

        let error = get_spacetree_root(temp.path()).expect_err("no marker must fail");

        assert!(
            format!("{error:#}").contains("no .spacetree-repo found above"),
            "unexpected message: {error:#}"
        );
    }

    #[test]
    fn walk_up_finds_a_dangling_marker() {
        let temp = TempDir::new().expect("create temp dir");
        let tree = temp.path().join("tree");
        fs::create_dir(&tree).expect("create tree dir");
        symlink("../missing-repo", tree.join(MARKER_FILENAME)).expect("create marker");

        let root = get_spacetree_root(&tree).expect("resolve root");

        assert_eq!(root, tree);
    }

    #[test]
    fn repo_resolution_follows_a_relative_target() {
        let temp = make_spacetree();

        let spacetree_repo = get_spacetree_repo(&temp.path().join("tree")).expect("resolve repo");

        assert_eq!(spacetree_repo, canonical_member(&temp, "spacetree-home"));
    }

    #[test]
    fn repo_resolution_rejects_a_plain_file_marker() {
        let temp = TempDir::new().expect("create temp dir");
        let tree = temp.path().join("tree");
        fs::create_dir(&tree).expect("create tree dir");
        fs::write(tree.join(MARKER_FILENAME), "").expect("create marker file");

        let error = get_spacetree_repo(&tree).expect_err("marker file must fail");

        assert!(
            format!("{error:#}").contains("is not a symlink; it must point at the spacetree repo"),
            "unexpected message: {error:#}"
        );
    }

    #[test]
    fn repo_resolution_names_a_missing_target() {
        let temp = TempDir::new().expect("create temp dir");
        let tree = temp.path().join("tree");
        fs::create_dir(&tree).expect("create tree dir");
        symlink("../missing-repo", tree.join(MARKER_FILENAME)).expect("create marker");

        let error = get_spacetree_repo(&tree).expect_err("missing target must fail");

        assert!(
            format!("{error:#}").contains("points at ../missing-repo, which does not resolve"),
            "unexpected message: {error:#}"
        );
    }

    #[test]
    fn repo_resolution_rejects_a_target_that_is_not_a_directory() {
        let temp = TempDir::new().expect("create temp dir");
        let tree = temp.path().join("tree");
        fs::create_dir(&tree).expect("create tree dir");
        fs::write(temp.path().join("not-a-repo"), "").expect("create plain file");
        symlink("../not-a-repo", tree.join(MARKER_FILENAME)).expect("create marker");

        let error = get_spacetree_repo(&tree).expect_err("file target must fail");

        assert!(
            format!("{error:#}").contains("which is not a directory"),
            "unexpected message: {error:#}"
        );
    }

    #[test]
    fn shared_resolution_follows_a_relative_target() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");

        let shared_root = get_shared_root(&tree, &tree, &spacetree_repo(&temp));

        assert_eq!(
            shared_root,
            Some(canonical_member(&temp, "spacetree-home/tree-shared"))
        );
    }

    #[test]
    fn shared_resolution_leaves_a_missing_marker_unbound() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(SHARED_MARKER_FILENAME)).expect("remove shared marker");

        let shared_root = get_shared_root(&tree, &tree, &spacetree_repo(&temp));

        assert_eq!(
            shared_root, None,
            "a tree without the marker shares nothing"
        );
    }

    #[test]
    fn shared_resolution_leaves_a_marker_in_another_directory_unbound() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");
        let inner = tree.join("inner");
        fs::create_dir(&inner).expect("create inner dir");
        fs::remove_file(tree.join(SHARED_MARKER_FILENAME)).expect("remove shared marker");
        symlink(
            "../../spacetree-home/tree-shared",
            inner.join(SHARED_MARKER_FILENAME),
        )
        .expect("create inner shared marker");

        let shared_root = get_shared_root(&inner, &tree, &spacetree_repo(&temp));

        assert_eq!(shared_root, None, "a misplaced marker binds nothing");
    }

    #[test]
    fn shared_resolution_leaves_a_plain_file_marker_unbound() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(SHARED_MARKER_FILENAME)).expect("remove shared marker");
        fs::write(tree.join(SHARED_MARKER_FILENAME), "").expect("create shared marker file");

        let shared_root = get_shared_root(&tree, &tree, &spacetree_repo(&temp));

        assert_eq!(shared_root, None, "a plain file binds nothing");
    }

    #[test]
    fn shared_resolution_leaves_a_missing_target_unbound() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(SHARED_MARKER_FILENAME)).expect("remove shared marker");
        symlink("../missing-shared", tree.join(SHARED_MARKER_FILENAME))
            .expect("create shared marker");

        let shared_root = get_shared_root(&tree, &tree, &spacetree_repo(&temp));

        assert_eq!(shared_root, None, "a dangling marker binds nothing");
    }

    #[test]
    fn shared_resolution_leaves_a_target_outside_the_namespace_unbound() {
        let temp = make_spacetree();
        let tree = temp.path().join("tree");
        fs::remove_file(tree.join(SHARED_MARKER_FILENAME)).expect("remove shared marker");
        symlink("../spacetree-home", tree.join(SHARED_MARKER_FILENAME))
            .expect("create shared marker at the repo root");

        let shared_root = get_shared_root(&tree, &tree, &spacetree_repo(&temp));

        assert_eq!(
            shared_root, None,
            "a marker that disagrees binds nothing rather than the repo"
        );
    }
}
