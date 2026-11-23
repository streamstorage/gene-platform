use super::super::super::sequence::{Process, Sequence};
use super::super::quality_count::QualityCount;

#[derive(Clone, Default)]
pub struct PerBaseQualityCounts {
    quality_counts: Vec<QualityCount>,
}

impl std::ops::Add for PerBaseQualityCounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (mut longer, shorter) = if self.quality_counts.len() > rhs.quality_counts.len() {
            (self.quality_counts, rhs.quality_counts)
        } else {
            (rhs.quality_counts, self.quality_counts)
        };
        for i in 0..shorter.len() {
            longer[i] += shorter[i].clone();
        }
        Self {
            quality_counts: longer,
        }
    }
}

impl std::fmt::Display for PerBaseQualityCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.quality_counts.is_empty() {
                0
            } else {
                self.quality_counts[0].get_total_count()
            }
        )
    }
}

#[derive(Clone, Default)]
pub struct PerBaseSequenceQuality {
    counts: PerBaseQualityCounts,
}

impl Process for PerBaseSequenceQuality {
    type Value = PerBaseQualityCounts;

    fn clone_box(&self) -> Box<dyn Process<Value = Self::Value>> {
        Box::new(self.clone())
    }

    fn process(&mut self, sequence: &Sequence) {
        let qual = &sequence.quality;

        // Grow the quality_counts array if needed
        if self.counts.quality_counts.len() < qual.len() {
            self.counts
                .quality_counts
                .resize_with(qual.len(), QualityCount::default);
        }

        for (i, &q) in qual.iter().enumerate() {
            self.counts.quality_counts[i].add_value(q);
        }
    }

    fn ref_into(&self) -> Self::Value {
        self.counts.to_owned()
    }
}
