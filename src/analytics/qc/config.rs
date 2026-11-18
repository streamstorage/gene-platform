use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

/// A parsed limit entry from limits.txt.
/// The key is "{module}\t{level}" (e.g. "duplication\twarn"), value is the threshold.
///
/// The Java ModuleConfig stores limits as a nested HashMap keyed on
/// module name and then level (warn/error/ignore). We flatten into a single HashMap
/// with a composite key of "module\tlevel" to simplify lookups while keeping the
/// same data accessible.
pub type Limits = HashMap<String, f64>;

/// Extension methods for the `Limits` type to reduce boilerplate in modules.
pub trait LimitsExt {
    /// Get a threshold value for a module/level key, returning a default if not set.
    ///
    /// Replaces the common pattern:
    ///   `self.limits.get("module\tlevel").copied().unwrap_or(default)`
    fn threshold(&self, key: &str, default: f64) -> f64;

    /// Check whether a module is configured to be ignored (ignore value > 0).
    ///
    /// Replaces the common pattern:
    ///   `self.limits.get("module\tignore").copied().unwrap_or(0.0) > 0.0`
    fn is_ignored(&self, module: &str) -> bool;

    /// Check whether a module should be created (not configured to be ignored).
    ///
    /// Replaces the common pattern in create_modules():
    ///   `limits.get("module\tignore").map_or(true, |&v| v == 0.0)`
    fn is_module_enabled(&self, module: &str) -> bool;
}

impl LimitsExt for Limits {
    fn threshold(&self, key: &str, default: f64) -> f64 {
        self.get(key).copied().unwrap_or(default)
    }

    fn is_ignored(&self, module: &str) -> bool {
        let key = format!("{}\tignore", module);
        self.get(&key).copied().unwrap_or(0.0) > 0.0
    }

    fn is_module_enabled(&self, module: &str) -> bool {
        let key = format!("{}\tignore", module);
        self.get(&key).is_none_or(|&v| v == 0.0)
    }
}

/// Parse a limits.txt formatted string into a Limits map.
///
/// Whitespace-separated fields. The Java code splits on arbitrary
/// whitespace (tabs/spaces), so we do the same with split_whitespace().
fn parse_limits(text: &str) -> Limits {
    let mut limits = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            let module = parts[0];
            let level = parts[1]; // "warn", "error", or "ignore"
            if let Ok(value) = parts[2].parse::<f64>() {
                let key = format!("{}\t{}", module, level);
                limits.insert(key, value);
            }
        }
    }
    limits
}

/// Configuration for a FastQC run, mirroring all fields from Java FastQCConfig.
#[derive(Debug, Clone)]
pub struct Config {
    pub limits_file: Option<PathBuf>,
}

impl Config {
    pub fn load_limits(&self) -> io::Result<Limits> {
        let text = match &self.limits_file {
            Some(path) => fs::read_to_string(path)?,
            None => include_str!("./limits.txt").to_string(),
        };
        Ok(parse_limits(&text))
    }
}
