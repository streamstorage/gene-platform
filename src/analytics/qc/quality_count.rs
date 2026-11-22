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

    pub fn get_total_count(&self) -> u64 {
        self.total_counts
    }

    /// Lowest ASCII character with a non-zero count.
    pub fn get_min_char(&self) -> Option<u8> {
        for i in 0..self.actual_counts.len() {
            if self.actual_counts[i] > 0 {
                return Some(i as u8);
            }
        }
        None
    }

    /// Highest ASCII character with a non-zero count.
    pub fn get_max_char(&self) -> Option<u8> {
        for i in (0..self.actual_counts.len()).rev() {
            if self.actual_counts[i] > 0 {
                return Some(i as u8);
            }
        }
        None
    }

    /// Weighted mean quality score (offset-adjusted).
    ///
    /// Matches `getMean(int offset)`. Iterates from `offset` to 149,
    /// accumulating `count * (index - offset)` and dividing by total count.
    /// Returns NaN when count is 0 (Java's 0.0/0 behaviour).
    pub fn get_mean(&self, offset: u8) -> f64 {
        let mut total: u64 = 0;
        let mut count: u64 = 0;
        let off = offset as usize;

        for i in off..self.actual_counts.len() {
            total += self.actual_counts[i] * (i - off) as u64;
            count += self.actual_counts[i];
        }

        // JAVA COMPAT: Java returns ((double)total)/count which is NaN when count == 0.
        total as f64 / count as f64
    }

    /// Percentile quality score (offset-adjusted).
    ///
    /// Matches `getPercentile(int offset, int percentile)` exactly,
    /// including the use of integer arithmetic for threshold calculation:
    ///   total = totalCounts * percentile / 100   (integer division)
    /// This is critical for byte-exact output matching.
    pub fn get_percentile(&self, offset: u8, percentile: u8) -> f64 {
        // JAVA COMPAT: Java uses `long total = totalCounts; total *= percentile; total /= 100;`
        // which is integer multiplication then integer division (truncating).
        let mut threshold: u64 = self.total_counts;
        threshold *= percentile as u64;
        threshold /= 100;

        let off = offset as usize;
        let mut count: u64 = 0;

        for i in off..self.actual_counts.len() {
            count += self.actual_counts[i];
            if count >= threshold {
                // JAVA COMPAT: Java returns `(char)(i - offset)` cast to double.
                // The char-to-double cast in Java just gives the numeric value.
                return (i - off) as f64;
            }
        }

        // JAVA COMPAT: Java returns -1 when no value found (cast to double = -1.0).
        -1.0
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
