use super::super::super::sequence::{Process, Sequence};
use std::collections::HashMap;

/// The 16 duplication level labels, matching DuplicationLevel.java
const LABELS: [&str; 16] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", ">10", ">50", ">100", ">500", ">1k", ">5k", ">10k",
];

/// Maximum number of unique sequences to track
const OBSERVATION_CUTOFF: usize = 100_000;

#[derive(Clone, Default)]
pub struct OverRepresentedData {
    /// Map of truncated sequence -> count
    pub sequences: HashMap<String, u64>,
    /// Total number of sequences processed
    pub count: u64,
    /// Total count at the point where we reached the unique sequence limit
    pub count_at_unique_limit: u64,
}

impl std::ops::Add for OverRepresentedData {
    type Output = Self;

    fn add(self, _rhs: Self) -> Self {
        unreachable!("cannot be in parallel")
    }
}

impl std::fmt::Display for OverRepresentedData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "count: {},  count_at_unique_limit {}",
            self.count, self.count_at_unique_limit
        )
    }
}

/// Result of contaminant matching for an overrepresented sequence.
struct ContaminantHit {
    name: String,
    length: usize,
    percent_id: usize,
}

impl ContaminantHit {
    /// Returns true if this hit is better (longer match or higher identity at same length)
    /// than `other`, or if `other` is None.
    /// Replicates the comparison logic used throughout Contaminant.findMatch().
    fn is_better_than(&self, other: &Option<ContaminantHit>) -> bool {
        match other {
            None => true,
            Some(b) => {
                self.length > b.length
                    || (self.length == b.length && self.percent_id > b.percent_id)
            }
        }
    }
}

impl std::fmt::Display for ContaminantHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format matches ContaminantHit.toString()
        write!(
            f,
            "{} ({}% over {}bp)",
            self.name, self.percent_id, self.length
        )
    }
}

/// A single overrepresented sequence entry for reporting.
struct OverrepresentedSeq {
    seq: String,
    count: u64,
    percentage: f64,
    contaminant_hit: Option<ContaminantHit>,
}

/// A contaminant entry loaded from the contaminant list.
#[derive(Clone)]
struct Contaminant {
    name: String,
    forward: Vec<u8>,
    reverse: Vec<u8>,
}

impl Contaminant {
    fn new(name: &str, sequence: &str) -> Self {
        let forward: Vec<u8> = sequence.to_uppercase().bytes().collect();
        // Reverse complement computed exactly as in Contaminant.java
        let mut reverse = vec![0u8; forward.len()];
        for (c, &base) in forward.iter().enumerate() {
            let rev_pos = (forward.len() - 1) - c;
            reverse[rev_pos] = match base {
                b'G' => b'C',
                b'A' => b'T',
                b'T' => b'A',
                b'C' => b'G',
                _ => base,
            };
        }
        Contaminant {
            name: name.to_string(),
            forward,
            reverse,
        }
    }

    /// Replicates Contaminant.findMatch() - for short sequences (<20bp, >=8bp)
    /// checks if query is a substring of the contaminant. For longer sequences, slides with
    /// 1 mismatch tolerance and requires >=20bp match length.
    fn find_match(&self, query: &str) -> Option<ContaminantHit> {
        let query_upper = query.to_uppercase();

        // Special case for short queries (8-19bp) - exact substring match
        if query_upper.len() < 20 && query_upper.len() >= 8 {
            let forward_str = std::str::from_utf8(&self.forward).unwrap_or("");
            let reverse_str = std::str::from_utf8(&self.reverse).unwrap_or("");

            if forward_str.contains(&query_upper) {
                return Some(ContaminantHit {
                    name: self.name.clone(),
                    length: query_upper.len(),
                    percent_id: 100,
                });
            }
            if reverse_str.contains(&query_upper) {
                return Some(ContaminantHit {
                    name: self.name.clone(),
                    length: query_upper.len(),
                    percent_id: 100,
                });
            }
        }

        let q: Vec<u8> = query_upper.bytes().collect();
        let mut best_hit: Option<ContaminantHit> = None;

        // Check forward strand with sliding window and 1 mismatch tolerance
        best_hit = Self::find_strand_match(&self.forward, &q, best_hit, &self.name);
        best_hit = Self::find_strand_match(&self.reverse, &q, best_hit, &self.name);

        best_hit
    }

    fn find_strand_match(
        ca: &[u8],
        cb: &[u8],
        mut best_hit: Option<ContaminantHit>,
        name: &str,
    ) -> Option<ContaminantHit> {
        let min_offset = -(ca.len() as isize - 20);
        let max_offset = cb.len() as isize - 20;

        for offset in min_offset..max_offset {
            if let Some(hit) = Self::find_match_at_offset(ca, cb, offset, name) {
                if hit.is_better_than(&best_hit) {
                    best_hit = Some(hit);
                }
            }
        }
        best_hit
    }

