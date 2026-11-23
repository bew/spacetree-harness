// Resolve the enclosing spacetree and print what it resolves to.

use std::env;
use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};

mod spacetree;

use spacetree::SpacetreeContext;

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
/// the shared namespace, and its declared name.
///
/// Fails when `--json` is absent, matching the tool's current output contract.
fn run_info(emit_json: bool) -> Result<()> {
    ensure!(emit_json, "info requires --json");

    let start = env::current_dir()?;
    let context = SpacetreeContext::resolve(&start)?;

    print_json(&context.render_json()?);
    Ok(())
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
