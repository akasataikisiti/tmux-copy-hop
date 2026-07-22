use std::cmp::Ordering;
use std::collections::HashSet;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const LABEL_ALPHABET: &str = "asdfghjklqwertyuiopzxcvbnm";
const TAB_WIDTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn distance_to(self, other: Point) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub point: Point,
    pub move_x: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledCandidate {
    pub point: Point,
    pub move_x: usize,
    pub label: String,
    pub display_start_x: usize,
}

pub fn find_candidates(lines: &[String], needle: char) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        let mut x = 0;
        let mut move_x = 0;

        for ch in line.chars() {
            if ch == needle {
                candidates.push(Candidate {
                    point: Point { x, y },
                    move_x,
                });
            }

            let width = char_width_at(ch, x);
            x += width;
            if width > 0 {
                move_x += 1;
            }
        }
    }

    candidates
}

pub fn find_line_candidates(lines: &[String]) -> Vec<Candidate> {
    lines
        .iter()
        .enumerate()
        .map(|(y, _)| Candidate {
            point: Point { x: 0, y },
            move_x: 0,
        })
        .collect()
}

pub fn sort_candidates_by_distance(candidates: &mut [Candidate], cursor: Point) {
    candidates.sort_by(|a, b| {
        a.point
            .distance_to(cursor)
            .cmp(&b.point.distance_to(cursor))
            .then_with(|| a.point.y.cmp(&b.point.y))
            .then_with(|| a.point.x.cmp(&b.point.x))
    });
}

pub fn label_width(candidate_count: usize) -> usize {
    if candidate_count == 0 {
        return 0;
    }

    let base = LABEL_ALPHABET.chars().count();
    let mut width = 1;
    let mut capacity = base;

    while candidate_count > capacity {
        width += 1;
        capacity *= base;
    }

    width
}

pub fn label_for_index(index: usize, width: usize) -> String {
    let alphabet: Vec<char> = LABEL_ALPHABET.chars().collect();
    let base = alphabet.len();
    let mut value = index;
    let mut chars = vec![alphabet[0]; width];

    for pos in (0..width).rev() {
        chars[pos] = alphabet[value % base];
        value /= base;
    }

    chars.into_iter().collect()
}

pub fn assign_labels(
    mut candidates: Vec<Candidate>,
    cursor: Point,
    pane_width: usize,
) -> Vec<LabeledCandidate> {
    if candidates.is_empty() || pane_width == 0 {
        return Vec::new();
    }

    sort_candidates_by_distance(&mut candidates, cursor);

    let one_char_labels = assign_labels_with_width(&candidates, 1, pane_width);
    let width = if candidates.len() <= LABEL_ALPHABET.chars().count()
        || one_char_labels.len() < LABEL_ALPHABET.chars().count()
    {
        return one_char_labels;
    } else {
        label_width(candidates.len())
    };

    assign_labels_with_width(&candidates, width, pane_width)
}

fn assign_labels_with_width(
    candidates: &[Candidate],
    width: usize,
    pane_width: usize,
) -> Vec<LabeledCandidate> {
    let mut occupied = HashSet::new();
    let mut labeled = Vec::new();

    for (index, candidate) in candidates.iter().enumerate() {
        if width == 1 && labeled.len() >= LABEL_ALPHABET.chars().count() {
            break;
        }

        let label_index = if width == 1 { labeled.len() } else { index };
        let label = label_for_index(label_index, width);
        let display_start_x = label_display_start(candidate.point.x, width, pane_width);
        let display_cells = display_start_x..display_start_x + width;

        if display_cells
            .clone()
            .any(|x| occupied.contains(&(x, candidate.point.y)))
        {
            continue;
        }

        for x in display_cells {
            occupied.insert((x, candidate.point.y));
        }

        labeled.push(LabeledCandidate {
            point: candidate.point,
            move_x: candidate.move_x,
            label,
            display_start_x,
        });
    }

    labeled
}

pub fn render_labeled_screen(
    lines: &[String],
    labeled: &[LabeledCandidate],
    pane_width: usize,
) -> String {
    render_labeled_screen_with_prefix(lines, labeled, pane_width, "")
}

