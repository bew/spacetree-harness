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

    let payload = format!(
        "{{\"name\":{},\"path\":{},\"spacetree_repo\":{}}}",
        json_string(&spacetree_name),
        json_string(&root.display().to_string()),
        json_string(&spacetree_repo.display().to_string())
    );
    print_json(&payload);
    Ok(())
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
    use super::json_string;

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
