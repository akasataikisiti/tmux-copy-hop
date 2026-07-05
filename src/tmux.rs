use std::env;
use std::fmt;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

use crate::hop::{Point, assign_labels, find_candidates, render_labeled_screen};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Tmux(String),
    Parse(String),
    Cancelled,
    NoMatches(char),
    InvalidLabel,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{error}"),
            Error::Tmux(message) => write!(f, "tmux error: {message}"),
            Error::Parse(message) => write!(f, "parse error: {message}"),
            Error::Cancelled => write!(f, "cancelled"),
            Error::NoMatches(needle) => write!(f, "no matches for '{needle}'"),
            Error::InvalidLabel => write!(f, "invalid label"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneInfo {
    pub pane_id: String,
    pub width: usize,
    pub height: usize,
    pub mode: String,
    pub pane_cursor: Point,
    pub copy_cursor: Option<Point>,
}

impl PaneInfo {
    pub fn is_copy_mode(&self) -> bool {
        self.mode.contains("copy")
    }

    pub fn cursor_for_labeling(&self) -> Point {
        if self.is_copy_mode() {
            self.copy_cursor.unwrap_or(self.pane_cursor)
        } else {
            self.pane_cursor
        }
    }
}

pub fn run_jump() -> Result<()> {
    let pane = current_pane_info()?;
    let exe = env::current_exe()?;
    let command = format!(
        "{} popup {} {} {} {} {} {}",
        shell_quote(&exe.to_string_lossy()),
        shell_quote(&pane.pane_id),
        pane.width,
        pane.height,
        if pane.is_copy_mode() { "1" } else { "0" },
        pane.cursor_for_labeling().x,
        pane.cursor_for_labeling().y
    );

    tmux_status(
        tmux_command()
            .arg("display-popup")
            .arg("-E")
            .arg("-w")
            .arg(pane.width.to_string())
            .arg("-h")
            .arg(pane.height.to_string())
            .arg(command),
    )
}

pub fn run_popup(args: &[String]) -> Result<()> {
    let popup = PopupArgs::parse(args)?;
    let _raw_mode = RawMode::enable()?;

    print!("jump key: ");
    io::stdout().flush()?;
    let needle = read_ascii_char()?;

    let lines = normalize_lines(capture_visible_lines(&popup.pane_id)?, popup.height);
    let candidates = find_candidates(&lines, needle);
    if candidates.is_empty() {
        display_message(&format!("tmux-copy-hop: no matches for '{needle}'"));
        return Err(Error::NoMatches(needle));
    }

    let labeled = assign_labels(candidates, popup.cursor, popup.width);
    let label_width = labeled
        .first()
        .map(|candidate| candidate.label.len())
        .unwrap_or(0);
    let rendered = render_labeled_screen(&lines, &labeled, popup.width);

    print!("\x1b[2J\x1b[H{rendered}");
    io::stdout().flush()?;

    let label = read_ascii_string(label_width)?;
    let target = labeled
        .iter()
        .find(|candidate| candidate.label == label)
        .map(|candidate| candidate.point)
        .ok_or(Error::InvalidLabel);

    let target = match target {
        Ok(target) => target,
        Err(error) => {
            display_message("tmux-copy-hop: invalid label");
            return Err(error);
        }
    };

    move_to_target(&popup.pane_id, popup.was_copy_mode, target)?;

    Ok(())
}

fn current_pane_info() -> Result<PaneInfo> {
    let output = tmux_output(
        tmux_command()
            .arg("display-message")
            .arg("-p")
            .arg("-F")
            .arg(
                "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_mode}\t#{cursor_x}\t#{cursor_y}\t#{copy_cursor_x}\t#{copy_cursor_y}",
            ),
    )?;

    parse_pane_info(output.trim_end())
}

fn parse_pane_info(value: &str) -> Result<PaneInfo> {
    let fields = value.split('\t').collect::<Vec<_>>();
    if fields.len() != 8 {
        return Err(Error::Parse(format!(
            "expected 8 pane fields, got {}",
            fields.len()
        )));
    }

    let pane_cursor = Point {
        x: parse_usize(fields[4], "cursor_x")?,
        y: parse_usize(fields[5], "cursor_y")?,
    };
    let copy_cursor = match (fields[6].parse::<usize>(), fields[7].parse::<usize>()) {
        (Ok(x), Ok(y)) => Some(Point { x, y }),
        _ => None,
    };

    Ok(PaneInfo {
        pane_id: fields[0].to_string(),
        width: parse_usize(fields[1], "pane_width")?,
        height: parse_usize(fields[2], "pane_height")?,
        mode: fields[3].to_string(),
        pane_cursor,
        copy_cursor,
    })
}

fn capture_visible_lines(pane_id: &str) -> Result<Vec<String>> {
    let output = tmux_output(
        tmux_command()
            .arg("capture-pane")
            .arg("-p")
            .arg("-N")
            .arg("-t")
            .arg(pane_id),
    )?;

    Ok(output.lines().map(|line| line.to_string()).collect())
}

fn normalize_lines(mut lines: Vec<String>, height: usize) -> Vec<String> {
    lines.truncate(height);
    while lines.len() < height {
        lines.push(String::new());
    }

    lines
}

fn move_to_target(pane_id: &str, was_copy_mode: bool, target: Point) -> Result<()> {
    if !was_copy_mode {
        tmux_status(tmux_command().arg("copy-mode").arg("-t").arg(pane_id))?;
    }

    let current = current_copy_cursor(pane_id)?;
    move_vertical(pane_id, current.y, target.y)?;
    move_horizontal(pane_id, current.x, target.x)?;

    Ok(())
}

fn current_copy_cursor(pane_id: &str) -> Result<Point> {
    let output = tmux_output(
        tmux_command()
            .arg("display-message")
            .arg("-p")
            .arg("-t")
            .arg(pane_id)
            .arg("-F")
            .arg("#{copy_cursor_x}\t#{copy_cursor_y}"),
    )?;
    let fields = output.trim_end().split('\t').collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err(Error::Parse("expected copy cursor x/y".to_string()));
    }

    Ok(Point {
        x: parse_usize(fields[0], "copy_cursor_x")?,
        y: parse_usize(fields[1], "copy_cursor_y")?,
    })
}

