use super::super::super::sequence::{Process, Sequence, phred};
use super::super::config::{Limits, LimitsExt};
use super::super::report::charts::CHART_WIDTH;
use super::super::report::charts::line_graph::{LineGraphData, render_line_graph};
use super::QCModule;
use std::io;

/// Maximum raw ASCII average quality score we can track.
/// Quality chars are in range 0-127; the average of any sequence of such chars
/// fits in the same range. 128 slots covers all possible values.
const MAX_QUALITY_SCORE: usize = 128;

#[derive(Clone)]
pub struct QualityScores {
    // JAVA COMPAT: Indexed by raw ASCII average quality (before offset subtraction),
    // computed using integer arithmetic: sum of quality chars / length.
    // Using a fixed array instead of HashMap eliminates hashing on every read.
    average_score_counts: [u64; MAX_QUALITY_SCORE],
    has_data: bool,
    lowest_char: u16,
    data: Option<DistributionData>,
}

impl Default for QualityScores {
    fn default() -> Self {
        Self {
            average_score_counts: [0u64; MAX_QUALITY_SCORE],
            has_data: false,
            lowest_char: phred::NO_QUALITY_SEEN,
            data: None,
        }
    }
}

impl std::ops::Add for QualityScores {
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
                lowest_char: self.lowest_char.min(rhs.lowest_char),
                data: None,
            }
        }
    }
}

impl std::fmt::Display for QualityScores {
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

impl QCModule for QualityScores {
    fn name(&self) -> &str {
        "Per sequence quality scores"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        if !self.has_data {
            return Ok(());
        }

        let encoding =
            phred::resolve(None, self.lowest_char).unwrap_or(phred::PhredEncoding::SANGER);

        // Find the range of scores with non-zero counts
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

        let range_start = range_start.unwrap() as i32;
        let range_end = range_end as i32;

        // Distribution runs from lowest to highest raw score
        let len = (1 + range_end - range_start) as usize;

        let mut quality_distribution = vec![0.0f64; len];
        let mut x_categories = Vec::with_capacity(len);

        // Build distribution and x_categories arrays in parallel
        for (i, qd) in quality_distribution.iter_mut().enumerate() {
            x_categories.push(range_start + i as i32 - encoding.offset as i32);
            let key = (range_start + i as i32) as usize;
            *qd = self.average_score_counts[key] as f64;
        }

        // Find most frequent score
        let mut max_count = 0.0;
        let mut most_frequent_score = 0;
        // index needed for both quality_distribution and x_categories
        for (&qd, &xc) in quality_distribution.iter().zip(x_categories.iter()) {
            if qd > max_count {
                max_count = qd;
                most_frequent_score = xc;
            }
        }

        self.data = Some(DistributionData {
            quality_distribution,
            x_categories,
            most_frequent_score,
            error_threshold: limits.threshold("quality_sequence\terror", 20.0),
            warn_threshold: limits.threshold("quality_sequence\twarn", 27.0),
            ignored: limits.is_ignored("quality_sequence"),
        });

        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        self.data.as_ref().unwrap().ignored || !self.has_data
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| (data.most_frequent_score as f64) <= data.error_threshold)
    }

    fn raises_warning(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| (data.most_frequent_score as f64) <= data.warn_threshold)
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        let data = self.data.clone()?;
        let max_count = data
            .quality_distribution
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max);
        // Java passes raw max to LineGraph (no ceil rounding)
        let max_y = max_count;

        let x_categories: Vec<String> =
            data.x_categories.iter().map(|v| format!("{}", v)).collect();

        let svg = render_line_graph(&LineGraphData {
            width: CHART_WIDTH,
            data: vec![data.quality_distribution],
            min_y: 0.0,
            max_y,
            // These labels match the Java constructor call
            x_label: "Mean Sequence Quality (Phred Score)".to_string(),
            series_names: vec!["Average Quality per read".to_string()],
            x_categories,
            title: "Quality score distribution over all sequences".to_string(),
        });

        Some(("Per Sequence quality graph", svg))
    }
}

#[derive(Clone, Default)]
pub struct PerSequenceQualityScores {
    scores: QualityScores,
}

impl Process for PerSequenceQualityScores {
    type Value = QualityScores;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let qual = &sequence.quality;

        // JAVA COMPAT: Average quality computed using integer arithmetic on raw ASCII values.
        // sum of quality chars (as int) / length (integer division), stored as raw value.
        let mut average_quality: i32 = 0;

        for &q in qual.iter() {
            self.scores.lowest_char = self.scores.lowest_char.min(q as u16);
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

#[derive(Clone)]
struct DistributionData {
    quality_distribution: Vec<f64>,
    x_categories: Vec<i32>,
    most_frequent_score: i32,
    error_threshold: f64,
    warn_threshold: f64,
    ignored: bool,
}
