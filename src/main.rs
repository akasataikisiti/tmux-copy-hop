use std::env;
use std::process::ExitCode;

use tmux_copy_hop::tmux::{Error, display_message, run_jump, run_popup};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next();

    match command.as_deref().map(normalize_command) {
        Some("jump") => exit(run_jump()),
        Some("popup") => exit(run_popup(&args.collect::<Vec<_>>())),
        Some("--help") | Some("-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(_) => {
            let command = command.unwrap_or_default();
            eprintln!("tmux-copy-hop: unknown command '{command}'");
            eprintln!("Run 'tmux-copy-hop --help' for usage.");
            ExitCode::from(2)
        }
    }
}

fn normalize_command(command: &str) -> &str {
    command.trim_matches(|ch: char| ch.is_ascii_whitespace() || ch.is_control())
}

fn exit(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Cancelled | Error::NoMatches(_) | Error::InvalidLabel) => ExitCode::SUCCESS,
        Err(error) => {
            report_error(&format!("tmux-copy-hop: {error}"));
            ExitCode::from(1)
        }
    }
}

fn report_error(message: &str) {
    if env::var_os("TMUX").is_some() || env::var_os("TMUX_COPY_HOP_SOCKET").is_some() {
        display_message(message);
    } else {
        eprintln!("{message}");
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

#[cfg(test)]
mod tests {
    use super::normalize_command;

    #[test]
    fn normalizes_tmux_config_line_artifacts() {
        assert_eq!(normalize_command("jump"), "jump");
        assert_eq!(normalize_command("jump\n"), "jump");
        assert_eq!(normalize_command("jump\r"), "jump");
        assert_eq!(normalize_command("\u{1b}jump"), "jump");
    }
}
