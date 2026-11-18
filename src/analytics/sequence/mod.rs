pub mod fastq;
pub mod phred;

/// A single sequence record with ID, bases, quality scores, and filter status.
///
/// Mirrors `Sequence.Sequence` in Java. The `sequence` field stores
/// uppercase ASCII bases as bytes (matching Java's `toUpperCase()` in the constructor).
/// The `quality` field stores raw ASCII quality characters as bytes.
#[derive(Debug, Clone)]
pub struct Sequence {
    pub id: String,
    /// Uppercase ASCII nucleotide bases (A, C, G, T, N).
    pub sequence: Vec<u8>,
    /// Raw ASCII quality characters (not yet offset-adjusted).
    pub quality: Vec<u8>,
}

impl Sequence {
    /// Create a new Sequence, converting the base sequence to uppercase.
    ///
    /// The Java constructor calls `sequence.toUpperCase()` on the
    /// sequence string, so we replicate that here.
    pub fn new(id: String, mut sequence: Vec<u8>, quality: Vec<u8>) -> Self {
        // uppercase conversion matches Java constructor behavior.
        // In-place mutation avoids allocating a new Vec.
        sequence.make_ascii_uppercase();
        Self {
            id,
            sequence,
            quality,
        }
    }

    /// Length of the sequence in bases.
    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    /// Whether the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    /// Roughly how much heap this record holds, for the analysis pipeline's
    /// memory budgeting. The `Sequence` itself is not counted — only what it
    /// owns, which is everything that scales with read length.
    pub fn heap_bytes(&self) -> usize {
        self.id.len() + self.sequence.len() + self.quality.len()
    }
}

/// Trait for reading sequences from various file formats.
///
/// Mirrors `Sequence.SequenceFile` interface.
pub trait SequenceFile: Send {
    /// Read the next sequence from the file, or None at EOF.
    fn next(&mut self) -> Option<std::io::Result<Sequence>>;

    /// The display name of this file (typically the filename).
    fn name(&self) -> &str;

    /// The quality encoding of this file, if the format specifies it.
    ///
    /// FASTQ carries no encoding information, so the default is `None` and
    /// the encoding is inferred downstream from the lowest observed quality
    /// character. BAM/SAM override this: their quality data is Phred+33 by
    /// construction, so inference would be guessing at a known answer (and
    /// guesses wrong when no base is below Q31).
    fn known_phred_encoding(&self) -> Option<phred::PhredEncoding> {
        None
    }

    /// Estimated percentage complete (0.0 - 100.0), for progress display.
    fn percent_complete(&self) -> f64;

    /// Threads decompressing this file in the background, beside the one
    /// calling [`next`](Self::next), so the runner can count them against the
    /// thread budget.
    fn background_threads(&self) -> usize {
        0
    }
}

pub trait Transform: Send + Sync {
    type Value;

    /// Process a single sequence record, accumulating statistics.
    fn transform(&self, sequence: &Sequence) -> Self::Value;

    fn none(&self) -> Self::Value;
}

/// Index constants for readability.
pub const IDX_A: usize = 0;
pub const IDX_C: usize = 1;
pub const IDX_G: usize = 2;
pub const IDX_T: usize = 3;
pub const IDX_N: usize = 4;

/// Counts of uppercase `A`, `C`, `G` and `T` bytes.
///
/// Per-chunk byte counters let all four counts vectorise; indexing a counter
/// array per base is a serial dependency chain.
pub fn count_acgt(seq: &[u8]) -> [u64; 4] {
    let mut totals = [0u64; 4];
    for chunk in seq.chunks(u8::MAX as usize) {
        let mut counts = [0u8; 4];
        for &b in chunk {
            counts[0] += (b == b'A') as u8;
            counts[1] += (b == b'C') as u8;
            counts[2] += (b == b'G') as u8;
            counts[3] += (b == b'T') as u8;
        }
        for (total, count) in totals.iter_mut().zip(counts) {
            *total += count as u64;
        }
    }
    totals
}
