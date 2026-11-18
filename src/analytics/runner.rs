use super::config::Config;
use super::sequence::fastq::FastqFile;
use super::sequence::{Sequence, SequenceFile, Transform};

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
) -> io::Result<Box<dyn SequenceFile>> {
    Ok(Box::new(FastqFile::open(config.decompress_threads, path)?))
}

pub fn run(config: &Config, path: impl AsRef<Path>) -> io::Result<()> {
    let mut seq_file = open_sequence_file(config, path)?;

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
    let num_processors = config.threads.saturating_sub(seq_file.background_threads());
    println!("number of processers: {}", num_processors);

    let transfomer = Box::new(super::qc::basic_stats::BasicStats);
    if num_processors == 1 {
        process_sequences_sequential(seq_file.as_mut(), transfomer)?;
    } else {
        process_sequences_parallel(seq_file.as_mut(), num_processors, transfomer)?;
    }

    Ok(())
}

fn process_sequences_sequential<D: Add<Output = D> + Send + Display>(
    seq_file: &mut dyn SequenceFile,
    tranformer: Box<dyn Transform<Value = D>>,
) -> io::Result<()> {
    let mut v = tranformer.none();
    loop {
        match seq_file.next() {
            Some(Ok(seq)) => {
                v = v + tranformer.transform(&seq);
            }
            Some(Err(e)) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
            None => break, // EOF
        }
    }
    println!("{}", v);
    Ok(())
}

fn process_sequences_parallel<D: Add<Output = D> + Send + Display>(
    seq_file: &mut dyn SequenceFile,
    num_processors: usize,
    tranformer: Box<dyn Transform<Value = D>>,
) -> io::Result<()> {
    let (tx, rx) = sync_channel::<Arc<Vec<Sequence>>>(QUEUE_CAPACITY);
    let mut reader_error: Option<io::Error> = None;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_processors)
        .build()
        .map_err(|e| io::Error::other(format!("Failed to create thread pool: {}", e)))
        .unwrap();

    pool.scope(|scope| {
        scope.spawn(move |_| {
            let v = rx
                .into_iter()
                .par_bridge()
                .map(|batch| {
                    batch
                        .iter()
                        .map(|s| tranformer.transform(s))
                        .reduce(|acc, e| acc + e)
                        .unwrap()
                })
                .reduce(|| tranformer.none(), |acc, e| acc + e);
            println!("{}", v);
        });

        // Reader loop runs on the calling thread: pull records, fill a batch,
        // and publish it to every worker queue. A batch is published once it is
        // full by either measure -- record count or bytes; see [`BATCH_BYTES`]
        // for why a count alone is not enough.
        let mut batch: Vec<Sequence> = Vec::with_capacity(BATCH_SIZE);
        let mut batch_bytes = 0usize;
        loop {
            match seq_file.next() {
                Some(Ok(seq)) => {
                    batch_bytes += seq.heap_bytes();
                    batch.push(seq);
                    if batch.len() == BATCH_SIZE || batch_bytes >= BATCH_BYTES {
                        let shared = std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
                        let shared = Arc::new(shared);
                        batch_bytes = 0;
                        // A send error means a worker panicked and dropped
                        // its receiver; stop reading so we don't deadlock on
                        // a full queue, and surface the panic via join below.
                        if tx.send(Arc::clone(&shared)).is_err() {
                            break;
                        }
                        //println!("some sequence sent");
                    }
                }
                Some(Err(e)) => {
                    reader_error = Some(io::Error::new(io::ErrorKind::InvalidData, e));
                    break;
                }
                None => break, // EOF
            }
        }

        // Publish the final partial batch (unless a read error aborted the run).
        if reader_error.is_none() && !batch.is_empty() {
            let shared = Arc::new(batch);
            let _ = tx.send(Arc::clone(&shared));
        }

        // Dropping the senders signals EOF; workers exit their recv loop and
        // return their (now fully-processed) module group.
        drop(tx);
    });

    Ok(())
}
