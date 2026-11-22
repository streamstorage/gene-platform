use super::super::super::sequence::{Process, Sequence};

/// Maximum raw ASCII average quality score we can track.
/// Quality chars are in range 0-127; the average of any sequence of such chars
/// fits in the same range. 128 slots covers all possible values.
const MAX_QUALITY_SCORE: usize = 128;

#[derive(Clone)]
pub struct PerSequenceQualityScores {
    // JAVA COMPAT: Indexed by raw ASCII average quality (before offset subtraction),
    // computed using integer arithmetic: sum of quality chars / length.
    // Using a fixed array instead of HashMap eliminates hashing on every read.
    average_score_counts: [u64; MAX_QUALITY_SCORE],
    has_data: bool,
}

impl Default for PerSequenceQualityScores {
    fn default() -> Self {
        Self {
            average_score_counts: [0u64; MAX_QUALITY_SCORE],
            has_data: false,
        }
    }
}

impl std::ops::Add for PerSequenceQualityScores {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if !self.has_data {
            rhs
        } else if !rhs.has_data {
            self
        } else {
            let average_score_counts =
                std::array::from_fn(|i| self.average_score_counts[i] + rhs.average_score_counts[i]);
            Self {
                average_score_counts,
                has_data: true,
            }
        }
    }
}

impl std::fmt::Display for PerSequenceQualityScores {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.has_data {
            write!(f, "{}-{}", 0, 0)
        } else {
            let mut range_start: Option<usize> = None;
            let mut range_end: usize = 0;
            for i in 0..MAX_QUALITY_SCORE {
                if self.average_score_counts[i] > 0 {
                    if range_start.is_none() {
                        range_start = Some(i);
                    }
                    range_end = i;
                }
            }
            write!(f, "{}-{}", range_start.unwrap_or(0), range_end)
        }
    }
}

#[derive(Clone, Default)]
pub struct PerSequenceQuality {
    scores: PerSequenceQualityScores,
}

impl Process for PerSequenceQuality {
    type Value = PerSequenceQualityScores;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let qual = &sequence.quality;

        // JAVA COMPAT: Average quality computed using integer arithmetic on raw ASCII values.
        // sum of quality chars (as int) / length (integer division), stored as raw value.
        let mut average_quality: i32 = 0;

        for &q in qual.iter() {
            average_quality += q as i32;
        }

        if !qual.is_empty() {
            // JAVA COMPAT: Integer division truncates towards zero, matching Java's `/`
            average_quality /= qual.len() as i32;

            // Clamp rather than index blindly. Java throws
            // ArrayIndexOutOfBoundsException and kills the run; a file whose
            // quality line holds bytes at or above MAX_QUALITY_SCORE (a
            // mis-encoded or non-ASCII line) would otherwise panic here, and
            // under the parallel pipeline take a worker thread with it. Same
            // treatment as QualityCount::add_value: the value is wrong for that
            // read, but the rest of the file still gets analysed. Valid input
            // never reaches this, so byte-identical output is unaffected.
            let slot = (average_quality as usize).min(MAX_QUALITY_SCORE - 1);
            if slot != average_quality as usize {
                // TODO: support logger
                log::warn!(
                    "Warning: mean quality {} exceeds maximum {}; clamping",
                    average_quality,
                    MAX_QUALITY_SCORE - 1
                );
            }
            self.scores.average_score_counts[slot] += 1;
            self.scores.has_data = true;
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.scores.to_owned()
    }
}
