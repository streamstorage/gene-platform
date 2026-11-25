use clap::Parser;
use gene_platform::analytics::config::Config;
use gene_platform::analytics::qc::config::{load_adapters, load_limits};
use gene_platform::analytics::qc::{modules, report};
use gene_platform::analytics::runner;
use gene_platform::analytics::sequence::{Process, erase_process, strip_extensions};
use gene_platform::utils::available_parallelism;
use std::path::PathBuf;
use std::time::Instant;

/// fastqc - A high throughput sequence QC analysis tool
#[derive(Parser, Debug)]
#[command(name = "fastqc", version, about)]
struct Cli {
    /// Total thread budget for the run [default: available CPUs ].
    #[arg(short, long, value_name = "N", value_parser = parse_threads)]
    threads: Option<usize>,

    /// Background gzip decoders for .fastq.gz file. If no the index file specified,
    /// it will be 1 by default, 0 decodes on the reading thread.
    /// If index file is specified, it is better to leave this unset.
    #[arg(long = "decompress-threads", value_name = "N")]
    decompress_threads: Option<usize>,

    /// Specifies processers and file readers ratio
    #[arg(short, long, value_name = "N", default_value = "1.0")]
    ratio: f64,

    /// Specifies the truncation length used for duplicate detection.
    /// Reads longer than this value will be truncated before checking for duplicates.
    // Java uses --dup_length (underscore)
    #[arg(
        long = "dup_length",
        alias = "dup-length",
        value_name = "N",
        default_value = "0"
    )]
    dup_length: usize,

    /// Specifies a non-default file which contains the list of contaminants to
    /// screen overrepresented sequences against.
    #[arg(short = 'c', long, value_name = "FILE")]
    contaminants: Option<PathBuf>,

    /// Specifies a non-default file which contains the list of adapter sequences
    /// which will be explicitly searched against the library.
    #[arg(short, long, value_name = "FILE")]
    adapters: Option<PathBuf>,

    /// Specifies a non-default file which contains a set of criteria used to
    /// determine the warn/error limits for the various modules.
    #[arg(short, long, value_name = "FILE")]
    limits: Option<PathBuf>,

    /// Specify the gzip index file
    #[arg(short, long, value_name = "FILE")]
    index_file: Option<PathBuf>,

    /// Input file (one FastQ, or FastQ.gz file).
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

    if let Some(ref threads) = cli.decompress_threads
        && total_threads <= *threads
    {
        eprintln!("threads should be above decompress_threads.");
        return;
    }

    if cli.ratio <= 0. {
        eprintln!("radio should be above 0.");
        return;
    }

    let config = Config {
        threads: total_threads,
        decompress_threads: cli.decompress_threads,
        ratio: cli.ratio,
        index_file: cli.index_file,
    };

    let start = Instant::now();

    let adapters = load_adapters(cli.adapters).expect("adapters");
    let processers = vec![
        erase_process(Box::new(modules::basic_stats::BasicStatistics::default())
            as Box<dyn Process<Value = modules::basic_stats::StatisticsCounters>>),
        erase_process(
            Box::new(modules::per_base_quality::PerBaseSequenceQuality::default())
                as Box<dyn Process<Value = modules::per_base_quality::PerBaseQualityCounts>>,
        ),
        erase_process(
            Box::new(modules::per_sequence_quality::PerSequenceQualityScores::default())
                as Box<dyn Process<Value = modules::per_sequence_quality::QualityScores>>,
        ),
        erase_process(
            Box::new(modules::per_base_content::PerBaseSequenceContent::default())
                as Box<dyn Process<Value = modules::per_base_content::SequenceContent>>,
        ),
        erase_process(
            Box::new(modules::gc_content::PerSequenceGCContent::default())
                as Box<dyn Process<Value = modules::gc_content::GCContent>>,
        ),
        erase_process(Box::new(modules::n_content::PerBaseNContent::default())
            as Box<dyn Process<Value = modules::n_content::NContent>>),
        erase_process(
            Box::new(modules::length_distribution::SequenceLengthDistribution::default())
                as Box<dyn Process<Value = modules::length_distribution::LengthDistribution>>,
        ),
        erase_process(
            Box::new(modules::adapter_content::AdapterContent::new(&adapters))
                as Box<dyn Process<Value = modules::adapter_content::AdapterContentData>>,
        ),
    ];

    let file_name = cli
        .file
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned();
    let file_folder = cli.file.parent().expect("file folder");
    let base_name = strip_extensions(&file_name);

    let html_path = file_folder.join(format!("{}_pgx_fastqc.html", base_name));

    let mut values = runner::run(&config, cli.file, processers).expect("runner");
    let basic_statistics = values
        .remove(0)
        .into_any()
        .downcast::<modules::basic_stats::StatisticsCounters>()
        .expect("cast StatisticsCounters");
    let per_base_sequence_quality = values
        .remove(0)
        .into_any()
        .downcast::<modules::per_base_quality::PerBaseQualityCounts>()
        .expect("cast PerBaseQualityCounts");
    let per_sequence_quality_scores = values
        .remove(0)
        .into_any()
        .downcast::<modules::per_sequence_quality::QualityScores>()
        .expect("cast QualityScores");
    let per_base_sequence_count = values
        .remove(0)
        .into_any()
        .downcast::<modules::per_base_content::SequenceContent>()
        .expect("cast QualityScores");

    let adapter_content = values
        .pop()
        .unwrap()
        .into_any()
        .downcast::<modules::adapter_content::AdapterContentData>()
        .expect("cast AdapterContentData");

    let limits = load_limits(cli.limits).expect("limits");
    let mut modules = [
        basic_statistics as Box<dyn modules::QCModule>,
        per_base_sequence_quality as Box<dyn modules::QCModule>,
        per_sequence_quality_scores as Box<dyn modules::QCModule>,
        per_base_sequence_count as Box<dyn modules::QCModule>,

        adapter_content as Box<dyn modules::QCModule>,
    ];
    for module in modules.iter_mut() {
        module.calc(&limits).expect("calc");
    }

    let html_content =
        report::html::generate_html_report(&modules, &file_name).expect("generate report");
    std::fs::write(&html_path, &html_content).expect("write report");

    let duration = start.elapsed();
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    println!("elapsed time: {:02}:{:02}", minutes, seconds);
}
