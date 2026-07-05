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
            report_error(&format!(
                "tmux-copy-hop: unknown command '{}'",
                display_command(&command)
            ));
            failure_exit_code()
        }
    }
}

fn normalize_command(command: &str) -> &str {
    let command =
        command.trim_start_matches(|ch: char| ch.is_ascii_whitespace() || ch.is_control());
    command
        .split_once(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .map(|(command, _)| command)
        .unwrap_or(command)
        .trim_end_matches(|ch: char| ch.is_ascii_whitespace() || ch.is_control())
}

fn display_command(command: &str) -> String {
    command
        .chars()
        .flat_map(|ch| ch.escape_default())
        .collect::<String>()
}

fn exit(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Cancelled | Error::NoMatches(_) | Error::InvalidLabel) => ExitCode::SUCCESS,
        Err(error) => {
            report_error(&format!("tmux-copy-hop: {error}"));
            failure_exit_code()
        }
    }
}

fn report_error(message: &str) {
    if running_inside_tmux() {
        display_message(message);
    } else {
        eprintln!("{message}");
    }
}

fn failure_exit_code() -> ExitCode {
    if running_inside_tmux() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn running_inside_tmux() -> bool {
    env::var_os("TMUX").is_some() || env::var_os("TMUX_COPY_HOP_SOCKET").is_some()
}

fn print_help() {
    println!(
        "\
tmux-copy-hop

Usage:
  tmux-copy-hop jump
  tmux-copy-hop popup <pane-id> <width> <height> <was-copy-mode> <cursor-x> <cursor-y> <scroll-position>

Commands:
  jump    Start a Hop-style pane jump
  popup   Internal command run inside tmux display-popup"
    );
}

#[cfg(test)]
mod tests {
    use super::{display_command, normalize_command};

    #[test]
    fn normalizes_tmux_config_line_artifacts() {
        assert_eq!(normalize_command("jump"), "jump");
        assert_eq!(normalize_command("jump\n"), "jump");
        assert_eq!(normalize_command("jump\r"), "jump");
        assert_eq!(normalize_command("\u{1b}jump"), "jump");
        assert_eq!(normalize_command("jump\u{21b4}"), "jump");
        assert_eq!(normalize_command("jump\u{23ce}"), "jump");
        assert_eq!(normalize_command("jump anything-after"), "jump");
    }

    #[test]
    fn escapes_unknown_command_for_status_message() {
        assert_eq!(display_command("jump\n"), "jump\\n");
    }
}
