use super::super::super::sequence::{Process, Sequence};
use aho_corasick::{Span, packed};
use memchr::memmem;

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
}

impl AdapterContentData {
    pub fn new(adapters: Vec<Adapter>) -> Self {
        Self {
            adapters,
            total_count: 0,
        }
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
            for j in 0..rhs.adapters[i].first_hits.len() {
                self.adapters[i].first_hits[i] += rhs.adapters[i].first_hits[j];
            }
        }

        Self {
            adapters: self.adapters,
            total_count,
        }
    }
}

impl std::fmt::Display for AdapterContentData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.total_count)
    }
}

#[derive(Clone)]
pub struct AdapterContent {
    data: AdapterContentData,
    search: AdapterSearch,
    longest_sequence: usize,
    longest_adapter: usize,
}

impl AdapterContent {
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
            data: AdapterContentData::new(adapters),
            search: AdapterSearch::new(adapter_entries),
            longest_sequence: 0,
            longest_adapter,
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
        if seq_len > self.longest_sequence && seq_len > self.longest_adapter {
            self.longest_sequence = seq_len;
            let new_len = (self.longest_sequence - self.longest_adapter) + 1;
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
