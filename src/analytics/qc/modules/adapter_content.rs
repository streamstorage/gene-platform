use super::super::super::sequence::{Process, Sequence};
use super::super::base_group::BaseGroup;
use super::super::config::{Limits, LimitsExt};
use super::super::report::charts::line_graph::{LineGraphData, render_line_graph};
use super::super::report::charts::scaled_chart_width;
use super::QCModule;
use aho_corasick::{Span, packed};
use memchr::memmem;
use std::io;

#[derive(Clone)]
struct Adapter {
    name: String,
    /// Reads whose first match starts at each position; `cumulative_positions`
    /// gives Java's running totals.
    first_hits: Vec<u64>,
}

impl Adapter {
    fn new(name: &str) -> Self {
        Adapter {
            name: name.to_string(),
            first_hits: vec![0; 1],
        }
    }

    fn record_hit(&mut self, position: usize) {
        // Like Java, a match past the currently tracked positions is dropped,
        // not counted once a longer read extends them.
        if let Some(count) = self.first_hits.get_mut(position) {
            *count += 1;
        }
    }

    /// Java's `positions`: reads with a match at or before each position.
    fn cumulative_positions(&self) -> Vec<u64> {
        self.first_hits
            .iter()
            .scan(0u64, |total, &hits| {
                *total += hits;
                Some(*total)
            })
            .collect()
    }
}

/// Finds each adapter's first match in a read. One SIMD multi-pattern pass
/// (aho-corasick's Teddy) where possible, else a substring search per adapter.
#[derive(Clone)]
struct AdapterSearch {
    finders: Vec<memmem::Finder<'static>>,
    teddy: Option<packed::Searcher>,
}

impl AdapterSearch {
    fn new(adapters: &[(String, String)]) -> Self {
        let seqs: Vec<&[u8]> = adapters.iter().map(|(_, s)| s.as_bytes()).collect();
        AdapterSearch {
            finders: seqs
                .iter()
                .map(|s| memmem::Finder::new(s).into_owned())
                .collect(),
            teddy: Self::build_teddy(&seqs),
        }
    }

    /// `None` if Teddy can't be built for these patterns, or if one adapter is
    /// a prefix of another: leftmost-first reports one match per start position.
    fn build_teddy(seqs: &[&[u8]]) -> Option<packed::Searcher> {
        if seqs.is_empty() || seqs.len() > 64 || seqs.iter().any(|s| s.is_empty()) {
            return None;
        }
        for (i, a) in seqs.iter().enumerate() {
            for (j, b) in seqs.iter().enumerate() {
                if i != j && b.starts_with(a) {
                    return None;
                }
            }
        }
        packed::Config::new()
            .match_kind(packed::MatchKind::LeftmostFirst)
            .builder()
            .extend(seqs)
            .build()
    }

    /// Calls `on_hit(adapter_index, start)` once for the leftmost match of
    /// each adapter in `seq`.
    fn first_matches(&self, seq: &[u8], mut on_hit: impl FnMut(usize, usize)) {
        let Some(teddy) = &self.teddy else {
            for (a, finder) in self.finders.iter().enumerate() {
                if let Some(start) = finder.find(seq) {
                    on_hit(a, start);
                }
            }
            return;
        };

        let mut remaining: u64 = u64::MAX >> (64 - self.finders.len());
        let mut at = 0;
        while remaining != 0 && at < seq.len() {
            let Some(m) = teddy.find_in(seq, Span::from(at..seq.len())) else {
                return;
            };
            let a = m.pattern().as_usize();
            if remaining & (1 << a) == 0 {
                // A repeat (typically inside a polyA/polyG run) would otherwise
                // cost one restart per base. No adapter is a prefix of another,
                // so nothing unseen starts before here: search the rest singly.
                break;
            }
            remaining &= !(1 << a);
            on_hit(a, m.start());
            // One past the start, not the end, so overlapping matches of other
            // adapters aren't skipped.
            at = m.start() + 1;
        }
        let rest = &seq[at.min(seq.len())..];
        while remaining != 0 {
            let a = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            if let Some(start) = self.finders[a].find(rest) {
                on_hit(a, at + start);
            }
        }
    }
}

#[derive(Clone)]
pub struct AdapterContentData {
    adapters: Vec<Adapter>,
    total_count: u64,
    longest_sequence: usize,
    longest_adapter: usize,
    computed: Option<ComputedEnrichment>,
}

impl AdapterContentData {
    pub fn new(adapter_entries: &[(String, String)]) -> Self {
        let mut longest_adapter = 0;
        let mut adapters = Vec::with_capacity(adapter_entries.len());

        for (name, seq) in adapter_entries {
            if seq.len() > longest_adapter {
                longest_adapter = seq.len();
            }
            adapters.push(Adapter::new(name));
        }

        if adapter_entries
            .iter()
            .any(|(_, seq)| seq.len() != longest_adapter)
        {
            //TODO: support logger
            log::warn!(
                "[Warning] You are using adapter sequences with different lengths. Matches will only be reported up to the position where the longest adapter could match. Matches to shorter adapters at the end of sequences will not be recorded."
            );
        }

        Self {
            adapters,
            total_count: 0,
            longest_adapter,
            longest_sequence: 0,
            computed: None,
        }
    }

    /// No read was longer than the longest adapter, so Java skips the analysis
    /// (no table or chart) and just warns.
    fn reads_too_short(&self) -> bool {
        self.longest_adapter > self.longest_sequence
    }

    /// Derive adapter names from the adapters Vec (avoids storing a redundant copy).
    fn adapter_names(&self) -> Vec<String> {
        self.adapters.iter().map(|a| a.name.clone()).collect()
    }
}

