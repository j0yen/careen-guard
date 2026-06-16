//! careen-guard binary entry point.
//!
//! Exit codes:
//!   0 — ok (below advisory)
//!   2 — warn (advisory band, no sweep)
//!   3 — breach-acted (sweep ran, now below high-water)
//!   4 — breach-unresolved (sweep exhausted live candidates; still above high-water)

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use careen_guard::guard;

/// careen-guard: SLO-triggered sweep of live target dirs.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Run a single SLO evaluation pass and exit.
    Run(RunArgs),
}

/// Arguments for the `run` subcommand.
#[derive(Debug, clap::Args)]
struct RunArgs {
    /// Path to guard.toml (default: ~/.config/careen/guard.toml).
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    /// Filesystem path to monitor (default: /).
    #[arg(long, default_value = "/")]
    mount: std::path::PathBuf,

    /// Append JSON events to this file in addition to stdout.
    #[arg(long)]
    event_sink: Option<std::path::PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("careen-guard: {e:#}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Run(args) => {
            let run_args = guard::RunArgs {
                config: args.config,
                mount: args.mount,
                event_sink: args.event_sink,
            };
            guard::run(&run_args)
        }
    }
}
