pub mod adapter_content; // 9  9
pub mod basic_stats; // 0  2
pub mod gc_content; // 4  2
pub mod length_distribution; //6  1
pub mod n_content; // 5  8
//pub mod overrepresented_seqs; // 8/7  4/1 duplication level
pub mod per_base_content; // 3  9
pub mod per_base_quality; // 1  10
pub mod per_sequence_quality; // 2  1

use super::config::Limits;

/// Status of a QC module after analysis, matching Java FastQC's pass/warn/fail icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    Pass,
    Warn,
    Fail,
}

pub trait QCModule {
    /// The display name of this module as shown in the report.
    fn name(&self) -> &str;

    /// The current status of this module.
    fn status(&self) -> ModuleStatus {
        if self.raises_error() {
            ModuleStatus::Fail
        } else if self.raises_warning() {
            ModuleStatus::Warn
        } else {
            ModuleStatus::Pass
        }
    }

    /// Whether this module should be excluded from the report.
    /// Matches `ignoreInReport()` - used e.g. by PerTileQuality when
    /// there are no tile IDs in the data.
    fn ignore_in_report(&self) -> bool {
        false
    }

    /// Whether the module results should trigger a failure status.
    fn raises_error(&self) -> bool;

    /// Whether the module results should trigger a warning status.
    fn raises_warning(&self) -> bool;

    fn calc(&mut self, _limits: &Limits) -> std::io::Result<()> {
        Ok(())
    }
    fn content_msg(&self) -> Option<String> {
        None
    }
    fn content_chart(&self) -> Option<(&str, String)> {
        None
    }
    fn content_table(&self) -> Option<String> {
        None
    }
}
