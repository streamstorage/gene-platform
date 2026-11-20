use super::config::Config;
use super::sequence::fastq::FastqFile;
use super::sequence::{Process, Sequence, SequenceFile};

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::fmt::Display;
use std::io;
use std::ops::Add;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;

/// Bounded capacity of each processor's batch queue. Provides backpressure so the
/// reader can run at most this many batches ahead of the slowest processor,
/// keeping peak memory bounded. Matches the Java pipeline's queue capacity.
const QUEUE_CAPACITY: usize = 32;

/// Number of sequences the reader batches before publishing to the processors.
/// Matches the Java pipeline's batch size. Large enough to amortise channel and
/// Arc overhead, small enough to keep the processors fed and memory bounded.
const BATCH_SIZE: usize = 1024;

/// Byte ceiling on a single batch, applied alongside [`BATCH_SIZE`].
///
/// A record count alone does not bound memory: at [`QUEUE_CAPACITY`] batches of
/// [`BATCH_SIZE`] records the pipeline holds ~32k records however big they are,
/// which is a few MB of Illumina reads but gigabytes of long reads, per file
/// analysed concurrently.
///
/// Capping the bytes too holds the pipeline to roughly
/// `QUEUE_CAPACITY * BATCH_BYTES` per file whatever the read length. Short
/// reads never reach the cap (1024 x ~250 B is well under it).
const BATCH_BYTES: usize = 1024 * 1024;

fn open_sequence_file(
    config: &Config,
    path: impl AsRef<Path>,
) -> io::Result<Vec<Box<dyn SequenceFile>>> {
    let files = FastqFile::open(config, path)?;
    Ok(files
        .into_iter()
        .map(|f| Box::new(f) as Box<dyn SequenceFile>)
        .collect())
}

pub fn run(config: &Config, path: impl AsRef<Path>) -> io::Result<()> {
    let mut seq_files = open_sequence_file(config, path)?;
    if seq_files.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no sequence files",
        ));
    }

    // Feed every sequence through all modules. With a budget of one thread we
    // take the original single-threaded path (byte-identical, no pipeline
    // overhead); otherwise a reader thread batches records and hands them to
    // worker threads that each own a disjoint subset of the modules. Either way
    // each module sees every sequence exactly once, in file order, on a single
    // thread, so the accumulated state -- and the resulting report -- is
    // identical regardless of the thread budget.
    //
    // There is no point in more workers than there are modules, so cap the
    // worker count by the number of active modules.
    let num_processors = config
        .threads
        .saturating_sub(seq_files.iter().map(|f| f.background_threads()).sum());
    println!("number of processors: {}", num_processors);

    let stats = super::qc::basic_stats::BasicStats::default();
    let mut processer =
        Box::new(stats) as Box<dyn Process<Value = super::qc::basic_stats::BasicStatsCounters>>;

    // Current implementation:
    // 1. sequencial read + process
    // 2. divide file into multi parts, for each part, sequencial read + process on different thread and reduce on main thread.
    // TODO:
    // 1. divide file into multi part, for each part, read on on different thread, and send to one mpsc channel, and process on one thread
    // 2. divide file into multi part, for each part, read on on different thread, and send to one mpmc channel, and process on different thread
    // 3. divide file into multi part, for each part, read on on different thread, and send to multi spsc channel, and process on different thread
    if num_processors == 1 && seq_files.len() == 1 {
        process_sequences_sequential(seq_files[0].as_mut(), &mut processer)?;
    } else {
        process_sequences_parallel(&mut seq_files, num_processors, processer)?;
    }

    Ok(())
}

fn process_sequences_sequential<D: Add<Output = D> + Send + Display>(
    seq_file: &mut dyn SequenceFile,
    processer: &mut Box<dyn Process<Value = D>>,
) -> io::Result<D> {
    loop {
        match seq_file.next() {
            Some(Ok(seq)) => {
                processer.process(&seq);
            }
            Some(Err(e)) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
            None => break, // EOF
        }
    }
    Ok(processer.mut_into())
}

fn process_sequences_parallel<D: Add<Output = D> + Send + Display>(
    seq_files: &mut Vec<Box<dyn SequenceFile>>,
    _num_processors: usize,
    processer: Box<dyn Process<Value = D>>,
) -> io::Result<()> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(seq_files.len())
        .build()
        .map_err(|e| io::Error::other(format!("Failed to create thread pool: {}", e)))?;

    let v = pool.install(|| {
        seq_files
            .par_iter_mut()
            .map(|seq_file| {
                let mut p = processer.clone();
                process_sequences_sequential(seq_file.as_mut(), &mut p)
            })
            .try_reduce_with(|acc, item| Ok(acc + item))
            .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::InvalidData, "empty")))
    })?;
    println!("{}", v);

    // let _ = pool.scope(move |scope| {
    //     let (tx, rx) = sync_channel::<Arc<Vec<Sequence>>>(QUEUE_CAPACITY);
    //     scope.spawn( move |_| {
    //         while let Ok(_) = rx.recv() {
    //             //println!("sequence received at {}", p_id);
    //         }
    //     });

    //     let v = seq_files
    //         .par_iter_mut()
    //         .map(|seq_file| {
    //             let tx = tx.clone();

    //             // Reader loop runs on the calling thread: pull records, fill a batch,
    //             // and publish it to every worker queue. A batch is published once it is
    //             // full by either measure -- record count or bytes; see [`BATCH_BYTES`]
    //             // for why a count alone is not enough.
    //             let mut batch: Vec<Sequence> = Vec::with_capacity(BATCH_SIZE);
    //             let mut batch_bytes = 0usize;
    //             let mut reader_error: Option<io::Error> = None;
    //             loop {
    //                 match seq_file.next() {
    //                     Some(Ok(seq)) => {
    //                         batch_bytes += seq.heap_bytes();
    //                         batch.push(seq);
    //                         if batch.len() == BATCH_SIZE || batch_bytes >= BATCH_BYTES {
    //                             let shared = std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
    //                             let shared = Arc::new(shared);
    //                             batch_bytes = 0;
    //                             // A send error means a worker panicked and dropped
    //                             // its receiver; stop reading so we don't deadlock on
    //                             // a full queue, and surface the panic via join below.
    //                             if tx.send(Arc::clone(&shared)).is_err() {
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                     Some(Err(e)) => {
    //                         reader_error = Some(io::Error::new(io::ErrorKind::InvalidData, e));
    //                         break;
    //                     }
    //                     None => break, // EOF
    //                 }
    //             }

    //             // Publish the final partial batch (unless a read error aborted the run).
    //             if reader_error.is_none() && !batch.is_empty() {
    //                 let shared = Arc::new(batch);
    //                 let _ = tx.send(Arc::clone(&shared));
    //             }

    //             // Dropping the senders signals EOF; workers exit their recv loop and
    //             // return their (now fully-processed) module group.
    //             drop(tx);
    //             1
    //         }).reduce(|| 0 , |acc, item| acc+ item);
    //     println!("{}", v);
    // });

    Ok(())
}