    /// Replicates the private findMatch(char[], char[], int, int) method.
    fn find_match_at_offset(
        ca: &[u8],
        cb: &[u8],
        offset: isize,
        name: &str,
    ) -> Option<ContaminantHit> {
        let mut best_hit: Option<ContaminantHit> = None;
        let mut mismatch_count: usize = 0;
        // Use isize to avoid underflow when offset causes start > end
        let mut start: isize = 0;
        let mut end: isize = 0;

        // index i used to access both ca[i] and cb[i+offset]
        for (i, &ca_byte) in ca.iter().enumerate() {
            let j = i as isize + offset;
            if j < 0 {
                start = i as isize + 1;
                continue;
            }
            if j >= cb.len() as isize {
                break;
            }

            if ca_byte == cb[j as usize] {
                end = i as isize;
            } else {
                mismatch_count += 1;
                if mismatch_count > 1 {
                    // Check if match so far is worth recording (>20bp)
                    if end >= start {
                        let match_len = (1 + end - start) as usize;
                        if match_len > 20 {
                            let id = ((match_len - (mismatch_count - 1)) * 100) / match_len;
                            let candidate = ContaminantHit {
                                name: name.to_string(),
                                length: match_len,
                                percent_id: id,
                            };
                            if candidate.is_better_than(&best_hit) {
                                best_hit = Some(candidate);
                            }
                        }
                    }
                    start = i as isize + 1;
                    end = i as isize + 1;
                    mismatch_count = 0;
                }
            }
        }

        // Check final stretch
        if end < start {
            return best_hit;
        }
        let match_len = (1 + end - start) as usize;
        if match_len > 20 {
            let id = ((match_len - mismatch_count) * 100) / match_len;
            let candidate = ContaminantHit {
                name: name.to_string(),
                length: match_len,
                percent_id: id,
            };
            if candidate.is_better_than(&best_hit) {
                best_hit = Some(candidate);
            }
        }

        best_hit
    }
}

/// Find the best contaminant match for a query sequence.
/// Replicates ContaminentFinder.findContaminantHit()
fn find_contaminant_hit(query: &str, contaminants: &[Contaminant]) -> Option<ContaminantHit> {
    let mut best_hit: Option<ContaminantHit> = None;

    for contaminant in contaminants {
        if let Some(hit) = contaminant.find_match(query) {
            if hit.is_better_than(&best_hit) {
                best_hit = Some(hit);
            }
        }
    }

    best_hit
}

#[derive(Clone)]
pub struct OverrepresentedSequences {
    data: OverRepresentedData,

    /// Maximum 100000 unique sequences tracked
    unique_sequence_count: usize,
    frozen: bool,
    dup_length: usize,
    contaminants: Vec<Contaminant>,
}

impl OverrepresentedSequences {
    pub fn new(dup_length: usize, contaminant_entries: &[(String, String)]) -> Self {
        let contaminants: Vec<Contaminant> = contaminant_entries
            .iter()
            .map(|(name, seq)| Contaminant::new(name, seq))
            .collect();
        Self {
            data: OverRepresentedData::default(),
            unique_sequence_count: 0,
            frozen: false,
            dup_length,
            contaminants,
        }
    }
}

impl Process for OverrepresentedSequences {
    type Value = OverRepresentedData;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        self.data.count += 1;

        // Truncate sequence to dup_length or 50bp if longer than 50
        // Safety: sequence bytes are ASCII (uppercased in Sequence::new), so slicing is valid UTF-8
        let seq_bytes = &sequence.sequence;
        let truncate_len = if self.dup_length != 0 && seq_bytes.len() > self.dup_length {
            self.dup_length
        } else if seq_bytes.len() > 50 {
            // Default truncation to 50bp for sequences longer than 50
            50
        } else {
            seq_bytes.len()
        };
        let seq = std::str::from_utf8(&seq_bytes[..truncate_len]).unwrap_or("");

        if let Some(count) = self.data.sequences.get_mut(seq) {
            *count += 1;
            // Keep updating countAtUniqueLimit while not frozen
            if !self.frozen {
                self.data.count_at_unique_limit = self.data.count;
            }
        } else if !self.frozen {
            self.data.sequences.insert(seq.to_string(), 1);
            self.unique_sequence_count += 1;
            self.data.count_at_unique_limit = self.data.count;
            if self.unique_sequence_count == OBSERVATION_CUTOFF {
                self.frozen = true;
            }
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.data.to_owned()
    }
}
