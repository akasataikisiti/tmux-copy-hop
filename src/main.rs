use std::env;
use std::process::ExitCode;

use tmux_copy_hop::tmux::{Error, run_jump, run_popup};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("jump") => exit(run_jump()),
        Some("popup") => exit(run_popup(&args.collect::<Vec<_>>())),
        Some("--help") | Some("-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(command) => {
            eprintln!("tmux-copy-hop: unknown command '{command}'");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn exit(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Cancelled | Error::NoMatches(_) | Error::InvalidLabel) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tmux-copy-hop: {error}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    println!(
        "\
tmux-copy-hop

Usage:
  tmux-copy-hop jump
  tmux-copy-hop popup <pane-id> <width> <height> <was-copy-mode> <cursor-x> <cursor-y>

Commands:
  jump    Start a Hop-style pane jump
  popup   Internal command run inside tmux display-popup"
    );
}
