// Resolve the enclosing spacetree root and print it as JSON.

use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow, ensure};
use clap::{Parser, Subcommand};

/// Filename of the marker that a spacetree root holds: a symlink to the spacetree repo.
const MARKER_FILENAME: &str = ".spacetree-repo";

/// Command-line surface of the tool.
#[derive(Debug, Parser)]
#[command(
    name = "spacetree",
    about = "Work with the enclosing spacetree.",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

/// Subcommands the tool accepts.
#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Print the enclosing spacetree root
    Root {
        /// Emit JSON; required (only supported output for now)
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let outcome = match cli.command {
        CliCommand::Root { json } => run_root(json),
    };

    if let Err(error) = outcome {
        // The exit-code convention is ours to keep: one line on stderr, status 1.
        eprintln!("!! ERROR: {error:#}");
        std::process::exit(1);
    }
}

/// Print the enclosing spacetree root, the repo its marker points at, and its name.
///
/// Fails when `--json` is absent, matching the tool's current output contract.
fn run_root(emit_json: bool) -> Result<()> {
    ensure!(emit_json, "root requires --json");

    let start = env::current_dir()?;
    let root = get_spacetree_root(&start)?;
    let spacetree_repo = get_spacetree_repo(&root)?;
    let spacetree_name = get_spacetree_name(&root);

    print_json(&render_root_json(&root, &spacetree_repo, &spacetree_name));
    Ok(())
}

/// Render the `root --json` payload for already resolved values.
fn render_root_json(root: &Path, spacetree_repo: &Path, name: &str) -> String {
    format!(
        "{{\"name\":{},\"path\":{},\"spacetree_repo\":{}}}",
        json_string(name),
        json_string(&root.display().to_string()),
        json_string(&spacetree_repo.display().to_string())
    )
}

/// Return the nearest ancestor of `start`, `start` included, holding the repo marker.
///
/// Fails with the marker filename when no ancestor carries it.
fn get_spacetree_root(start: &Path) -> Result<PathBuf> {
    // A dangling symlink still marks a spacetree, so the walk looks at the entry itself
    // rather than at what it points to; resolution reports the broken target afterwards.
    let marker_location = start
        .ancestors()
        .find(|candidate| candidate.join(MARKER_FILENAME).symlink_metadata().is_ok());

    marker_location.map(Path::to_path_buf).ok_or_else(|| {
        anyhow!(
            "no {MARKER_FILENAME} found above {}; the directory is not in a spacetree",
            start.display()
        )
    })
}

/// Resolve `<root>/.spacetree-repo` to the absolute path of the spacetree repo it points at.
///
/// The target is read relative to `root`, so a marker holding a relative link works from any
/// working directory.
/// Fails when the marker is not a symlink, or when its target does not exist or is not a
/// directory.
fn get_spacetree_repo(root: &Path) -> Result<PathBuf> {
    let marker_path = root.join(MARKER_FILENAME);
    let marker_metadata = marker_path.symlink_metadata()?;

    ensure!(
        marker_metadata.file_type().is_symlink(),
        "{} is not a symlink; it must point at the spacetree repo",
        marker_path.display()
    );

    let link_target = std::fs::read_link(&marker_path)?;
    // A marker travels with the tree, so an absolute target would pin it to one machine.
    // Resolve it anyway, and say so rather than rejecting a tree that already works.
    if link_target.is_absolute() {
        eprintln!(
            "!! WARNING: {} points at {}, an absolute path; the marker is meant to hold \
             a path relative to the spacetree root",
            marker_path.display(),
            link_target.display()
        );
    }
    // The link is stored relative to the marker's directory, so joining before resolving
    // keeps the answer independent of the working directory the tool was started from.
    let target = root.join(&link_target);
    let resolved = target.canonicalize().map_err(|error| {
        anyhow!(
            "{} points at {}, which does not resolve: {error}",
            marker_path.display(),
            link_target.display()
        )
    })?;

    ensure!(
        resolved.is_dir(),
        "{} points at {}, which is not a directory",
        marker_path.display(),
        resolved.display()
    );

    Ok(resolved)
}

/// Name a spacetree reports for itself: the basename of its root directory.
fn get_spacetree_name(root: &Path) -> String {
    let basename = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    basename.unwrap_or_else(|| root.display().to_string())
}

/// Print `payload` on a single line, pretty-printing through `jq` when stdout is a terminal.
fn print_json(payload: &str) {
    let stdout_is_terminal = io::stdout().is_terminal();
    if stdout_is_terminal && print_pretty_with_jq(payload) {
        return;
    }
    println!("{payload}");
}

/// Feed `payload` to `jq .` and report whether it produced output successfully.
fn print_pretty_with_jq(payload: &str) -> bool {
    let Ok(mut jq) = Command::new("jq").arg(".").stdin(Stdio::piped()).spawn() else {
        return false;
    };

    if let Some(mut stdin) = jq.stdin.take() {
        // jq writes its own errors when the payload does not parse, so a write failure is
        // not worth reporting: the exit status below already decides the outcome.
        let _ = stdin.write_all(payload.as_bytes());
    }

    matches!(jq.wait(), Ok(status) if status.success())
}

/// Render `value` as a JSON string literal.
fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => escaped.push(other),
        }
    }

    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;

    use tempfile::TempDir;

    use super::*;

    /// Build a temporary spacetree: `<tmp>/tree` holds a `.spacetree-repo` symlink to
    /// `<tmp>/spacetree-home`, a sibling directory playing the repo.
    ///
    /// The marker holds a relative link on purpose, the form a real tree uses.
    fn make_spacetree() -> TempDir {
        let temp = TempDir::new().expect("create temp dir");
        let tree = temp.path().join("tree");
        let spacetree_repo = temp.path().join("spacetree-home");
        fs::create_dir(&tree).expect("create tree dir");
        fs::create_dir(&spacetree_repo).expect("create repo dir");
        symlink("../spacetree-home", tree.join(MARKER_FILENAME)).expect("create marker");
        temp
    }

    /// Canonical form of `temp/<name>`: `TempDir` hands out the non-canonical macOS path
    /// (`/var/…`), while the tool resolves symlinks (`/private/var/…`).
    fn canonical_member(temp: &TempDir, name: &str) -> PathBuf {
        temp.path()
            .canonicalize()
            .expect("canonicalize temp dir")
            .join(name)
    }

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
    fn root_json_carries_every_resolved_value() {
        let payload = render_root_json(
            Path::new("/trees/a tree"),
            Path::new("/trees/spacetree-home"),
            "work",
        );

        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload is valid JSON");

        // Compact form pins the whole contract: field set, values, and absence of extras.
        assert_eq!(
            parsed.to_string(),
            r#"{"name":"work","path":"/trees/a tree","spacetree_repo":"/trees/spacetree-home"}"#
        );
    }

    #[test]
    fn json_string_quotes_a_plain_value() {
        assert_eq!(json_string("test-tree-root"), "\"test-tree-root\"");
    }

    #[test]
    fn json_string_escapes_quotes_and_backslashes() {
        assert_eq!(json_string("a\"b\\c"), r#""a\"b\\c""#);
    }

    #[test]
    fn json_string_escapes_control_characters() {
        assert_eq!(json_string("a\tb"), "\"a\\tb\"");
        assert_eq!(json_string("a\u{1}b"), "\"a\\u0001b\"");
    }
}
