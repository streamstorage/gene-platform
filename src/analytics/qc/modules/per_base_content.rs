use super::super::super::sequence::{BASE_INDEX, IDX_A, IDX_C, IDX_G, IDX_T, Process, Sequence};
use super::super::base_group::BaseGroup;
use super::super::config::{Limits, LimitsExt};
use super::super::report::charts::line_graph::{LineGraphData, render_line_graph};
use super::super::report::charts::scaled_chart_width;
use super::QCModule;
use std::io;

#[derive(Clone, Default)]
pub struct SequenceContent {
    /// Per-position base counts stored as [A, C, G, T] per position.
    /// Using a single Vec of arrays gives better cache locality than
    /// four separate Vecs, since all counts for a position are adjacent.
    counts: Vec<[u64; 4]>,
    data: Option<ContentData>,
}

impl std::ops::Add for SequenceContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.counts.len() > rhs.counts.len() {
            (self.counts, rhs.counts)
        } else {
            (rhs.counts, self.counts)
        };
        for i in 0..shorter.len() {
            for (x, y) in longer[i].iter_mut().zip(shorter[i].iter()) {
                *x += *y;
            }
        }
        Self {
            counts: longer,
            data: None,
        }
    }
}

impl std::fmt::Display for SequenceContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.counts.len())
    }
}

impl QCModule for SequenceContent {
    fn name(&self) -> &str {
        "Per base sequence content"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        let groups = BaseGroup::make_base_groups(self.counts.len(), false, false);

        let mut x_categories = Vec::with_capacity(groups.len());
        let mut g_percent = vec![0.0f64; groups.len()];
        let mut a_percent = vec![0.0f64; groups.len()];
        let mut t_percent = vec![0.0f64; groups.len()];
        let mut c_percent = vec![0.0f64; groups.len()];

        for (i, group) in groups.iter().enumerate() {
            x_categories.push(group.label());

            let mut a_count: u64 = 0;
            let mut c_count: u64 = 0;
            let mut g_count: u64 = 0;
            let mut t_count: u64 = 0;
            let mut total: u64 = 0;

            // Java iterates `for (int bp=groups[i].lowerCount()-1;bp<groups[i].upperCount();bp++)`
            // which is 0-based lowerCount-1 to upperCount-1 inclusive. Our lower_count/upper_count
            // are already 0-based.
            for bp in group.lower_count..=group.upper_count {
                let c = &self.counts[bp];
                a_count += c[IDX_A];
                c_count += c[IDX_C];
                g_count += c[IDX_G];
                t_count += c[IDX_T];
                total += c[IDX_A] + c[IDX_C] + c[IDX_G] + c[IDX_T];
            }

            g_percent[i] = (g_count as f64 / total as f64) * 100.0;
            a_percent[i] = (a_count as f64 / total as f64) * 100.0;
            t_percent[i] = (t_count as f64 / total as f64) * 100.0;
            c_percent[i] = (c_count as f64 / total as f64) * 100.0;
        }

        // percentages stored in order [T, C, A, G] matching Java's array layout
        self.data = Some(ContentData {
            x_categories,
            t_percent,
            c_percent,
            a_percent,
            g_percent,
            error_threshold: limits.threshold("sequence\terror", 20.0),
            warn_threshold: limits.threshold("sequence\twarn", 10.0),
            ignored: limits.is_ignored("sequence"),
        });

        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        self.data.as_ref().unwrap().ignored
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        let data = self.data.as_ref().unwrap();

        // Check GC diff (C vs G) and AT diff (T vs A) per group
        for i in 0..data.g_percent.len() {
            let gc_diff = (data.c_percent[i] - data.g_percent[i]).abs();
            let at_diff = (data.t_percent[i] - data.a_percent[i]).abs();

            if gc_diff > data.error_threshold || at_diff > data.error_threshold {
                return true;
            }
        }
        false
    }

    fn raises_warning(&self) -> bool {
        let data = self.data.as_ref().unwrap();

        // Check GC diff (C vs G) and AT diff (T vs A) per group
        for i in 0..data.g_percent.len() {
            let gc_diff = (data.c_percent[i] - data.g_percent[i]).abs();
            let at_diff = (data.t_percent[i] - data.a_percent[i]).abs();

            if gc_diff > data.warn_threshold || at_diff > data.warn_threshold {
                return true;
            }
        }
        false
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        let data = self.data.clone().unwrap();

        // Series order in LineGraph is [%T, %C, %A, %G], matching Java's
        // `new LineGraph(percentages, 0d, 100d, ..., new String[] {"%T","%C","%A","%G"}, ...)`
        let svg = render_line_graph(&LineGraphData {
            width: scaled_chart_width(data.x_categories.len()),
            data: vec![
                data.t_percent,
                data.c_percent,
                data.a_percent,
                data.g_percent,
            ],
            min_y: 0.0,
            max_y: 100.0,
            x_label: "Position in read (bp)".to_string(),
            series_names: vec![
                "%T".to_string(),
                "%C".to_string(),
                "%A".to_string(),
                "%G".to_string(),
            ],
            x_categories: data.x_categories,
            title: "Sequence content across all bases".to_string(),
        });
        Some(("Per base sequence content", svg))
    }
}

#[derive(Clone, Default)]
pub struct PerBaseSequenceContent {
    content: SequenceContent,
}

impl Process for PerBaseSequenceContent {
    type Value = SequenceContent;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq = &sequence.sequence;

        // Grow array if needed
        if self.content.counts.len() < seq.len() {
            self.content.counts.resize(seq.len(), [0; 4]);
        }

        // Use lookup table for branchless base classification.
        // Only A/C/G/T (indices 0-3) are counted; N and others (indices 4-5)
        // are filtered by the single comparison `idx < 4`.
        for (i, &b) in seq.iter().enumerate() {
            let idx = BASE_INDEX[b as usize] as usize;
            if idx < 4 {
                self.content.counts[i][idx] += 1;
            }
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.content.to_owned()
    }
}

#[derive(Clone)]
struct ContentData {
    x_categories: Vec<String>,
    t_percent: Vec<f64>,
    c_percent: Vec<f64>,
    a_percent: Vec<f64>,
    g_percent: Vec<f64>,
    error_threshold: f64,
    warn_threshold: f64,
    ignored: bool,
}
