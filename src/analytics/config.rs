#[derive(Debug, Clone)]
pub struct Config {
    /// Total thread budget for the run;
    pub threads: usize,
    /// Background gzip decoders per `.gz` file; `0` decodes on the reading
    /// thread. The runner lowers the default `1` to `0` for a file given only
    /// one thread.
    pub decompress_threads: usize,
}
