use clap::Parser;
use gene_platform::utils::available_parallelism;
use rapidgzip_core::{Decoder, IndexOptions};
use std::fs::File;
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

    /// Input files (one FastQ, BAM, or SAM).
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

    let decoder = Decoder::builder()
        .decoder_threads(total_threads)
        .count_lines(true)
        .build()
        .expect("decoder");
    let source = File::open(&cli.file).expect("file");
    let name = cli.file.file_stem().expect("file name");
    let folder = cli.file.parent().expect("file folder");
    let mut index_file_name = std::ffi::OsString::new();
    index_file_name.push(name);
    index_file_name.push(".gzi");
    let index_file_path = folder.join(index_file_name);
    let mut index_file = File::create(index_file_path).expect("index file");

    let start = Instant::now();

    let report = decoder
        .decode_with_index(&source, &mut std::io::sink(), IndexOptions::default())
        .expect("report");

    report
        .index
        .write_native(&mut index_file)
        .expect("write index");
    println!(
        "total line: {}",
        report.index.total_line_count().unwrap_or(0)
    );

    let duration = start.elapsed();
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    println!("elapsed time: {:02}:{:02}", minutes, seconds);
}