impl std::ops::Add for AdapterContentData {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self {
        let total_count = self.total_count + rhs.total_count;
        for i in 0..self.adapters.len() {
            if self.adapters[i].first_hits.len() < rhs.adapters[i].first_hits.len() {
                std::mem::swap(&mut self.adapters[i], &mut rhs.adapters[i])
            };
            let len = rhs.adapters[i].first_hits.len();
            for (x, y) in self.adapters[i].first_hits[..len]
                .iter_mut()
                .zip(rhs.adapters[i].first_hits.iter())
            {
                *x += *y;
            }
        }

        Self {
            adapters: self.adapters,
            total_count,
            longest_sequence: self.longest_sequence.max(rhs.longest_sequence),
            longest_adapter: self.longest_adapter,
            computed: None,
        }
    }
}

impl std::fmt::Display for AdapterContentData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.total_count)
    }
}

impl QCModule for AdapterContentData {
    fn name(&self) -> &str {
        "Adapter Content"
    }

    fn calc(&mut self, limits: &Limits) -> io::Result<()> {
        let all_positions: Vec<Vec<u64>> = self
            .adapters
            .iter()
            .map(Adapter::cumulative_positions)
            .collect();
        let max_length = all_positions.iter().map(Vec::len).max().unwrap_or(0);

        // Group positions using BaseGroup
        let groups = BaseGroup::make_base_groups(max_length, false, false);

        let x_labels: Vec<String> = groups.iter().map(|g| g.label()).collect();

        let mut enrichments = vec![vec![0.0f64; groups.len()]; self.adapters.len()];

        for (a, positions) in all_positions.iter().enumerate() {
            for (g, group) in groups.iter().enumerate() {
                // lowerCount() is 1-based in Java, we use 0-based internally
                // Java: p=groups[g].lowerCount()-1; p<groups[g].upperCount()
                let lower = group.lower_count; // already 0-based
                let upper = group.upper_count; // already 0-based, inclusive

                for p in lower..=upper {
                    if p < positions.len() {
                        enrichments[a][g] +=
                            (positions[p] as f64 * 100.0) / self.total_count as f64;
                    }
                }

                // Average over the group width
                enrichments[a][g] /= (upper - lower + 1) as f64;
            }
        }

        self.computed = Some(ComputedEnrichment {
            enrichments,
            x_labels,
            error_threshold: limits.threshold("adapter\terror", 10.0),
            warn_threshold: limits.threshold("adapter\twarn", 5.0),
            ignored: limits.is_ignored("adapter"),
        });
        Ok(())
    }

    fn ignore_in_report(&self) -> bool {
        self.computed.as_ref().unwrap().ignored
    }
    // BasicStats never raises error or warning
    fn raises_error(&self) -> bool {
        let computed = self.computed.as_ref().unwrap();
        computed.enrichments.iter().any(|enrichments| {
            enrichments
                .iter()
                .any(|&val| val > computed.error_threshold)
        })
    }

    fn raises_warning(&self) -> bool {
        if self.reads_too_short() {
            return true;
        }
        let computed = self.computed.as_ref().unwrap();
        computed
            .enrichments
            .iter()
            .any(|enrichments| enrichments.iter().any(|&val| val > computed.warn_threshold))
    }

    fn content_msg(&self) -> Option<String> {
        if self.reads_too_short() {
            Some(format!(
                "Can't analyse adapters as read length is too short ({} vs {})",
                self.longest_adapter, self.longest_sequence
            ))
        } else {
            None
        }
    }

    fn content_chart(&self) -> Option<(&str, String)> {
        if self.reads_too_short() {
            return None;
        }
        let computed = self.computed.clone().unwrap();

        let svg = render_line_graph(&LineGraphData {
            width: scaled_chart_width(computed.x_labels.len()),
            data: computed.enrichments.clone(),
            min_y: 0.0,
            max_y: 100.0,
            x_label: "Position in read (bp)".to_string(),
            series_names: self.adapter_names(),
            x_categories: computed.x_labels.clone(),
            title: "% Adapter".to_string(),
        });
        Some(("Adapter graph", svg))
    }
}

#[derive(Clone)]
pub struct AdapterContent {
    data: AdapterContentData,
    search: AdapterSearch,
}

impl AdapterContent {
    pub fn new(adapter_entries: &[(String, String)]) -> Self {
        Self {
            data: AdapterContentData::new(adapter_entries),
            search: AdapterSearch::new(adapter_entries),
        }
    }
}

impl Process for AdapterContent {
    type Value = AdapterContentData;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        self.data.total_count += 1;

        let seq_len = sequence.sequence.len();

        // Java only grows the arrays once a read is longer than the longest adapter.
        if seq_len > self.data.longest_sequence && seq_len > self.data.longest_adapter {
            self.data.longest_sequence = seq_len;
            let new_len = (self.data.longest_sequence - self.data.longest_adapter) + 1;
            for adapter in &mut self.data.adapters {
                adapter.first_hits.resize(new_len, 0);
            }
        }

        let adapters = &mut self.data.adapters;
        self.search
            .first_matches(&sequence.sequence, |a, start| adapters[a].record_hit(start));
    }

    fn ref_into(&self) -> Self::Value {
        self.data.to_owned()
    }
}

#[derive(Clone)]
struct ComputedEnrichment {
    enrichments: Vec<Vec<f64>>,
    x_labels: Vec<String>,
    error_threshold: f64,
    warn_threshold: f64,
    ignored: bool,
}
