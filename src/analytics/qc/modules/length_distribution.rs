use super::super::super::sequence::{Process, Sequence};

#[derive(Clone, Default)]
pub struct LengthDistribution {
    /// length_counts[i] = number of sequences with length i
    length_counts: Vec<u64>,
}

impl std::ops::Add for LengthDistribution {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.length_counts.len() > rhs.length_counts.len() {
            (self.length_counts, rhs.length_counts)
        } else {
            (rhs.length_counts, self.length_counts)
        };

        for (x, y) in longer.iter_mut().zip(shorter.iter()) {
            *x += *y;
        }
        Self {
            length_counts: longer,
        }
    }
}

impl std::fmt::Display for LengthDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.length_counts.len())
    }
}

#[derive(Clone, Default)]
pub struct SequenceLengthDistribution {
    distribution: LengthDistribution,
}

impl Process for SequenceLengthDistribution {
    type Value = LengthDistribution;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq_len = sequence.sequence.len();

        // Array is extended to seqLen+2 to match Java's `seqLen+2 > lengthCounts.length`
        if seq_len + 2 > self.distribution.length_counts.len() {
            self.distribution.length_counts.resize(seq_len + 2, 0);
        }

        self.distribution.length_counts[seq_len] += 1;
    }

    fn ref_into(&self) -> Self::Value {
        self.distribution.to_owned()
    }
}
