pub struct Form<'a> {
    pub line_widths_em: &'a [f32],
    pub block_line_counts: &'a [usize],
    pub words: usize,
}

pub struct Layout {
    pub column_em: f32,
    pub leading: f32,
    pub block_gap_em: f32,
    pub inset_em: f32,
    pub shortness: f32,
}

pub fn compose(form: Form<'_>) -> Layout {
    let n = form.line_widths_em.len().max(1) as f32;
    let mean = form.line_widths_em.iter().sum::<f32>() / n;
    let longest = form.line_widths_em.iter().copied().fold(0.0, f32::max);
    let rhythm = (form
        .line_widths_em
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>()
        / n)
        .sqrt()
        / mean.max(1.0);
    let block_mean = n / form.block_line_counts.len().max(1) as f32;
    let density = (form.words as f32 / n / 10.0).clamp(0.0, 1.0);
    let breaks = (form.block_line_counts.len() as f32 / 8.0).min(1.0);

    Layout {
        column_em: (longest + 0.7).max(17.0),
        leading: (1.53 + density * 0.10 + rhythm * 0.10).clamp(1.55, 1.68),
        block_gap_em: (1.28 + block_mean * 0.06 + (1.0 - breaks) * 0.16).clamp(1.42, 1.95),
        inset_em: (2.9 - longest * 0.065 + rhythm * 1.7).clamp(0.2, 2.8),
        shortness: ((25.0 - n) / 20.0)
            .min((150.0 - form.words as f32) / 110.0)
            .clamp(0.0, 1.0),
    }
}

/// Derive the breath at each existing stanza or paragraph boundary.
pub fn cadence(blocks: &[Vec<String>]) -> Vec<f64> {
    let words: Vec<_> = blocks
        .iter()
        .map(|block| {
            block
                .iter()
                .map(|line| line.split_whitespace().count())
                .sum::<usize>()
        })
        .collect();
    let heading = |block: &[String]| {
        block.len() == 1
            && (block[0] == "(end)"
                || block[0]
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                    .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())))
    };
    blocks
        .windows(2)
        .zip(words.windows(2))
        .map(|(pair, lengths)| {
            if heading(&pair[1]) {
                return 1.4;
            }
            if heading(&pair[0]) {
                return 0.6;
            }
            let close = pair[0].last().is_some_and(|line| {
                line.trim_end_matches([' ', '”', '’', '"', ')'])
                    .ends_with(['.', '!', '?', '。', '！', '？', '…'])
            });
            let size = (lengths[0].abs_diff(lengths[1]) as f64 / 72.0).min(0.2);
            let pause = if close { 1.03 } else { 0.88 };
            pause + size
        })
        .chain(std::iter::once(0.0))
        .collect()
}

/// Upper triangular matrix of shaped phrase widths indexed by [first * n + last].
/// Break costs: 0 for strong punctuation, 1 for soft punctuation, 2 for a
/// phrase boundary, 3 for any other word boundary.
pub fn mobile_breaks(widths: &[f32], breaks: &[f32], width: f32, indent: f32) -> Vec<usize> {
    let n = breaks.len();
    if n == 0 || widths[n - 1] <= width {
        return Vec::new();
    }
    let mut best = vec![(usize::MAX, f32::INFINITY); n + 1];
    let mut next = vec![n; n];
    best[n] = (0, 0.0);
    for start in (0..n).rev() {
        let room = if start == 0 {
            width
        } else {
            (width - indent).max(1.0)
        };
        for end in start..n {
            let used = widths[start * n + end];
            if used > room && end > start {
                break;
            }
            let slack = ((room - used) / room).max(0.0);
            let wrap_cost = if end + 1 == n {
                if end == start { 1.05 } else { 0.08 * slack }
            } else {
                0.15 * breaks[end] + 5.6 * slack.powi(2)
            };
            let (rest_rows, rest_cost) = best[end + 1];
            let score = (rest_rows.saturating_add(1), wrap_cost + rest_cost);
            if score < best[start] {
                best[start] = score;
                next[start] = end + 1;
            }
        }
    }
    let mut cuts = Vec::new();
    let mut start = next[0];
    while start < n {
        cuts.push(start);
        start = next[start];
    }
    cuts
}

/// Pick visual breathing points within a long stanza at existing line boundaries.
pub fn mobile_pauses(rows: &[usize], lines: &[String]) -> Vec<usize> {
    let mut remaining = rows.iter().sum::<usize>();
    let mut start = 0;
    let mut pauses = Vec::new();
    while remaining >= 19 {
        let mut used = 0;
        let mut choice = None;
        let mut best = usize::MAX;
        for (line, &count) in rows.iter().enumerate().skip(start) {
            used += count;
            if used > 15 || remaining - used < 6 {
                break;
            }
            if used < 8 {
                continue;
            }
            let text = lines[line].trim_end_matches([' ', '”', '’', '"', ')']);
            let punctuation = if text.ends_with(['.', '!', '?', '…', '。', '！', '？']) {
                0
            } else if text.ends_with([';', ':', '；', '：']) {
                2
            } else {
                4
            };
            let cost = used.abs_diff(11) + punctuation;
            if cost < best {
                best = cost;
                choice = Some((line + 1, used));
            }
        }
        let Some((line, used)) = choice else {
            break;
        };
        pauses.push(line);
        start = line;
        remaining -= used;
    }
    pauses
}

