use super::super::super::sequence::{BASE_INDEX, IDX_N, Process, Sequence};

#[derive(Clone, Default)]
pub struct NContent {
    n_counts: Vec<u64>,
    not_n_counts: Vec<u64>,
}

impl std::ops::Add for NContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer_n_counts, shorter_n_counts, mut longer_not_n_counts, shorter_not_n_counts) =
            if self.n_counts.len() > rhs.n_counts.len() {
                (
                    self.n_counts,
                    rhs.n_counts,
                    self.not_n_counts,
                    rhs.not_n_counts,
                )
            } else {
                (
                    rhs.n_counts,
                    self.n_counts,
                    rhs.not_n_counts,
                    self.not_n_counts,
                )
            };
        for i in 0..longer_n_counts.len() {
            longer_n_counts[i] += shorter_n_counts[i];
            longer_not_n_counts[i] += shorter_not_n_counts[i];
        }
        Self {
            n_counts: longer_n_counts,
            not_n_counts: longer_not_n_counts,
        }
    }
}

impl std::fmt::Display for NContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.n_counts.len())
    }
}

#[derive(Clone, Default)]
pub struct PerBaseNContent {
    content: NContent,
}

impl Process for PerBaseNContent {
    type Value = NContent;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq = &sequence.sequence;

        // Grow arrays if needed
        if self.content.n_counts.len() < seq.len() {
            self.content.n_counts.resize(seq.len(), 0);
            self.content.not_n_counts.resize(seq.len(), 0);
        }

        // Use lookup table to classify each byte without a multi-way match
        for (i, &b) in seq.iter().enumerate() {
            if BASE_INDEX[b as usize] as usize == IDX_N {
                self.content.n_counts[i] += 1;
            } else {
                self.content.not_n_counts[i] += 1;
            }
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.content.to_owned()
    }
}
