use super::super::super::sequence::{Process, Sequence, phred};
use super::super::base_group::BaseGroup;
use super::super::config::{Limits, LimitsExt};
use super::super::quality_count::{QualityCount, calculate_offsets};
use super::super::report::charts::quality_boxplot::{QualityBoxPlotData, render_quality_boxplot};
use super::super::report::charts::scaled_chart_width;
use super::QCModule;
use std::io;

#[derive(Clone, Default)]
pub struct PerBaseQualityCounts {
    quality_counts: Vec<QualityCount>,
    calc_data: Option<CalculatedData>,
}

impl std::ops::Add for PerBaseQualityCounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.quality_counts.len() > rhs.quality_counts.len() {
            (self.quality_counts, rhs.quality_counts)
        } else {
            (rhs.quality_counts, self.quality_counts)
        };
        for i in 0..shorter.len() {
            longer[i] += shorter[i].clone();
        }
        Self {
            quality_counts: longer,
            calc_data: None,
        }
    }
}

impl std::fmt::Display for PerBaseQualityCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.quality_counts.is_empty() {
                0
            } else {
                self.quality_counts[0].get_total_count()
            }
        )
    }
}

impl PerBaseQualityCounts {
    /// Replicates `getPercentile(int minbp, int maxbp, int offset, int percentile)`.
    /// Only includes positions with >100 total counts.
    /// minbp and maxbp are 0-based inclusive indices into quality_counts.
    fn get_percentile(&self, min_base: usize, max_base: usize, offset: u8, percentile: u8) -> f64 {
        let mut count = 0;
        let mut total = 0.0;

        // Java loop is `for (int i=minbp-1;i<maxbp;i++)` where minbp
        // is 1-based. Our min_base is 0-based, so we iterate min_base..=max_base.
        for i in min_base..=max_base {
            // Only include positions with >100 counts for percentile calculation
            if self.quality_counts[i].get_total_count() > 100 {
                count += 1;
                total += self.quality_counts[i].get_percentile(offset, percentile);
            }
        }

        if count > 0 {
            total / count as f64
        } else {
            f64::NAN
        }
    }

    /// Replicates `getMean(int minbp, int maxbp, int offset)`.
    /// Only includes positions with >0 total counts.
    fn get_mean(&self, min_base: usize, max_base: usize, offset: u8) -> f64 {
        let mut count = 0;
        let mut total = 0.0;

        for i in min_base..=max_base {
            // getMean includes positions with >0 counts (not >100 like percentile)
            if self.quality_counts[i].get_total_count() > 0 {
                count += 1;
                total += self.quality_counts[i].get_mean(offset);
            }
        }

        if count > 0 {
            total / count as f64
        } else {
            // Java returns 0 when count is 0 for mean (not NaN)
            0.0
        }
    }
}

impl QCModule for PerBaseQualityCounts {
    fn name(&self) -> &str {
        "Per base sequence quality"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        let (min_char, _max_char) = calculate_offsets(&self.quality_counts);
        // If no quality data, default to the Sanger offset.
        let offset = phred::resolve(None, min_char as u16)
            .map(|e| e.offset)
            .unwrap_or(phred::PhredEncoding::SANGER.offset);

        let groups = BaseGroup::make_base_groups(self.quality_counts.len(), false, false);

        let mut means = vec![0.0f64; groups.len()];
        let mut medians = vec![0.0f64; groups.len()];
        let mut lower_quartile = vec![0.0f64; groups.len()];
        let mut upper_quartile = vec![0.0f64; groups.len()];
        let mut lowest = vec![0.0f64; groups.len()];
        let mut highest = vec![0.0f64; groups.len()];
        let mut x_labels = Vec::with_capacity(groups.len());

        for (i, group) in groups.iter().enumerate() {
            x_labels.push(group.label());
            // Java uses 1-based lowerCount/upperCount; our BaseGroup
            // stores 0-based lower_count/upper_count.
            let min_base = group.lower_count;
            let max_base = group.upper_count;
            lowest[i] = self.get_percentile(min_base, max_base, offset, 10);
            highest[i] = self.get_percentile(min_base, max_base, offset, 90);
            means[i] = self.get_mean(min_base, max_base, offset);
            medians[i] = self.get_percentile(min_base, max_base, offset, 50);
            lower_quartile[i] = self.get_percentile(min_base, max_base, offset, 25);
            upper_quartile[i] = self.get_percentile(min_base, max_base, offset, 75);
        }

        let lq_error = limits.threshold("quality_base_lower\terror", 5.0);
        let median_error = limits.threshold("quality_base_median\terror", 20.0);
        let lq_warn = limits.threshold("quality_base_lower\twarn", 10.0);
        let median_warn = limits.threshold("quality_base_median\twarn", 25.0);

        self.calc_data = Some(CalculatedData {
            means,
            medians,
            lower_quartile,
            upper_quartile,
            lowest,
            highest,
            x_labels,
            lq_error,
            median_error,
            lq_warn,
            median_warn,
            ignored: limits.is_ignored("quality_base"),
        });

        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        // Ignore if configured to ignore or no quality data
        self.calc_data.as_ref().unwrap().ignored || self.quality_counts.is_empty()
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        let data = self.calc_data.as_ref().unwrap();
        for i in 0..data.lower_quartile.len() {
            if data.lower_quartile[i].is_nan() {
                // Skip groups without enough data
                continue;
            }
            if data.lower_quartile[i] < data.lq_error || data.medians[i] < data.median_error {
                return true;
            }
        }
        false
    }

