use super::super::super::sequence::{Process, Sequence};
use super::super::config::{Limits, LimitsExt};
use super::super::report::charts::line_graph::{LineGraphData, render_line_graph};
use super::super::report::charts::scaled_chart_width;
use super::QCModule;
use std::io;

#[derive(Clone, Default)]
pub struct LengthDistribution {
    /// length_counts[i] = number of sequences with length i
    length_counts: Vec<u64>,
    distribution: Option<ComputedDistribution>,
}

impl std::ops::Add for LengthDistribution {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.length_counts.len() > rhs.length_counts.len() {
            (self.length_counts, rhs.length_counts)
        } else {
            (rhs.length_counts, self.length_counts)
        };

        for (x, y) in longer.iter_mut().zip(shorter.iter()) {
            *x += *y;
        }
        Self {
            length_counts: longer,
            distribution: None,
        }
    }
}

impl std::fmt::Display for LengthDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.length_counts.len())
    }
}

impl LengthDistribution {
    /// Find interval and starting point for binning sequence lengths.
    /// Replicates getSizeDistribution() from SequenceLengthDistribution.java.
    fn get_size_distribution(min: usize, max: usize, nogroup: bool) -> (usize, usize) {
        // If nogroup is set, don't bin
        if nogroup {
            return (min, 1);
        }

        // Find the smallest interval from [1,2,5]*base that gives <=50 bins
        let mut base: usize = 1;

        // The Java code starts with base=1 and has `while (base > (max-min)) base /= 10`
        // which is a no-op when base starts at 1 (since 1 is always <= max-min for any valid range).
        // We skip this loop as it has no effect in practice.

        let divisions = [1, 2, 5];
        let interval;

        'outer: loop {
            for &d in &divisions {
                let tester = base * d;
                if (max - min) / tester <= 50 {
                    interval = tester;
                    break 'outer;
                }
            }
            base *= 10;
        }

        // Calculate starting value aligned to interval boundary
        let basic_division = min / interval;
        let starting = basic_division * interval;

        (starting, interval)
    }
}

impl QCModule for LengthDistribution {
    fn name(&self) -> &str {
        "Sequence Length Distribution"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        let mut max_len: usize = 0;
        let mut min_len: Option<usize> = None;

        // Find min and max lengths
        for i in 0..self.length_counts.len() {
            if self.length_counts[i] > 0 {
                if min_len.is_none() {
                    min_len = Some(i);
                }
                max_len = i;
            }
        }

        // Default min to 0 if no sequences
        let mut min_len = min_len.unwrap_or(0);

        // Add one extra category on either side
        min_len = min_len.saturating_sub(1);
        max_len += 1;

        let (starting, interval) = Self::get_size_distribution(min_len, max_len, false);

        // Count how many categories we need
        let mut categories = 0;
        let mut current_value = starting;
        while current_value <= max_len {
            categories += 1;
            current_value += interval;
        }

        let mut graph_counts = vec![0.0f64; categories];
        let mut x_categories = Vec::with_capacity(categories);

        // i needed to compute bin boundaries
        for (i, gc) in graph_counts.iter_mut().enumerate() {
            let min_value = starting + (interval * i);
            let mut max_value = (starting + (interval * (i + 1))) - 1;

            // Clamp max_value to maxLen
            if max_value > max_len {
                max_value = max_len;
            }

            // Sum counts in this bin
            for bp in min_value..=max_value {
                if bp < self.length_counts.len() {
                    *gc += self.length_counts[bp] as f64;
                }
            }

            // Label format depends on interval
            if interval == 1 {
                x_categories.push(format!("{}", min_value));
            } else {
                x_categories.push(format!("{}-{}", min_value, max_value));
            }
        }

        self.distribution = Some(ComputedDistribution {
            graph_counts,
            x_categories,
            error_threshold: limits.threshold("sequence_length\terror", 1.0),
            warn_threshold: limits.threshold("sequence_length\twarn", 1.0),
            ignored: limits.is_ignored("sequence_length"),
        });

        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        self.distribution.as_ref().unwrap().ignored
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        let threshold = self.distribution.as_ref().unwrap().error_threshold;
        if threshold == 0.0 {
            return false;
        }

        // Error if there are sequences of length 0
        if !self.length_counts.is_empty() && self.length_counts[0] > 0 {
            return true;
        }
        false
    }

    fn raises_warning(&self) -> bool {
        let threshold = self.distribution.as_ref().unwrap().warn_threshold;
        if threshold == 0.0 {
            return false;
        }

        // Warn if there are sequences of different lengths
        let mut seen_length = false;
        for &count in &self.length_counts {
            if count > 0 {
                if seen_length {
                    return true;
                }
                seen_length = true;
            }
        }
        false
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        let computed = self.distribution.clone().unwrap();
        let max_val = computed
            .graph_counts
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max);
        // Java passes raw max to LineGraph (no ceil rounding)
        let max_y = max_val;

        // Matches Java constructor call
        let svg = render_line_graph(&LineGraphData {
            width: scaled_chart_width(computed.graph_counts.len()),
            data: vec![computed.graph_counts],
            min_y: 0.0,
            max_y,
            x_label: "Sequence Length (bp)".to_string(),
            series_names: vec!["Sequence Length".to_string()],
            x_categories: computed.x_categories,
            title: "Distribution of sequence lengths over all sequences".to_string(),
        });

        Some(("Sequence length distribution", svg))
    }
}

#[derive(Clone, Default)]
pub struct SequenceLengthDistribution {
    distribution: LengthDistribution,
}

impl Process for SequenceLengthDistribution {
    type Value = LengthDistribution;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq_len = sequence.sequence.len();

        // Array is extended to seqLen+2 to match Java's `seqLen+2 > lengthCounts.length`
        if seq_len + 2 > self.distribution.length_counts.len() {
            self.distribution.length_counts.resize(seq_len + 2, 0);
        }

        self.distribution.length_counts[seq_len] += 1;
    }

    fn ref_into(&self) -> Self::Value {
        self.distribution.to_owned()
    }
}

#[derive(Clone)]
struct ComputedDistribution {
    graph_counts: Vec<f64>,
    x_categories: Vec<String>,
    error_threshold: f64,
    warn_threshold: f64,
    ignored: bool,
}
