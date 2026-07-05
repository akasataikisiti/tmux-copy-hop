use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("jump") => {
            eprintln!("tmux-copy-hop: jump is not implemented yet");
            ExitCode::from(1)
        }
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

fn print_help() {
    println!(
        "\
tmux-copy-hop

Usage:
  tmux-copy-hop jump

Commands:
  jump    Start a Hop-style pane jump"
    );
}
