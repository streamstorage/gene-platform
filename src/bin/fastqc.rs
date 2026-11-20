use clap::Parser;
use gene_platform::analytics::config::Config;
use gene_platform::analytics::runner;
use gene_platform::utils::available_parallelism;
use std::path::PathBuf;
use std::time::Instant;

/// FastQC - A high throughput sequence QC analysis tool
#[derive(Parser, Debug)]
#[command(name = "fastqc", version, about)]
struct Cli {
    /// Total thread budget for the run [default: available CPUs, up to 6].
    /// Spread across the files processed simultaneously and then within each
    /// file (a reader, its gzip decoder, and analysis workers).
    #[arg(short, long, value_name = "N", value_parser = parse_threads)]
    threads: Option<usize>,

    /// Background gzip decoders per .fastq.gz file. The first counts towards
    /// --threads and keeps up with the analysis; any more do not count. 0
    /// decodes on the reading thread, parallel will be ignored.
    #[arg(long = "decompress-threads", value_name = "N", default_value = "1")]
    decompress_threads: usize,

    #[arg(short, long, value_name = "FILE")]
    index_file: Option<PathBuf>,

    #[arg(short, long, value_name = "N", value_parser = parse_threads, default_value = "1")]
    parallel: usize,

    /// Input file (one FastQ, BAM, or SAM file).
    #[arg(required = true)]
    file: PathBuf,
}

fn parse_threads(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(0) => Err("must be at least 1".to_string()),
        Ok(n) => Ok(n),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    let cli = Cli::parse();

    let total_threads = cli.threads.unwrap_or_else(available_parallelism).max(1);

    let config = Config {
        threads: total_threads,
        decompress_threads: cli.decompress_threads,
        index_file: cli.index_file,
        parallel: cli.parallel,
    };

    let start = Instant::now();
    runner::run(&config, cli.file).expect("runner");
    let duration = start.elapsed();
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    println!("elapsed time: {:02}:{:02}", minutes, seconds);
}
