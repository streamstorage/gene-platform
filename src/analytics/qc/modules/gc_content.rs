use super::super::super::sequence::{Process, Sequence, count_acgt};

/// Mirrors GCModel/GCModelValue.java - a single percentage bin and its increment weight.
#[derive(Clone)]
struct GCModelValue {
    percentage: usize,
    increment: f64,
}

/// Mirrors GCModel/GCModel.java - maps a GC base count to weighted percentage bins.
/// For a given read length, each possible GC count (0..=length) maps to one or more percentage
/// bins with fractional increments so that counts at bin boundaries are shared.
#[derive(Clone)]
struct GCModel {
    models: Vec<Vec<GCModelValue>>,
}

impl GCModel {
    fn new(read_length: usize) -> Self {
        // Two-pass algorithm from GCModel.java
        // First pass: count how many GC-count positions claim each percentage bin
        let mut claiming_counts = vec![0usize; 101];

        for pos in 0..=read_length {
            let low_count = if pos == 0 { 0.0 } else { pos as f64 - 0.5 };
            let high_count = (pos as f64 + 0.5).min(read_length as f64);
            // clamp is also applied for lowCount < 0 (only matters for pos==0)
            let low_count = low_count.max(0.0);

            let low_percentage = ((low_count * 100.0) / read_length as f64).round() as usize;
            let high_percentage = ((high_count * 100.0) / read_length as f64).round() as usize;

            for cc in &mut claiming_counts[low_percentage..=high_percentage] {
                *cc += 1;
            }
        }

        // Second pass: build the model with weighted increments
        let mut models = Vec::with_capacity(read_length + 1);

        for pos in 0..=read_length {
            let low_count = if pos == 0 { 0.0 } else { pos as f64 - 0.5 };
            let high_count = (pos as f64 + 0.5).min(read_length as f64);
            let low_count = low_count.max(0.0);

            let low_percentage = ((low_count * 100.0) / read_length as f64).round() as usize;
            let high_percentage = ((high_count * 100.0) / read_length as f64).round() as usize;

            let mut values = Vec::with_capacity((high_percentage - low_percentage) + 1);
            // need both index p (for percentage field) and claiming_counts[p]
            for (p, &cc) in (low_percentage..=high_percentage)
                .zip(&claiming_counts[low_percentage..=high_percentage])
            {
                values.push(GCModelValue {
                    percentage: p,
                    increment: 1.0 / cc as f64,
                });
            }
            models.push(values);
        }

        GCModel { models }
    }

    fn get_model_values(&self, gc_count: usize) -> &[GCModelValue] {
        &self.models[gc_count]
    }
}

#[derive(Clone)]
pub struct GCContent {
    gc_distribution: [f64; 101],
}

impl Default for GCContent {
    fn default() -> Self {
        Self {
            gc_distribution: [0.0; 101],
        }
    }
}

impl GCContent {
    fn calculate_distribution(&self) -> f64 {
        let mut total_count: f64 = 0.0;
        let mut first_mode: usize = 0;
        let mut mode_count: f64 = 0.0;

        for i in 0..101 {
            total_count += self.gc_distribution[i];
            if self.gc_distribution[i] > mode_count {
                mode_count = self.gc_distribution[i];
                first_mode = i;
            }
        }

        // Average over adjacent points that stay above 90% of the modal value
        // (the comment says 95% but the code checks gcDistribution[firstMode] - gcDistribution[firstMode]/10,
        // which is 90% of the mode value)
        let mut mode: f64 = 0.0;
        let mut mode_duplicates: usize = 0;
        let mut fell_off_top = true;

        for i in first_mode..101 {
            if self.gc_distribution[i]
                > self.gc_distribution[first_mode] - (self.gc_distribution[first_mode] / 10.0)
            {
                mode += i as f64;
                mode_duplicates += 1;
            } else {
                fell_off_top = false;
                break;
            }
        }

        let mut fell_off_bottom = true;
        if first_mode > 0 {
            for i in (0..first_mode).rev() {
                if self.gc_distribution[i]
                    > self.gc_distribution[first_mode] - (self.gc_distribution[first_mode] / 10.0)
                {
                    mode += i as f64;
                    mode_duplicates += 1;
                } else {
                    fell_off_bottom = false;
                    break;
                }
            }
        }

        // If distribution is so skewed that 90% of the mode falls off the
        // 0-100% scale, keep first_mode as center
        if fell_off_bottom || fell_off_top {
            mode = first_mode as f64;
        } else {
            mode /= mode_duplicates as f64;
        }

        // Calculate standard deviation
        let mut stdev: f64 = 0.0;
        for i in 0..101 {
            stdev += (i as f64 - mode).powi(2) * self.gc_distribution[i];
        }
        // Divides by totalCount-1 (Bessel's correction)
        stdev /= total_count - 1.0;
        stdev = stdev.sqrt();

        // Calculate theoretical distribution using the normal PDF
        // NormalDistribution.getZScoreForValue() is actually a PDF, not a z-score
        let mut deviation_percent: f64 = 0.0;
        let mut theoretical = [0.0f64; 101];

        // Calculate theoretical[i] and compare with gc_distribution[i]
        for (i, (theo, &observed)) in theoretical
            .iter_mut()
            .zip(self.gc_distribution.iter())
            .enumerate()
        {
            let lhs = 1.0 / (2.0 * std::f64::consts::PI * stdev * stdev).sqrt();
            let rhs =
                std::f64::consts::E.powf(-(((i as f64) - mode).powi(2)) / (2.0 * stdev * stdev));
            *theo = lhs * rhs * total_count;

            deviation_percent += (*theo - observed).abs();
        }

        deviation_percent /= total_count;
        deviation_percent *= 100.0;

        // self.theoretical_distribution = Some(theoretical);
        deviation_percent
    }
}

impl std::ops::Add for GCContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let gc_distribution =
            std::array::from_fn(|i| self.gc_distribution[i] + rhs.gc_distribution[i]);
        Self { gc_distribution }
    }
}

impl std::fmt::Display for GCContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "deviation percent: {}", self.calculate_distribution())
    }
}

#[derive(Clone, Default)]
pub struct PerSequenceGCContent {
    content: GCContent,
    // Cache GCModels by read length to avoid recomputing
    cached_models: Vec<Option<GCModel>>,
}

impl PerSequenceGCContent {
    /// Truncate sequence to reduce number of distinct GCModel lengths.
    /// Sequences >1000bp are truncated to a multiple of 1000, >100bp to a multiple of 100.
    fn truncate_length(len: usize) -> usize {
        if len > 1000 {
            (len / 1000) * 1000
        } else if len > 100 {
            (len / 100) * 100
        } else {
            len
        }
    }
}

impl Process for PerSequenceGCContent {
    type Value = GCContent;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq = &sequence.sequence;
        let truncated_len = Self::truncate_length(seq.len());
        if truncated_len == 0 {
            return;
        }

        // Count G and C in the truncated portion only
        let [_, c, g, _] = count_acgt(&seq[..truncated_len]);
        let gc_count = (c + g) as usize;

        // Ensure cache is large enough
        if truncated_len >= self.cached_models.len() {
            self.cached_models.resize_with(truncated_len + 1, || None);
        }

        // Create model if not cached
        if self.cached_models[truncated_len].is_none() {
            self.cached_models[truncated_len] = Some(GCModel::new(truncated_len));
        }

        let model = self.cached_models[truncated_len].as_ref().unwrap();
        let values = model.get_model_values(gc_count);

        for v in values {
            self.content.gc_distribution[v.percentage] += v.increment;
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.content.to_owned()
    }
}
