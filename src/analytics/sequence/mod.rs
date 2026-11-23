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

pub trait Process: Send + Sync {
    type Value: std::ops::Add<Output = Self::Value> + Send + std::fmt::Display + 'static;

    fn ref_into(&self) -> Self::Value;

    /// Process a single sequence record, accumulating statistics.
    fn process(&mut self, sequence: &Sequence);

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>>;
}

pub trait ProcessedValue: std::fmt::Display + Send {
    fn add(
        self: Box<Self>,
        rhs: Box<dyn ProcessedValue>,
    ) -> std::io::Result<Box<dyn ProcessedValue>>;

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any + Send>;
}

impl<T> ProcessedValue for T
where
    T: std::ops::Add<Output = T> + Send + std::fmt::Display + 'static,
{
    fn add(
        self: Box<Self>,
        rhs: Box<dyn ProcessedValue>,
    ) -> std::io::Result<Box<dyn ProcessedValue>> {
        let rhs = rhs.into_any().downcast::<T>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "incompatible process accumulator types",
            )
        })?;
        Ok(Box::new(*self + *rhs))
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any + Send> {
        self
    }
}

pub trait ErasedProcess: Send + Sync {
    fn process(&mut self, sequence: &Sequence);

    fn ref_into(&self) -> Box<dyn ProcessedValue>;

    fn clone_box(&self) -> Box<dyn ErasedProcess>;
}

impl Clone for Box<dyn ErasedProcess> {
    fn clone(&self) -> Box<dyn ErasedProcess> {
        (**self).clone_box()
    }
}

struct ProcessAdapter<T: std::ops::Add<Output = T> + Send + std::fmt::Display + 'static>(
    Box<dyn Process<Value = T>>,
);

impl<T> ErasedProcess for ProcessAdapter<T>
where
    T: std::ops::Add<Output = T> + Send + std::fmt::Display + 'static,
{
    fn process(&mut self, sequence: &Sequence) {
        self.0.process(sequence);
    }

    fn ref_into(&self) -> Box<dyn ProcessedValue> {
        Box::new(self.0.ref_into())
    }

    fn clone_box(&self) -> Box<dyn ErasedProcess> {
        Box::new(ProcessAdapter(self.0.clone()))
    }
}

pub fn erase_process<T>(process: Box<dyn Process<Value = T>>) -> Box<dyn ErasedProcess>
where
    T: std::ops::Add<Output = T> + Send + std::fmt::Display + 'static,
{
    Box::new(ProcessAdapter(process))
}

impl<T: std::ops::Add<Output = T> + Send + std::fmt::Display + 'static> Clone
    for Box<dyn Process<Value = T>>
{
    fn clone(&self) -> Box<dyn Process<Value = T>> {
        (**self).clone_box()
    }
}

/// Index constants for readability.
pub const IDX_A: usize = 0;
pub const IDX_C: usize = 1;
pub const IDX_G: usize = 2;
pub const IDX_T: usize = 3;
pub const IDX_N: usize = 4;

/// Lookup table: maps ASCII byte to base index.
/// 0=A, 1=C, 2=G, 3=T, 4=N, 5=other
///
/// Using a lookup table instead of match/if-else eliminates branch misprediction
/// on random DNA data (where each branch has ~25% probability, the worst case
/// for branch predictors).
pub const BASE_INDEX: [u8; 256] = {
    let mut table = [5u8; 256];
    table[b'A' as usize] = IDX_A as u8;
    table[b'C' as usize] = IDX_C as u8;
    table[b'G' as usize] = IDX_G as u8;
    table[b'T' as usize] = IDX_T as u8;
    table[b'N' as usize] = IDX_N as u8;
    table
};

/// Counts of uppercase `A`, `C`, `G` and `T` bytes.
///
/// Per-chunk byte counters let all four counts vectorise; indexing a counter
/// array per base is a serial dependency chain.
pub fn count_acgt(seq: &[u8]) -> [u64; 4] {
    let mut totals = [0u64; 4];
    for chunk in seq.chunks(u8::MAX as usize) {
        let mut counts = [0u8; 4];
        for &b in chunk {
            counts[IDX_A] += (b == b'A') as u8;
            counts[IDX_C] += (b == b'C') as u8;
            counts[IDX_G] += (b == b'G') as u8;
            counts[IDX_T] += (b == b'T') as u8;
        }
        for (total, count) in totals.iter_mut().zip(counts) {
            *total += count as u64;
        }
    }
    totals
}
