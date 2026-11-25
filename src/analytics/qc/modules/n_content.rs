use super::super::super::sequence::{BASE_INDEX, IDX_N, Process, Sequence};
use super::super::base_group::BaseGroup;
use super::super::config::{Limits, LimitsExt};
use super::super::report::charts::line_graph::{LineGraphData, render_line_graph};
use super::super::report::charts::scaled_chart_width;
use super::QCModule;
use std::io;

#[derive(Clone, Default)]
pub struct NContent {
    n_counts: Vec<u64>,
    not_n_counts: Vec<u64>,
    data: Option<NContentData>,
}

impl std::ops::Add for NContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer_n_counts, shorter_n_counts, mut longer_not_n_counts, shorter_not_n_counts) =
            if self.n_counts.len() > rhs.n_counts.len() {
                (
                    self.n_counts,
                    rhs.n_counts,
                    self.not_n_counts,
                    rhs.not_n_counts,
                )
            } else {
                (
                    rhs.n_counts,
                    self.n_counts,
                    rhs.not_n_counts,
                    self.not_n_counts,
                )
            };
        for i in 0..longer_n_counts.len() {
            longer_n_counts[i] += shorter_n_counts[i];
            longer_not_n_counts[i] += shorter_not_n_counts[i];
        }
        Self {
            n_counts: longer_n_counts,
            not_n_counts: longer_not_n_counts,
            data: None,
        }
    }
}

impl std::fmt::Display for NContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.n_counts.len())
    }
}

impl QCModule for NContent {
    fn name(&self) -> &str {
        "Per base N content"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        let groups = BaseGroup::make_base_groups(self.n_counts.len(), false, false);

        let mut x_categories = Vec::with_capacity(groups.len());
        let mut percentages = vec![0.0f64; groups.len()];

        for (i, group) in groups.iter().enumerate() {
            x_categories.push(group.label());

            let mut n_count: u64 = 0;
            let mut total: u64 = 0;

            // Java iterates `for (int bp=groups[i].lowerCount()-1;bp<groups[i].upperCount();bp++)`
            // Our lower_count/upper_count are 0-based.
            for bp in group.lower_count..=group.upper_count {
                n_count += self.n_counts[bp];
                total += self.n_counts[bp];
                total += self.not_n_counts[bp];
            }

            // percentages[i] = 100 * (nCount / (double)total)
            percentages[i] = 100.0 * (n_count as f64 / total as f64);
        }

        self.data = Some(NContentData {
            x_categories,
            percentages,
            error_threshold: limits.threshold("n_content\terror", 20.0),
            warn_threshold: limits.threshold("n_content\twarn", 5.0),
            ignored: limits.is_ignored("n_content"),
        });

        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        self.data.as_ref().unwrap().ignored
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        let data = self.data.as_ref().unwrap();
        data.percentages.iter().any(|&p| p > data.error_threshold)
    }

    fn raises_warning(&self) -> bool {
        let data = self.data.as_ref().unwrap();
        data.percentages.iter().any(|&p| p > data.warn_threshold)
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        let data = self.data.clone().unwrap();

        // minY=0, maxY=100 for percentage, matching Java's constructor
        let svg = render_line_graph(&LineGraphData {
            width: scaled_chart_width(data.x_categories.len()),
            data: vec![data.percentages],
            min_y: 0.0,
            max_y: 100.0,
            x_label: "Position in read (bp)".to_string(),
            series_names: vec!["%N".to_string()],
            x_categories: data.x_categories,
            title: "N content across all bases".to_string(),
        });

        Some(("N content graph", svg))
    }
}

#[derive(Clone, Default)]
pub struct PerBaseNContent {
    content: NContent,
}

impl Process for PerBaseNContent {
    type Value = NContent;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq = &sequence.sequence;

        // Grow arrays if needed
        if self.content.n_counts.len() < seq.len() {
            self.content.n_counts.resize(seq.len(), 0);
            self.content.not_n_counts.resize(seq.len(), 0);
        }

        // Use lookup table to classify each byte without a multi-way match
        for (i, &b) in seq.iter().enumerate() {
            if BASE_INDEX[b as usize] as usize == IDX_N {
                self.content.n_counts[i] += 1;
            } else {
                self.content.not_n_counts[i] += 1;
            }
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.content.to_owned()
    }
}

#[derive(Clone)]
struct NContentData {
    x_categories: Vec<String>,
    percentages: Vec<f64>,
    error_threshold: f64,
    warn_threshold: f64,
    ignored: bool,
}
