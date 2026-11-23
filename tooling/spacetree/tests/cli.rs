// End-to-end checks on the `spacetree` command: process-level behaviour only, since the
// resolution logic itself is covered by unit tests.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// The binary under test, as built by cargo for this test run.
const BINARY_PATH: &str = env!("CARGO_BIN_EXE_spacetree");

/// Marker filename, repeated here because the crate exposes no library surface.
const MARKER_FILENAME: &str = ".spacetree-repo";

/// Build a temporary spacetree: `<tmp>/tree` holds a `.spacetree-repo` symlink to
/// `<tmp>/spacetree-home`, a sibling directory playing the repo.
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

/// Run the binary in `directory` with `arguments`, capturing both streams.
fn run_cli(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(BINARY_PATH)
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("run the spacetree binary")
}

/// Capture stderr as text, for assertion messages that read better than raw bytes.
fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Capture stdout as text.
fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn help_lists_the_root_subcommand() {
    let temp = TempDir::new().expect("create temp dir");

    let output = run_cli(temp.path(), &["--help"]);

    assert!(output.status.success());
    assert!(stdout_text(&output).contains("Print the enclosing spacetree root"));
    assert!(!stderr_text(&output).contains("ERROR"));
}

#[test]
fn root_without_json_fails_with_a_usage_error() {
    let temp = make_spacetree();

    let output = run_cli(&temp.path().join("tree"), &["root"]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("root requires --json"),
        "unexpected stderr: {}",
        stderr_text(&output)
    );
    assert!(
        stdout_text(&output).is_empty(),
        "a usage error must print no JSON"
    );
}

#[test]
fn root_prints_one_line_of_json() {
    let temp = make_spacetree();
    let tree = temp.path().join("tree");

    let output = run_cli(&tree, &["root", "--json"]);

    assert!(output.status.success());
    let stdout = stdout_text(&output);
    assert_eq!(stdout.lines().count(), 1, "output must be a single line");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(parsed["name"], "tree");
    assert_eq!(
        parsed["path"],
        canonical_member(&temp, "tree")
            .display()
            .to_string()
            .as_str()
    );
    assert_eq!(
        parsed["spacetree_repo"],
        canonical_member(&temp, "spacetree-home")
            .display()
            .to_string()
            .as_str()
    );
    assert!(
        !stderr_text(&output).contains("WARNING"),
        "a working tree must not warn"
    );
}

#[test]
fn root_fails_outside_a_spacetree() {
    let temp = TempDir::new().expect("create temp dir");

    let output = run_cli(temp.path(), &["root", "--json"]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("no .spacetree-repo found above"),
        "unexpected stderr: {}",
        stderr_text(&output)
    );
    assert!(
        stdout_text(&output).is_empty(),
        "a resolution failure must print no JSON"
    );
}

#[test]
fn root_warns_about_an_absolute_marker_target() {
    let temp = make_spacetree();
    let tree = temp.path().join("tree");
    let spacetree_repo = temp.path().join("spacetree-home");
    fs::remove_file(tree.join(MARKER_FILENAME)).expect("remove relative marker");
    symlink(&spacetree_repo, tree.join(MARKER_FILENAME)).expect("create absolute marker");

    let output = run_cli(&tree, &["root", "--json"]);

    assert!(output.status.success());
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("an absolute path; the marker is meant to hold"),
        "unexpected stderr: {stderr}"
    );
    assert!(
        !stderr.contains("ERROR"),
        "the warning must not be an error"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_text(&output)).expect("valid JSON");
    assert_eq!(
        parsed["spacetree_repo"],
        canonical_member(&temp, "spacetree-home")
            .display()
            .to_string()
            .as_str()
    );
}
