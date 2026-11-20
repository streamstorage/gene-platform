// Basic Statistics module
// Corresponds to Modules/BasicStats.java

use super::super::sequence::phred;
use super::super::sequence::{IDX_A, IDX_C, IDX_G, IDX_T, Process, Sequence, count_acgt};

/// The raw accumulated counters behind the Basic Statistics table.
///
/// Kept separate from [`BasicStats`] so that exactly the same numbers can be
/// formatted for the text/HTML report and for the live terminal table, with
/// [`BasicStatsCounters::rows`] as the single source of truth for both — which
/// is also why the counters themselves are private: everything outside this
/// module wants the formatted rows, not the raw tallies. It is `Copy` so a
/// consistent snapshot can be handed to the progress display without holding a
/// lock while rendering.
#[derive(Debug, Clone)]
pub struct BasicStatsCounters {
    actual_count: u64,
    min_length: usize,
    max_length: usize,
    total_bases: u64,
    g_count: u64,
    c_count: u64,
    a_count: u64,
    t_count: u64,
    lowest_char: u16,
    length_counts: Vec<u64>,
}

/// Spelled out rather than derived: `lowest_char` starts at a sentinel above
/// the range and is lowered, so a derived all-zeroes default would be a state
/// the counters can never legitimately reach — and one whose `rows()` reports
/// a bogus encoding. This is the only constructor, so there is nowhere for that
/// state to come from.
impl Default for BasicStatsCounters {
    fn default() -> Self {
        BasicStatsCounters {
            actual_count: 0,
            min_length: usize::MAX,
            max_length: 0,
            total_bases: 0,
            g_count: 0,
            c_count: 0,
            a_count: 0,
            t_count: 0,
            lowest_char: phred::NO_QUALITY_SEEN,
            length_counts: Vec::new(),
        }
    }
}

impl BasicStatsCounters {
    /// The measures reported, in report order, minus the leading "Filename"
    /// row. [`Self::rows`] returns a value for each of these, in this order.
    pub const MEASURES: [&'static str; 7] = [
        "Encoding",
        "Total Sequences",
        "Total Bases",
        "Sequence length",
        "Mean Length",
        "Median Length",
        "%GC",
    ];

    /// The Basic Statistics rows, minus the leading "Filename" row, in report
    /// order. Both the text report and the live progress table render from
    /// this, so the values on screen always agree with the values on disk.
    pub fn rows(&self) -> Vec<(&'static str, String)> {
        // The report propagates a resolve failure before rendering rows (see
        // write_text_report), so "Unknown" only ever reaches the live table.
        let encoding = phred::resolve(None, self.lowest_char)
            .map(|e| e.name.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let sequence_length = if self.min_length == self.max_length {
            self.min_length.to_string()
        } else {
            format!("{}-{}", self.min_length, self.max_length)
        };

        // JAVA COMPAT: integer division
        let mean_length = self.total_bases.checked_div(self.actual_count).unwrap_or(0);

        // JAVA COMPAT: Integer division: ((gCount+cCount)*100)/(aCount+tCount+gCount+cCount)
        let total = self.a_count + self.t_count + self.g_count + self.c_count;
        let gc = ((self.g_count + self.c_count) * 100)
            .checked_div(total)
            .unwrap_or(0);

        let values = [
            encoding,
            self.actual_count.to_string(),
            format_length(self.total_bases),
            sequence_length,
            mean_length.to_string(),
            median_length(&self.length_counts).to_string(),
            gc.to_string(),
        ];
        Self::MEASURES.into_iter().zip(values).collect()
    }
}

/// Format a base count into a human-readable string.
///
/// Replicates `BasicStats.formatLength(long)` exactly, including
/// its custom decimal truncation logic (keeps at most 1 non-zero decimal digit).
pub fn format_length(original_length: u64) -> String {
    let mut length = original_length as f64;

    let unit = if length >= 1_000_000_000.0 {
        length /= 1_000_000_000.0;
        " Gbp"
    } else if length >= 1_000_000.0 {
        length /= 1_000_000.0;
        " Mbp"
    } else if length >= 1_000.0 {
        length /= 1_000.0;
        " kbp"
    } else {
        " bp"
    };

    // JAVA COMPAT: Java builds `"" + length` which calls Double.toString(),
    // then applies a custom truncation: find the dot, keep one more char if
    // it's non-zero, otherwise drop the dot.
    let raw = format!("{}", length);
    let chars: Vec<char> = raw.chars().collect();

    let mut last_index = 0;

    // Find the dot
    for (i, &ch) in chars.iter().enumerate() {
        last_index = i;
        if ch == '.' {
            break;
        }
    }

    // Keep next char if non-zero
    if last_index + 1 < chars.len() && chars[last_index + 1] != '0' {
        last_index += 1;
    } else if last_index > 0 && chars[last_index] == '.' {
        // Lose the dot if it would be the last character
        last_index -= 1;
    }

    let truncated: String = chars[..=last_index].iter().collect();
    format!("{}{}", truncated, unit)
}

/// Replicates `SequenceLengthDistribution.medianLength()`: the upper of the
/// two central values for an even read count, 0 when there are no reads.
pub fn median_length(length_counts: &[u64]) -> usize {
    let rank50 = length_counts.iter().sum::<u64>() / 2;
    let mut running = 0;
    for (len, &count) in length_counts.iter().enumerate() {
        running += count;
        if running > rank50 {
            return len;
        }
    }
    0
}

impl std::ops::Add for BasicStatsCounters {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let (mut longer, shorter) = if self.length_counts.len() > other.length_counts.len() {
            (self.length_counts, other.length_counts)
        } else {
            (other.length_counts, self.length_counts)
        };
        for i in 0..shorter.len() {
            longer[i] += shorter[i];
        }

        Self {
            actual_count: self.actual_count + other.actual_count,
            min_length: self.min_length.min(other.min_length),
            max_length: self.max_length.max(other.max_length),
            total_bases: self.total_bases + other.total_bases,
            g_count: self.g_count + other.g_count,
            c_count: self.c_count + other.c_count,
            a_count: self.a_count + other.a_count,
            t_count: self.t_count + other.t_count,
            lowest_char: self.lowest_char.min(other.lowest_char),
            length_counts: longer,
        }
    }
}

impl std::fmt::Display for BasicStatsCounters {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{:?}", self.rows())
    }
}

#[derive(Clone, Default)]
pub struct BasicStats {
    counters: BasicStatsCounters,
}

impl Process for BasicStats {
    type Value = BasicStatsCounters;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn none(&self) -> Self::Value {
        BasicStatsCounters::default()
    }

    fn process(&mut self, sequence: &Sequence) {
        let c = &mut self.counters;
        c.actual_count += 1;
        c.total_bases += sequence.sequence.len() as u64;

        let len = sequence.sequence.len();
        if c.length_counts.len() <= len {
            c.length_counts.resize(len + 1, 0);
        }
        c.length_counts[len] += 1;

        c.min_length = c.min_length.min(len);
        c.max_length = c.max_length.max(len);

        let counts = count_acgt(&sequence.sequence);
        c.a_count += counts[IDX_A];
        c.c_count += counts[IDX_C];
        c.g_count += counts[IDX_G];
        c.t_count += counts[IDX_T];

        let lowest_char = sequence
            .quality
            .iter()
            .min()
            .map_or_else(|| phred::NO_QUALITY_SEEN, |v| *v as u16);

        c.lowest_char = c.lowest_char.min(lowest_char as u16);
    }

    fn mut_into(&mut self) -> Self::Value {
        self.counters.to_owned()
    }
}
