// Resolve the enclosing spacetree and print what it resolves to.

use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow, ensure};
use clap::{Parser, Subcommand};
use serde::Deserialize;

/// Filename of the marker that a spacetree root holds: a symlink to the spacetree repo.
const MARKER_FILENAME: &str = ".spacetree-repo";

/// Filename of the config that a spacetree repo holds, declaring the spacetree's identity.
const CONFIG_FILENAME: &str = "spacetree.toml";

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
    /// Print what the enclosing spacetree resolves to
    Info {
        /// Emit JSON; required (only supported output for now)
        #[arg(long)]
        json: bool,
    },
}

/// Identity a spacetree repo declares for its tree, as the `[spacetree]` table holds it.
///
/// Every field is parsed so that a malformed value fails at load, even where no consumer
/// reads it yet; `name` is the one field that resolution itself uses.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpacetreeConfig {
    /// Name the tree reports, and the one the `wkg` spacetree axis keys on.
    name: String,
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

fn main() {
    let cli = Cli::parse();

    let outcome = match cli.command {
        CliCommand::Info { json } => run_info(json),
    };

    if let Err(error) = outcome {
        // The exit-code convention is ours to keep: one line on stderr, status 1.
        eprintln!("!! ERROR: {error:#}");
        std::process::exit(1);
    }
}

/// Print what the enclosing spacetree resolves to: its root, the repo its marker points at,
/// and its declared name.
///
/// Fails when `--json` is absent, matching the tool's current output contract.
fn run_info(emit_json: bool) -> Result<()> {
    ensure!(emit_json, "info requires --json");

    let start = env::current_dir()?;
    let root = get_spacetree_root(&start)?;
    let spacetree_repo = get_spacetree_repo(&root)?;
    let config = load_spacetree_config(&spacetree_repo)?;

    print_json(&render_info_json(&root, &spacetree_repo, &config.name));
    Ok(())
}

/// Render the `info --json` payload for already resolved values.
///
/// Paths stay absolute: consumers read them as paths, not as prose for humans.
fn render_info_json(root: &Path, spacetree_repo: &Path, name: &str) -> String {
    format!(
        "{{\"spacetree\":{{\"name\":{},\"repo\":{},\"root\":{}}}}}",
        json_string(name),
        json_string(&spacetree_repo.display().to_string()),
        json_string(&root.display().to_string())
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
            display_path(start)
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
        display_path(&marker_path)
    );

    let link_target = std::fs::read_link(&marker_path)?;
    // A marker travels with the tree, so an absolute target would pin it to one machine.
    // Resolve it anyway, and say so rather than rejecting a tree that already works.
    if link_target.is_absolute() {
        eprintln!(
            "!! WARNING: {} points at {}, an absolute path; the marker is meant to hold \
             a path relative to the spacetree root",
            display_path(&marker_path),
            display_path(&link_target)
        );
    }
    // The link is stored relative to the marker's directory, so joining before resolving
    // keeps the answer independent of the working directory the tool was started from.
    let target = root.join(&link_target);
    let resolved = target.canonicalize().map_err(|error| {
        anyhow!(
            "{} points at {}, which does not resolve: {error}",
            display_path(&marker_path),
            display_path(&link_target)
        )
    })?;

    ensure!(
        resolved.is_dir(),
        "{} points at {}, which is not a directory",
        display_path(&marker_path),
        display_path(&resolved)
    );

    Ok(resolved)
}

/// Load and validate the config the spacetree repo at `spacetree_repo` holds.
///
/// Fails when the config file is missing or unreadable, when its TOML does not match the
/// declared shape, when `name` is absent, and when `name` is not a short label.
/// The name never falls back to a directory name.
fn load_spacetree_config(spacetree_repo: &Path) -> Result<SpacetreeConfig> {
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

/// Render `path` for humans, with the home directory shortened to `~`.
///
/// Reads the home directory from the environment; without it, the path is rendered as is.
fn display_path(path: &Path) -> String {
    match env::var_os("HOME") {
        Some(home) => shorten_home(path, Path::new(&home)),
        None => path.display().to_string(),
    }
}

/// Render `path` with `home` replaced by `~`, leaving paths outside `home` untouched.
///
/// Comparison is component-wise, so a sibling directory that merely shares the home name
/// prefix (`/Users/ben` against `/Users/ben-x`) is not shortened.
fn shorten_home(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(relative) if relative.as_os_str().is_empty() => "~".to_string(),
        Ok(relative) => format!("~/{}", relative.display()),
        Err(_) => path.display().to_string(),
    }
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

    /// Config text declaring every field, for the tests that assert the whole struct loads.
    const CONFIG_WITH_EVERY_FIELD: &str = "\
[spacetree]
name = \"work\"
description = \"Work trees for the greenhouse\"
variant = \"devtree\"
";

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

    /// Path of the repo directory inside a `make_spacetree` fixture.
    fn spacetree_repo(temp: &TempDir) -> PathBuf {
        temp.path().join("spacetree-home")
    }

    /// Write `text` as the config the repo of a `make_spacetree` fixture holds.
    fn write_spacetree_config(temp: &TempDir, text: &str) {
        fs::write(spacetree_repo(temp).join(CONFIG_FILENAME), text).expect("write config");
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

    #[test]
    fn info_json_carries_every_resolved_value() {
        let payload = render_info_json(
            Path::new("/trees/a tree"),
            Path::new("/trees/spacetree-home"),
            "work",
        );

        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload is valid JSON");

        // Compact form pins the whole contract: field set, values, and absence of extras.
        assert_eq!(
            parsed.to_string(),
            r#"{"spacetree":{"name":"work","repo":"/trees/spacetree-home","root":"/trees/a tree"}}"#
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

    #[test]
    fn shorten_home_renders_a_path_inside_home() {
        let rendered = shorten_home(Path::new("/Users/ben/self/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "~/self/tree");
    }

    #[test]
    fn shorten_home_renders_the_home_directory_itself() {
        let rendered = shorten_home(Path::new("/Users/ben"), Path::new("/Users/ben"));

        assert_eq!(rendered, "~");
    }

    #[test]
    fn shorten_home_leaves_a_path_outside_home() {
        let rendered = shorten_home(Path::new("/opt/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "/opt/tree");
    }

    #[test]
    fn shorten_home_leaves_a_sibling_sharing_the_home_name_prefix() {
        let rendered = shorten_home(Path::new("/Users/ben-x/tree"), Path::new("/Users/ben"));

        assert_eq!(rendered, "/Users/ben-x/tree");
    }
}
