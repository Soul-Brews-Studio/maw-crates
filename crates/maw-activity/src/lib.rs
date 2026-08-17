//! Dependency-free activity classification for terminal snapshots.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityState {
    Busy,
    Idle,
    Stuck,
}

impl ActivityState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Busy => "busy",
            Self::Idle => "idle",
            Self::Stuck => "stuck",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityConfidence {
    Low,
    Medium,
    High,
}

impl ActivityConfidence {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivitySample {
    pub text: String,
    pub at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityResult {
    pub pane: String,
    pub state: ActivityState,
    pub confidence: ActivityConfidence,
    pub samples: u32,
    pub diff_samples: u32,
    pub last_change_ago_seconds: f64,
    pub sample_window_seconds: f64,
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn classify_snapshots(
    pane: &str,
    raw_samples: &[ActivitySample],
    window_ms: u64,
) -> ActivityResult {
    let normalized = raw_samples
        .iter()
        .map(|sample| normalize_snapshot(&sample.text))
        .collect::<Vec<_>>();
    let mut changed_indexes = BTreeSet::new();
    let mut last_change_at = None;
    for index in 1..normalized.len() {
        if normalized[index] != normalized[index - 1] {
            changed_indexes.insert(index - 1);
            changed_indexes.insert(index);
            last_change_at = raw_samples.get(index).map(|sample| sample.at_ms);
        }
    }
    let end = raw_samples.last().map_or(0, |sample| sample.at_ms);
    let state = if changed_indexes.is_empty() {
        if raw_samples
            .last()
            .is_some_and(|sample| is_stuck_snapshot(&sample.text))
        {
            ActivityState::Stuck
        } else {
            ActivityState::Idle
        }
    } else {
        ActivityState::Busy
    };
    let sample_window_seconds = round_seconds(window_ms as f64 / 1_000.0);
    let last_change_ago_seconds = last_change_at.map_or(sample_window_seconds, |changed| {
        round_seconds(end.saturating_sub(changed) as f64 / 1_000.0)
    });
    ActivityResult {
        pane: pane.to_owned(),
        state,
        confidence: confidence_for(raw_samples.len()),
        samples: u32::try_from(raw_samples.len()).unwrap_or(u32::MAX),
        diff_samples: u32::try_from(changed_indexes.len()).unwrap_or(u32::MAX),
        last_change_ago_seconds,
        sample_window_seconds,
    }
}

#[must_use]
pub fn normalize_snapshot(input: &str) -> String {
    strip_terminal_sequences(input)
        .replace('\r', "\n")
        .split('\n')
        .map(|line| line.trim_end_matches(is_ecmascript_whitespace))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_matches(is_ecmascript_whitespace)
        .to_owned()
}

#[must_use]
pub fn is_stuck_snapshot(input: &str) -> bool {
    let normalized = normalize_snapshot(input);
    if normalized
        .lines()
        .map(|line| line.trim_matches(is_ecmascript_whitespace))
        .filter(|line| !line.is_empty())
        .rev()
        .take(10)
        .any(is_prompt_line)
    {
        return true;
    }
    let lower = normalized.to_ascii_lowercase();
    let suffix = lower.strip_suffix('?').unwrap_or(&lower);
    [
        "type a message",
        "send a message",
        "ask codex",
        "ask claude",
        "what can i help with",
    ]
    .iter()
    .any(|phrase| suffix.ends_with(phrase))
}

fn is_prompt_line(line: &str) -> bool {
    let mut chars = line.chars();
    if !chars
        .next()
        .is_some_and(|first| matches!(first, '>' | '$' | '#' | '❯' | '›' | 'λ'))
    {
        return false;
    }
    matches!(
        chars.as_str().trim_matches(is_ecmascript_whitespace),
        "" | "▌" | "█" | "_"
    )
}

fn is_ecmascript_whitespace(ch: char) -> bool {
    (ch.is_whitespace() && ch != '\u{85}') || ch == '\u{feff}'
}

fn strip_terminal_sequences(input: &str) -> String {
    let without_osc = strip_matching_sequences(input, b"]", osc_end, true);
    let without_strings = strip_matching_sequences(&without_osc, b"P^_", string_control_end, true);
    let without_csi = strip_matching_sequences(&without_strings, b"[", csi_end, false);
    strip_matching_sequences(&without_csi, b"()", charset_end, false)
}

fn strip_matching_sequences(
    input: &str,
    starts: &[u8],
    sequence_end: fn(&[u8], usize) -> Option<usize>,
    stop_on_unterminated: bool,
) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copied_from = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b
            && bytes
                .get(index + 1)
                .is_some_and(|next| starts.contains(next))
        {
            if let Some(end) = sequence_end(bytes, index) {
                out.push_str(&input[copied_from..index]);
                copied_from = end;
                index = end;
                continue;
            }
            if stop_on_unterminated {
                break;
            }
        }
        index += 1;
    }
    out.push_str(&input[copied_from..]);
    out
}

fn osc_end(bytes: &[u8], start: usize) -> Option<usize> {
    let index = start + 2;
    if let Some(offset) = bytes[index..].iter().position(|byte| *byte == 0x07) {
        return Some(index + offset + 1);
    }
    string_terminators(bytes, index)
        .next_back()
        .map(|at| at + 2)
}

fn string_control_end(bytes: &[u8], start: usize) -> Option<usize> {
    string_terminators(bytes, start + 2).next().map(|at| at + 2)
}

fn string_terminators(bytes: &[u8], start: usize) -> impl DoubleEndedIterator<Item = usize> + '_ {
    (start..bytes.len().saturating_sub(1))
        .filter(|at| bytes[*at] == 0x1b && bytes.get(*at + 1) == Some(&b'\\'))
}

fn csi_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 2;
    while bytes
        .get(index)
        .is_some_and(|byte| (0x30..=0x3f).contains(byte))
    {
        index += 1;
    }
    while bytes
        .get(index)
        .is_some_and(|byte| (0x20..=0x2f).contains(byte))
    {
        index += 1;
    }
    bytes
        .get(index)
        .filter(|byte| (0x40..=0x7e).contains(*byte))
        .map(|_| index + 1)
}

fn charset_end(bytes: &[u8], start: usize) -> Option<usize> {
    bytes
        .get(start + 2)
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(|_| start + 3)
}

const fn confidence_for(samples: usize) -> ActivityConfidence {
    if samples >= 3 {
        ActivityConfidence::High
    } else if samples == 2 {
        ActivityConfidence::Medium
    } else {
        ActivityConfidence::Low
    }
}

fn round_seconds(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

pub const ACTIVITY_CLASSIFICATION_FIXTURES_JSON: &str =
    include_str!("../tests/fixtures/activity-classification.fixtures.json");