pub fn render_labeled_screen_with_prefix(
    lines: &[String],
    labeled: &[LabeledCandidate],
    pane_width: usize,
    prefix: &str,
) -> String {
    let mut grid = lines
        .iter()
        .map(|line| line_to_cells(line, pane_width))
        .collect::<Vec<_>>();

    for candidate in labeled {
        if !prefix.is_empty() && !candidate.label.starts_with(prefix) {
            continue;
        }

        if let Some(row) = grid.get_mut(candidate.point.y) {
            let prefix_len = prefix.chars().count();
            let label_chars = candidate.label.chars().collect::<Vec<_>>();
            let chars = if prefix.is_empty() {
                label_chars.as_slice()
            } else {
                label_chars
                    .get(prefix_len..prefix_len.saturating_add(1))
                    .unwrap_or_default()
            };

            for (offset, ch) in chars.iter().enumerate() {
                let x = candidate.display_start_x
                    + if prefix.is_empty() {
                        offset
                    } else {
                        prefix_len + offset
                    };
                if let Some(cell) = row.get_mut(x) {
                    *cell = format!("\x1b[38;5;205;1m{ch}\x1b[0m");
                }
            }
        }
    }

    grid.into_iter()
        .map(|row| row.concat().trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_plain_screen(lines: &[String], pane_width: usize) -> String {
    lines
        .iter()
        .map(|line| {
            line_to_cells(line, pane_width)
                .concat()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn label_display_start(candidate_x: usize, label_width: usize, pane_width: usize) -> usize {
    match (
        label_width.cmp(&pane_width),
        candidate_x + label_width <= pane_width,
    ) {
        (Ordering::Greater, _) => 0,
        (_, true) => candidate_x,
        (_, false) => pane_width - label_width,
    }
}

fn line_to_cells(line: &str, pane_width: usize) -> Vec<String> {
    let mut cells = vec![" ".to_string(); pane_width];
    let mut x = 0;

    for ch in line.chars() {
        let width = char_width_at(ch, x);
        if width == 0 {
            continue;
        }
        if x >= pane_width {
            break;
        }

        if ch != '\t' {
            cells[x] = ch.to_string();
            for continuation_x in x + 1..(x + width).min(pane_width) {
                cells[continuation_x].clear();
            }
        }
        x += width;
    }

    cells
}

fn char_width_at(ch: char, x: usize) -> usize {
    if ch == '\t' {
        return TAB_WIDTH - (x % TAB_WIDTH);
    }

    UnicodeWidthChar::width(ch).unwrap_or(0)
}

pub fn visual_width(line: &str) -> usize {
    UnicodeWidthStr::width(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn candidate(x: usize, y: usize, move_x: usize) -> Candidate {
        Candidate {
            point: Point { x, y },
            move_x,
        }
    }

    #[test]
    fn finds_case_sensitive_matches_at_cell_positions() {
        let candidates = find_candidates(&lines(&["aA", "界a"]), 'a');

        assert_eq!(candidates, vec![candidate(0, 0, 0), candidate(2, 1, 1)]);
    }

    #[test]
    fn finds_the_start_of_every_line() {
        assert_eq!(
            find_line_candidates(&lines(&["first", "", "third"])),
            vec![candidate(0, 0, 0), candidate(0, 1, 0), candidate(0, 2, 0)]
        );
    }

    #[test]
    fn expands_tabs_to_terminal_cells_when_finding_candidates() {
        let candidates = find_candidates(&lines(&["a\tb", "1234567\tb"]), 'b');

        assert_eq!(candidates, vec![candidate(8, 0, 2), candidate(8, 1, 8)]);
    }

    #[test]
    fn finds_ascii_matches_after_japanese_text_at_cell_positions() {
        let candidates = find_candidates(&lines(&["日本語abc"]), 'a');

        assert_eq!(candidates, vec![candidate(6, 0, 3)]);
    }

    #[test]
    fn tracks_copy_mode_move_count_separately_from_display_cells() {
        assert_eq!(
            find_candidates(&lines(&["😀abc"]), 'a'),
            vec![candidate(2, 0, 1)]
        );
        assert_eq!(
            find_candidates(&lines(&["e\u{301}abc"]), 'a'),
            vec![candidate(1, 0, 1)]
        );
    }

    #[test]
    fn sorts_by_cursor_distance_then_screen_order() {
        let mut candidates = vec![candidate(0, 0, 0), candidate(5, 2, 5), candidate(3, 1, 3)];

        sort_candidates_by_distance(&mut candidates, Point { x: 4, y: 1 });

        assert_eq!(
            candidates.iter().map(|c| c.point).collect::<Vec<_>>(),
            vec![
                Point { x: 3, y: 1 },
                Point { x: 5, y: 2 },
                Point { x: 0, y: 0 },
            ]
        );
    }

    #[test]
    fn grows_label_width_uniformly() {
        assert_eq!(label_width(0), 0);
        assert_eq!(label_width(1), 1);
        assert_eq!(label_width(26), 1);
        assert_eq!(label_width(27), 2);
    }

    #[test]
    fn generates_fixed_width_labels() {
        assert_eq!(label_for_index(0, 1), "a");
        assert_eq!(label_for_index(1, 1), "s");
        assert_eq!(label_for_index(0, 2), "aa");
        assert_eq!(label_for_index(1, 2), "as");
        assert_eq!(label_for_index(26, 2), "sa");
    }

    #[test]
    fn assigns_labels_near_cursor_first() {
        let candidates = find_candidates(&lines(&["a---a", "--a--"]), 'a');
        let labeled = assign_labels(candidates, Point { x: 2, y: 1 }, 5);

        assert_eq!(labeled[0].point, Point { x: 2, y: 1 });
        assert_eq!(labeled[0].label, "a");
    }

    #[test]
    fn shifts_wide_label_left_at_right_edge() {
        let candidates = (0..27)
            .map(|index| Candidate {
                point: Point { x: index, y: 0 },
                move_x: index,
            })
            .collect::<Vec<_>>();
        let labeled = assign_labels(candidates, Point { x: 26, y: 0 }, 27);

        assert_eq!(labeled[0].point, Point { x: 26, y: 0 });
        assert_eq!(labeled[0].display_start_x, 25);
    }

    #[test]
    fn uses_one_character_labels_when_collisions_leave_fewer_than_26_candidates() {
        let mut candidates = (0..25)
            .map(|index| Candidate {
                point: Point { x: index, y: 0 },
                move_x: index,
            })
            .collect::<Vec<_>>();
        candidates.extend((0..5).map(|_| Candidate {
            point: Point { x: 0, y: 0 },
            move_x: 0,
        }));

        let labeled = assign_labels(candidates, Point { x: 0, y: 0 }, 25);

        assert_eq!(labeled.len(), 25);
        assert!(labeled.iter().all(|candidate| candidate.label.len() == 1));
    }

    #[test]
    fn drops_colliding_labels() {
        let candidates = vec![candidate(0, 0, 0), candidate(1, 0, 1)];
        let labeled = assign_labels(candidates, Point { x: 0, y: 0 }, 1);

        assert_eq!(labeled.len(), 1);
    }

    #[test]
    fn renders_labels_over_plain_text() {
        let rendered = render_labeled_screen(
            &lines(&["abc"]),
            &[LabeledCandidate {
                point: Point { x: 1, y: 0 },
                move_x: 1,
                label: "s".to_string(),
                display_start_x: 1,
            }],
            3,
        );

        assert_eq!(rendered, "a\u{1b}[38;5;205;1ms\u{1b}[0mc");
    }

    #[test]
    fn renders_plain_screen_with_cell_widths() {
        let rendered = render_plain_screen(&lines(&["a\tb", "界a", "日本語abc"]), 12);

        assert_eq!(rendered, "a       b\n界a\n日本語abc");
        assert_eq!(visual_width("界a"), 3);
        assert_eq!(visual_width("日本語abc"), 9);
    }

    #[test]
    fn renders_labels_after_japanese_text_without_extra_padding() {
        let rendered = render_labeled_screen(
            &lines(&["日本語abc"]),
            &[LabeledCandidate {
                point: Point { x: 6, y: 0 },
                move_x: 3,
                label: "a".to_string(),
                display_start_x: 6,
            }],
            12,
        );

        assert_eq!(rendered, "日本語\u{1b}[38;5;205;1ma\u{1b}[0mbc");
    }

    #[test]
    fn renders_only_next_label_character_after_prefix() {
        let rendered = render_labeled_screen_with_prefix(
            &lines(&["abcdef"]),
            &[
                LabeledCandidate {
                    point: Point { x: 0, y: 0 },
                    move_x: 0,
                    label: "aa".to_string(),
                    display_start_x: 0,
                },
                LabeledCandidate {
                    point: Point { x: 3, y: 0 },
                    move_x: 3,
                    label: "as".to_string(),
                    display_start_x: 3,
                },
                LabeledCandidate {
                    point: Point { x: 5, y: 0 },
                    move_x: 5,
                    label: "sa".to_string(),
                    display_start_x: 5,
                },
            ],
            6,
            "a",
        );

        assert_eq!(
            rendered,
            "a\u{1b}[38;5;205;1ma\u{1b}[0mcd\u{1b}[38;5;205;1ms\u{1b}[0mf"
        );
    }
}
