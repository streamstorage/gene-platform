use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Total thread budget for the run;
    pub threads: usize,
    /// Background gzip decoders per `.gz` file; `0` decodes on the reading thread.
    pub decompress_threads: Option<usize>,
    /// Processer/Reader ratio
    pub ratio: f64,
    /// gzip index file.
    pub index_file: Option<PathBuf>,
}
