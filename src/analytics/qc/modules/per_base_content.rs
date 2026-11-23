use super::super::super::sequence::{BASE_INDEX, Process, Sequence};

#[derive(Clone, Default)]
pub struct SequenceContent {
    /// Per-position base counts stored as [A, C, G, T] per position.
    /// Using a single Vec of arrays gives better cache locality than
    /// four separate Vecs, since all counts for a position are adjacent.
    counts: Vec<[u64; 4]>,
}

impl std::ops::Add for SequenceContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.counts.len() > rhs.counts.len() {
            (self.counts, rhs.counts)
        } else {
            (rhs.counts, self.counts)
        };
        for i in 0..shorter.len() {
            for (x, y) in longer[i].iter_mut().zip(shorter[i].iter()) {
                *x += *y;
            }
        }
        Self { counts: longer }
    }
}

impl std::fmt::Display for SequenceContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.counts.len())
    }
}

#[derive(Clone, Default)]
pub struct PerBaseSequenceContent {
    content: SequenceContent,
}

impl Process for PerBaseSequenceContent {
    type Value = SequenceContent;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let seq = &sequence.sequence;

        // Grow array if needed
        if self.content.counts.len() < seq.len() {
            self.content.counts.resize(seq.len(), [0; 4]);
        }

        // Use lookup table for branchless base classification.
        // Only A/C/G/T (indices 0-3) are counted; N and others (indices 4-5)
        // are filtered by the single comparison `idx < 4`.
        for (i, &b) in seq.iter().enumerate() {
            let idx = BASE_INDEX[b as usize] as usize;
            if idx < 4 {
                self.content.counts[i][idx] += 1;
            }
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.content.to_owned()
    }
}