fn move_vertical(pane_id: &str, current_y: usize, target_y: usize) -> Result<()> {
    if target_y > current_y {
        repeat_copy_command(pane_id, "cursor-down", target_y - current_y)
    } else {
        repeat_copy_command(pane_id, "cursor-up", current_y - target_y)
    }
}

fn move_horizontal(pane_id: &str, current_x: usize, target_x: usize) -> Result<()> {
    if target_x > current_x {
        repeat_copy_command(pane_id, "cursor-right", target_x - current_x)
    } else {
        repeat_copy_command(pane_id, "cursor-left", current_x - target_x)
    }
}

fn repeat_copy_command(pane_id: &str, copy_command: &str, count: usize) -> Result<()> {
    if count == 0 {
        return Ok(());
    }

    tmux_status(
        tmux_command()
            .arg("send-keys")
            .arg("-t")
            .arg(pane_id)
            .arg("-X")
            .arg("-N")
            .arg(count.to_string())
            .arg(copy_command),
    )
}

fn display_message(message: &str) {
    let _ = tmux_command().arg("display-message").arg(message).status();
}

fn tmux_command() -> Command {
    let mut command = Command::new("tmux");
    if let Ok(socket) = env::var("TMUX_COPY_HOP_SOCKET") {
        command.arg("-S").arg(socket);
    }

    command
}

fn tmux_output(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(Error::Tmux(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn tmux_status(command: &mut Command) -> Result<()> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(Error::Tmux(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(())
}

fn parse_usize(value: &str, name: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .map_err(|_| Error::Parse(format!("invalid {name}: {value:?}")))
}

fn read_ascii_char() -> Result<char> {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer)?;
    match buffer[0] {
        0x03 | 0x1b => Err(Error::Cancelled),
        byte if byte.is_ascii() => Ok(byte as char),
        byte => Err(Error::Parse(format!("non-ASCII input byte: {byte}"))),
    }
}

fn read_ascii_string(width: usize) -> Result<String> {
    let mut value = String::new();

    for _ in 0..width {
        value.push(read_ascii_char()?);
    }

    Ok(value)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

struct RawMode {
    saved: String,
}

impl RawMode {
    fn enable() -> Result<Self> {
        let output = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(Error::Tmux(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let saved = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let status = Command::new("stty")
            .args(["raw", "-echo", "min", "1", "time", "0"])
            .stdin(Stdio::inherit())
            .status()?;
        if !status.success() {
            return Err(Error::Tmux(
                "failed to enable raw terminal mode".to_string(),
            ));
        }

        Ok(Self { saved })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = Command::new("stty")
            .arg(&self.saved)
            .stdin(Stdio::inherit())
            .status();
    }
}

struct PopupArgs {
    pane_id: String,
    width: usize,
    height: usize,
    was_copy_mode: bool,
    cursor: Point,
}

impl PopupArgs {
    fn parse(args: &[String]) -> Result<Self> {
        if args.len() != 6 {
            return Err(Error::Parse(format!(
                "popup expects 6 args, got {}",
                args.len()
            )));
        }

        Ok(Self {
            pane_id: args[0].clone(),
            width: parse_usize(&args[1], "width")?,
            height: parse_usize(&args[2], "height")?,
            was_copy_mode: args[3] == "1",
            cursor: Point {
                x: parse_usize(&args[4], "cursor_x")?,
                y: parse_usize(&args[5], "cursor_y")?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pane_info_with_copy_cursor() {
        let pane = parse_pane_info("%1\t80\t24\tcopy-mode\t10\t20\t3\t4").unwrap();

        assert_eq!(pane.pane_id, "%1");
        assert_eq!(pane.width, 80);
        assert_eq!(pane.height, 24);
        assert_eq!(pane.pane_cursor, Point { x: 10, y: 20 });
        assert_eq!(pane.copy_cursor, Some(Point { x: 3, y: 4 }));
        assert_eq!(pane.cursor_for_labeling(), Point { x: 3, y: 4 });
    }

    #[test]
    fn parses_pane_info_without_copy_cursor() {
        let pane = parse_pane_info("%1\t80\t24\t\t10\t20\t\t").unwrap();

        assert!(!pane.is_copy_mode());
        assert_eq!(pane.copy_cursor, None);
        assert_eq!(pane.cursor_for_labeling(), Point { x: 10, y: 20 });
    }

    #[test]
    fn shell_quotes_single_quotes() {
        assert_eq!(shell_quote("/tmp/it's/bin"), "'/tmp/it'\\''s/bin'");
    }

    #[test]
    fn normalizes_lines_to_popup_height() {
        assert_eq!(normalize_lines(vec!["a".into(), "b".into()], 1), vec!["a"]);
        assert_eq!(
            normalize_lines(vec!["a".into()], 2),
            vec!["a".to_string(), String::new()]
        );
    }
}