/// Fill one spread in reading order. Prefer the split that fits the most
/// complete blocks, then balance the used columns without forcing a split.
/// Reserve continuation space beneath the last occupied column only.
pub fn fit_spread(
    heights: &[Vec<f64>],
    capacity: &[f64],
    start: usize,
    gap_after: &[f64],
    continuation: f64,
) -> Vec<std::ops::Range<usize>> {
    let count = heights[0].len();
    let mut best = vec![start..start; heights.len()];
    let mut best_end = start;
    let mut best_cost = (true, f64::INFINITY);
    let candidates = |column: usize, first: usize| {
        let mut used = 0.0;
        let mut ends = vec![(first, used)];
        for end in first..count {
            used += heights[column][end]
                + if end == first {
                    0.0
                } else {
                    gap_after[end - 1]
                };
            if used > capacity[column] {
                break;
            }
            ends.push((end + 1, used));
        }
        ends
    };
    for (split, left) in candidates(0, start) {
        if heights.len() == 1 {
            if left + continuation <= capacity[0] {
                best[0] = start..split;
            }
            continue;
        }
        for (end, right) in candidates(1, split) {
            let (used, room) = if end > split {
                (right, capacity[1])
            } else {
                (left, capacity[0])
            };
            if used + continuation > room {
                continue;
            }
            let cost = (split == start, (left - right).abs());
            if end > best_end || (end == best_end && cost < best_cost) {
                best[0] = start..split;
                best[1] = split..end;
                best_end = end;
                best_cost = cost;
            }
        }
    }
    best
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// Find the first vertically safe position while keeping the horizontal placement.
pub fn avoid_overlap(rect: Rectangle, placed: &[Rectangle], gap: f64) -> Rectangle {
    let mut result = rect;
    loop {
        let next = placed
            .iter()
            .filter(|other| {
                result.left < other.left + other.width - 0.5
                    && other.left < result.left + result.width - 0.5
                    && result.top < other.top + other.height + gap
                    && other.top < result.top + result.height + gap
            })
            .map(|other| other.top + other.height + gap)
            .fold(result.top, f64::max);
        if next <= result.top {
            return result;
        }
        result.top = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_wrap_prefers_the_fewest_rows() {
        let widths = [
            23.0,
            48.0,
            76.0,
            103.0,
            f32::INFINITY,
            23.0,
            49.0,
            77.0,
            f32::INFINITY,
            f32::INFINITY,
            24.0,
            50.0,
            f32::INFINITY,
            f32::INFINITY,
            f32::INFINITY,
            24.0,
        ];
        assert_eq!(
            mobile_breaks(&widths, &[0.0, 7.0, 0.0, 0.0], 60.0, 8.0),
            vec![2]
        );
    }

    #[test]
    fn mobile_breathing_points_favour_sentence_ends_and_keep_short_stanzas_intact() {
        let mut lines = vec!["a line,".to_owned(); 24];
        lines[9] = "end of sentence.".to_owned();
        assert_eq!(mobile_pauses(&[1; 24], &lines), vec![10]);
        assert!(mobile_pauses(&[1; 18], &lines[..18]).is_empty());
    }

    #[test]
    fn fills_current_spread_before_continuing() {
        let heights = vec![vec![160.0, 80.0, 70.0, 130.0, 160.0]; 2];
        assert_eq!(
            fit_spread(&heights, &[250.0; 2], 0, &[10.0; 5], 0.0),
            vec![0..2, 2..4]
        );
        assert_eq!(
            fit_spread(&heights, &[250.0; 2], 4, &[10.0; 5], 0.0),
            vec![4..5, 5..5]
        );
        assert_eq!(
            fit_spread(&heights, &[120.0; 2], 4, &[10.0; 5], 0.0),
            vec![4..4, 4..4]
        );
    }

    #[test]
    fn continuation_only_uses_room_in_the_last_occupied_column() {
        let heights = vec![vec![120.0, 100.0, 150.0, 120.0]; 2];
        assert_eq!(
            fit_spread(&heights, &[230.0; 2], 0, &[5.0; 4], 25.0),
            vec![0..2, 2..3]
        );
        let one_column = vec![vec![210.0], vec![300.0]];
        assert_eq!(
            fit_spread(&one_column, &[230.0; 2], 0, &[0.0], 25.0),
            vec![0..0, 0..0]
        );
    }

    #[test]
    fn columns_keep_short_siblings_and_stacked_frames_clear() {
        let left = Rectangle {
            left: 0.0,
            top: 0.0,
            width: 150.0,
            height: 180.0,
        };
        let right = Rectangle {
            left: 180.0,
            ..left
        };
        let below = Rectangle { top: 100.0, ..left };
        assert_eq!(avoid_overlap(right, &[left], 20.0).top, 0.0);
        assert_eq!(avoid_overlap(below, &[left], 20.0).top, 200.0);
    }
}
