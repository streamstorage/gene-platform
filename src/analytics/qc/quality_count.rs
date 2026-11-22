/// Per-position quality score accumulator.
///
/// Replicates the logic from `Utilities/QualityCount.java`.
/// Accumulates quality score counts for a single read position.
///
/// JAVA COMPAT: Uses a fixed 150-slot array indexed by ASCII value, matching
/// the Java `long[] actualCounts = new long[150]` exactly.
#[derive(Clone, Debug)]
pub struct QualityCount {
    actual_counts: [u64; 150],
    total_counts: u64,
}

impl Default for QualityCount {
    fn default() -> Self {
        Self {
            actual_counts: [0u64; 150],
            total_counts: 0,
        }
    }
}

impl QualityCount {
    /// Record a quality character (raw ASCII value, not offset-adjusted).
    ///
    /// Matches `addValue(char c)` which indexes by `(int)c`.
    #[inline]
    pub fn add_value(&mut self, quality_char: u8) {
        let mut idx = quality_char as usize;
        if idx >= self.actual_counts.len() {
            // Java throws ArrayIndexOutOfBoundsException here; clamp to the last
            // slot so a corrupt quality char doesn't abort the run.
            idx = self.actual_counts.len() - 1;
            // TODO: support logger
            log::warn!(
                "Warning: quality character '{}' (ASCII {}) exceeds maximum {}; clamping",
                quality_char as char,
                quality_char,
                idx
            );
        }
        self.actual_counts[idx] += 1;
        self.total_counts += 1;
    }
}

impl std::ops::AddAssign for QualityCount {
    fn add_assign(&mut self, rhs: Self) {
        for (x, y) in self.actual_counts.iter_mut().zip(rhs.actual_counts.iter()) {
            *x += *y;
        }
        self.total_counts += rhs.total_counts;
    }
}
