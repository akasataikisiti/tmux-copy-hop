use std::cmp::Ordering;
use std::collections::HashSet;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const LABEL_ALPHABET: &str = "asdfghjklqwertyuiopzxcvbnm";

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledCandidate {
    pub point: Point,
    pub label: String,
    pub display_start_x: usize,
}

pub fn find_candidates(lines: &[String], needle: char) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        let mut x = 0;

        for ch in line.chars() {
            if ch == needle {
                candidates.push(Candidate {
                    point: Point { x, y },
                });
            }

            x += char_width(ch);
        }
    }

    candidates
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

    let width = label_width(candidates.len());
    let mut occupied = HashSet::new();
    let mut labeled = Vec::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let label = label_for_index(index, width);
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
    let mut grid = lines
        .iter()
        .map(|line| line_to_cells(line, pane_width))
        .collect::<Vec<_>>();

    for candidate in labeled {
        if let Some(row) = grid.get_mut(candidate.point.y) {
            for (offset, ch) in candidate.label.chars().enumerate() {
                let x = candidate.display_start_x + offset;
                if let Some(cell) = row.get_mut(x) {
                    *cell = format!("\x1b[34;1m{ch}\x1b[0m");
                }
            }
        }
    }

    grid.into_iter()
        .map(|row| row.concat().trim_end().to_string())
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
        let width = char_width(ch);
        if width == 0 {
            continue;
        }
        if x >= pane_width {
            break;
        }

        cells[x] = ch.to_string();
        x += width;
    }

    cells
}

fn char_width(ch: char) -> usize {
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

    #[test]
    fn finds_case_sensitive_matches_at_cell_positions() {
        let candidates = find_candidates(&lines(&["aA", "界a"]), 'a');

        assert_eq!(
            candidates,
            vec![
                Candidate {
                    point: Point { x: 0, y: 0 }
                },
                Candidate {
                    point: Point { x: 2, y: 1 }
                }
            ]
        );
    }

    #[test]
    fn sorts_by_cursor_distance_then_screen_order() {
        let mut candidates = vec![
            Candidate {
                point: Point { x: 0, y: 0 },
            },
            Candidate {
                point: Point { x: 5, y: 2 },
            },
            Candidate {
                point: Point { x: 3, y: 1 },
            },
        ];

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
            })
            .collect::<Vec<_>>();
        let labeled = assign_labels(candidates, Point { x: 26, y: 0 }, 27);

        assert_eq!(labeled[0].point, Point { x: 26, y: 0 });
        assert_eq!(labeled[0].display_start_x, 25);
    }

    #[test]
    fn drops_colliding_labels() {
        let candidates = vec![
            Candidate {
                point: Point { x: 0, y: 0 },
            },
            Candidate {
                point: Point { x: 1, y: 0 },
            },
        ];
        let labeled = assign_labels(candidates, Point { x: 0, y: 0 }, 1);

        assert_eq!(labeled.len(), 1);
    }

    #[test]
    fn renders_labels_over_plain_text() {
        let rendered = render_labeled_screen(
            &lines(&["abc"]),
            &[LabeledCandidate {
                point: Point { x: 1, y: 0 },
                label: "s".to_string(),
                display_start_x: 1,
            }],
            3,
        );

        assert_eq!(rendered, "a\u{1b}[34;1ms\u{1b}[0mc");
    }
}