    fn raises_warning(&self) -> bool {
        let data = self.calc_data.as_ref().unwrap();
        for i in 0..data.lower_quartile.len() {
            if data.lower_quartile[i].is_nan() {
                continue;
            }
            if data.lower_quartile[i] < data.lq_warn || data.medians[i] < data.median_warn {
                return true;
            }
        }
        false
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        let data = self.calc_data.clone().unwrap();
        let (min_char, max_char) = calculate_offsets(&self.quality_counts);
        // If no quality data, default to Sanger.
        let encoding =
            phred::resolve(None, min_char as u16).unwrap_or(phred::PhredEncoding::SANGER);
        let (offset, encoding_name) = (encoding.offset, encoding.name);

        // The chart title includes the encoding scheme name
        let title = format!(
            "Quality scores across all bases ({} encoding)",
            encoding_name
        );

        // Java computes: high = maxChar - offset; if (high < 35) high = 35;
        // This determines the Y-axis range from the encoding, not from the data.
        // Java passes high directly as maxY (no rounding).
        let mut max_y = (max_char as i32 - offset as i32).max(0) as f64;
        if max_y < 35.0 {
            max_y = 35.0;
        }
        let y_interval = 2.0;
        let min_y = 0.0;

        let svg = render_quality_boxplot(&QualityBoxPlotData {
            width: scaled_chart_width(data.x_labels.len()),
            means: data.means,
            medians: data.medians,
            lower_quartile: data.lower_quartile,
            upper_quartile: data.upper_quartile,
            lowest: data.lowest,
            highest: data.highest,
            min_y,
            max_y,
            y_interval,
            x_labels: data.x_labels,
            title,
        });

        Some(("Per base quality graph", svg))
    }
}

#[derive(Clone, Default)]
pub struct PerBaseSequenceQuality {
    counts: PerBaseQualityCounts,
}

impl Process for PerBaseSequenceQuality {
    type Value = PerBaseQualityCounts;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let qual = &sequence.quality;

        // Grow the quality_counts array if needed
        if self.counts.quality_counts.len() < qual.len() {
            self.counts
                .quality_counts
                .resize_with(qual.len(), QualityCount::default);
        }

        for (i, &q) in qual.iter().enumerate() {
            self.counts.quality_counts[i].add_value(q);
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.counts.to_owned()
    }
}

/// Internal struct holding calculated per-base quality data.
#[derive(Clone)]
struct CalculatedData {
    means: Vec<f64>,
    medians: Vec<f64>,
    lower_quartile: Vec<f64>,
    upper_quartile: Vec<f64>,
    lowest: Vec<f64>,
    highest: Vec<f64>,
    x_labels: Vec<String>,
    lq_error: f64,
    median_error: f64,
    lq_warn: f64,
    median_warn: f64,
    ignored: bool,
}
