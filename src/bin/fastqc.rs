use clap::Parser;
use gene_platform::analytics::config::Config;
use gene_platform::analytics::qc::modules;
use gene_platform::analytics::runner;
use gene_platform::analytics::sequence::{Process, erase_process};
use gene_platform::utils::available_parallelism;
use std::path::PathBuf;
use std::time::Instant;

/// FastQC - A high throughput sequence QC analysis tool
#[derive(Parser, Debug)]
#[command(name = "fastqc", version, about)]
struct Cli {
    /// Total thread budget for the run [default: (available CPUs).min(20) ].
    /// Spread across the file processed simultaneously.
    #[arg(short, long, value_name = "N", value_parser = parse_threads)]
    threads: Option<usize>,

    /// Background gzip decoders per .fastq.gz file. The first counts towards
    /// --threads and keeps up with the analysis; any more do not count. 0
    /// decodes on the reading thread.
    #[arg(long = "decompress-threads", value_name = "N")]
    decompress_threads: Option<usize>,

    /// Gzip index file
    #[arg(short, long, value_name = "FILE")]
    index_file: Option<PathBuf>,

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

    let total_threads = cli
        .threads
        .unwrap_or_else(|| available_parallelism().min(20))
        .max(1);

    if let Some(ref threads) = cli.decompress_threads
        && total_threads <= *threads
    {
        eprintln!("threads should be above decompress_threads");
        return;
    }

    let config = Config {
        threads: total_threads,
        decompress_threads: cli.decompress_threads,
        index_file: cli.index_file,
    };

    let start = Instant::now();

    let processers = vec![
        erase_process(Box::new(modules::basic_stats::BasicStats::default())
            as Box<dyn Process<Value = modules::basic_stats::BasicStatsCounters>>),
        erase_process(
            Box::new(modules::per_base_quality::PerBaseQuality::default())
                as Box<dyn Process<Value = modules::per_base_quality::PerBaseQualityCounts>>,
        ),
        erase_process(
            Box::new(modules::per_sequence_quality::PerSequenceQuality::default())
                as Box<
                    dyn Process<Value = modules::per_sequence_quality::PerSequenceQualityScores>,
                >,
        ),
    ];

    runner::run(&config, cli.file, processers).expect("runner");

    let duration = start.elapsed();
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    println!("elapsed time: {:02}:{:02}", minutes, seconds);
}
