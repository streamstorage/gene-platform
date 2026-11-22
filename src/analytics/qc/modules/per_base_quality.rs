use super::super::super::sequence::{Process, Sequence};
use super::super::quality_count::QualityCount;

#[derive(Clone, Default)]
pub struct PerBaseQualityCounts {
    counts: Vec<QualityCount>,
}

impl std::ops::Add for PerBaseQualityCounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.counts.len() > rhs.counts.len() {
            (self.counts, rhs.counts)
        } else {
            (rhs.counts, self.counts)
        };
        for i in 0..shorter.len() {
            longer[i] += shorter[i].clone();
        }
        Self { counts: longer }
    }
}

impl std::fmt::Display for PerBaseQualityCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.counts.is_empty() {
                0
            } else {
                self.counts[0].get_total_count()
            }
        )
    }
}

#[derive(Clone, Default)]
pub struct PerBaseQuality {
    quality_counts: PerBaseQualityCounts,
}

impl Process for PerBaseQuality {
    type Value = PerBaseQualityCounts;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let qual = &sequence.quality;

        // Grow the quality_counts array if needed
        if self.quality_counts.counts.len() < qual.len() {
            self.quality_counts
                .counts
                .resize_with(qual.len(), QualityCount::default);
        }

        for (i, &q) in qual.iter().enumerate() {
            self.quality_counts.counts[i].add_value(q);
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.quality_counts.to_owned()
    }
}
