use super::config::Config;
use super::sequence::fastq::FastqFile;
use super::sequence::{ErasedProcess, ProcessedValue, Sequence, SequenceFile};

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::io;
use std::path::Path;
use std::sync::Arc;

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

pub fn run(
    config: &Config,
    path: impl AsRef<Path>,
    processers: Vec<Box<dyn ErasedProcess>>,
) -> io::Result<Vec<Box<dyn ProcessedValue>>> {
    let mut seq_files = open_sequence_file(config, path)?;
    if seq_files.is_empty() {
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
    let background_threads = seq_files.iter().map(|f| f.background_threads()).sum();
    let mut processer_threads = config.threads.saturating_sub(background_threads);
    if seq_files.len() > 1 {
        processer_threads -= seq_files.len();
    }

    println!("parallel of reading sequence file: {}", seq_files.len());
    println!("number of gzip background threads: {}", background_threads);
    println!(
        "number of sequence processing threads: {}",
        processer_threads
    );

    // Supported mode:
    // 1. sequencial read + process the whole file from begining to end
    // 2. divide file into multi sections, for each section, sequencial read + process on the same thread.
    // 3. divide file into multi sections, each section owns a dedicated channel, reader and processer. reader:processer = 1:1.
    // 4. divide file into multi sections, each section owns on reader but shares one mpmc channel, multiple processer on different threads. reader:processer = M:N.
    // TODO:
    // 1. divide file into multi sections, each section owns on reader but shares one mpsc channel and process on one thread. reader:processer = M:1.
    let v = if seq_files.len() == 1 && processer_threads == 1 {
        process_sequences_sequential(seq_files[0].as_mut(), processers)?
    } else {
        process_sequences_parallel(&mut seq_files, processer_threads, processers)?
    };
    Ok(v)
}

fn process_sequences_sequential(
    seq_file: &mut dyn SequenceFile,
    mut processers: Vec<Box<dyn ErasedProcess>>,
) -> io::Result<Vec<Box<dyn ProcessedValue>>> {
    loop {
        match seq_file.next() {
            Some(Ok(seq)) => {
                processers.iter_mut().for_each(|p| p.process(&seq));
            }
            Some(Err(e)) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
            None => break, // EOF
        }
    }
    Ok(processers.iter().map(|p| p.ref_into()).collect())
}

fn process_sequences_parallel(
    seq_files: &mut Vec<Box<dyn SequenceFile>>,
    processer_threads: usize,
    processers: Vec<Box<dyn ErasedProcess>>,
) -> io::Result<Vec<Box<dyn ProcessedValue>>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(seq_files.len())
        .build()
        .map_err(|e| io::Error::other(format!("Failed to create thread pool: {}", e)))?;

    // // For read speed boundary detection only.
    // // OED00091183_L99A_1.fq.gz 6.7G / OED00091183_L99A_1.fq.gzi 22M
    // // 10 sections: 23s
    // // 20 sections: 15s
    // // 30 sections: 12s
    // // 40 sections: 12s
    // let v = pool.install(|| {
    //     seq_files
    //         .par_iter_mut()
    //         .map(|seq_file| {
    //             while let Some(_seq) = seq_file.next() {
    //             }
    //             let p = processers.clone();
    //             Ok(p
    //                 .iter()
    //                 .map(|p| p.ref_into())
    //                 .collect::<Vec<Box<dyn ProcessedValue>>>())
    //         })
    //         .try_reduce_with(|left, right| {
    //             left.into_iter().zip(right).map(|(a, b)| a.add(b)).collect()
    //         })
    //         .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::InvalidData, "empty")))
    // })?;

    // let v = pool.install(|| {
    //     seq_files
    //         .par_iter_mut()
    //         .map(|seq_file| {
    //             let p = processers.clone();
    //             process_sequences_sequential(seq_file.as_mut(), p)
    //         })
    //         .try_reduce_with(|left, right| {
    //             left.into_iter().zip(right).map(|(a, b)| a.add(b)).collect()
    //         })
    //         .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::InvalidData, "empty")))
    // })?;

    // let v = pool.install(|| {
    //     seq_files
    //         .par_iter_mut()
    //         .map(|seq_file| {
    //             let (tx, rx) = std::sync::mpsc::sync_channel::<Arc<Vec<Sequence>>>(QUEUE_CAPACITY);

    //             let mut processers = processers.clone();
    //             let mut processer_refs: Vec<&mut Box<dyn ErasedProcess>> =
    //                 processers.iter_mut().collect();

    //             std::thread::scope(move |scope| {
    //                 scope.spawn(move || {
    //                     while let Ok(seqs) = rx.recv() {
    //                         for seq in seqs.iter() {
    //                             for processer in processer_refs.iter_mut() {
    //                                 (*processer).process(seq);
    //                             }
    //                         }
    //                     }
    //                 });

    //                 Reader loop runs on the calling thread: pull records, fill a batch,
    //                 and publish it to every worker queue. A batch is published once it is
    //                 full by either measure -- record count or bytes; see [`BATCH_BYTES`]
    //                 for why a count alone is not enough.
    //                 let mut batch: Vec<Sequence> = Vec::with_capacity(BATCH_SIZE);
    //                 let mut batch_bytes = 0usize;
    //                 let mut reader_error: Option<io::Error> = None;
    //                 loop {
    //                     match seq_file.next() {
    //                         Some(Ok(seq)) => {
    //                             batch_bytes += seq.heap_bytes();
    //                             batch.push(seq);
    //                             if batch.len() == BATCH_SIZE || batch_bytes >= BATCH_BYTES {
    //                                 let shared = std::mem::replace(
    //                                     &mut batch,
    //                                     Vec::with_capacity(BATCH_SIZE),
    //                                 );
    //                                 let shared = Arc::new(shared);
    //                                 batch_bytes = 0;
    //                                 A send error means a worker panicked and dropped
    //                                 its receiver; stop reading so we don't deadlock on
    //                                 a full queue, and surface the panic via join below.
    //                                 if tx.send(Arc::clone(&shared)).is_err() {
    //                                     break;
    //                                 }
    //                             }
    //                         }
    //                         Some(Err(e)) => {
    //                             reader_error = Some(io::Error::new(io::ErrorKind::InvalidData, e));
    //                             break;
    //                         }
    //                         None => break, // EOF
    //                     }
    //                 }

    //                 Publish the final partial batch (unless a read error aborted the run).
    //                 if reader_error.is_none() && !batch.is_empty() {
    //                     let shared = Arc::new(batch);
    //                     let _ = tx.send(Arc::clone(&shared));
    //                 }

    //                 Dropping the senders signals EOF; workers exit their recv loop and
    //                 return their (now fully-processed) module group.
    //                 drop(tx);
    //             });

    //             Ok(processers
    //                 .iter()
    //                 .map(|p| p.ref_into())
    //                 .collect::<Vec<Box<dyn ProcessedValue>>>())
    //         })
    //         .try_reduce_with(|left, right| {
    //             left.into_iter().zip(right).map(|(a, b)| a.add(b)).collect()
    //         })
    //         .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::InvalidData, "empty")))
    // })?;

    let processer_pool = ThreadPoolBuilder::new()
        .num_threads(processer_threads)
        .build()
        .map_err(|e| io::Error::other(format!("Failed to create read thread pool: {}", e)))?;
    let threads: Vec<usize> = (1..=processer_threads).collect();

    let (tx, rx) = crossbeam_channel::bounded::<Arc<Vec<Sequence>>>(QUEUE_CAPACITY);

    let v = pool.scope(|scope| {
        for seq_file in seq_files {
            let tx = tx.clone();
            scope.spawn(move |_| {
                // Reader loop runs on the calling thread: pull records, fill a batch,
                // and publish it to every worker queue. A batch is published once it is
                // full by either measure -- record count or bytes; see [`BATCH_BYTES`]
                // for why a count alone is not enough.
                let mut batch: Vec<Sequence> = Vec::with_capacity(BATCH_SIZE);
                let mut batch_bytes = 0usize;
                let mut reader_error: Option<io::Error> = None;
                loop {
                    match seq_file.next() {
                        Some(Ok(seq)) => {
                            batch_bytes += seq.heap_bytes();
                            batch.push(seq);
                            if batch.len() == BATCH_SIZE || batch_bytes >= BATCH_BYTES {
                                let shared =
                                    std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
                                let shared = Arc::new(shared);
                                batch_bytes = 0;
                                // A send error means a worker panicked and dropped
                                // its receiver; stop reading so we don't deadlock on
                                // a full queue, and surface the panic via join below.
                                if tx.send(Arc::clone(&shared)).is_err() {
                                    break;
                                }
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
        }
        drop(tx);

        processer_pool.install(|| {
            threads
                .par_iter()
                .map(|_| {
                    let mut processers = processers.clone();
                    let mut processer_refs: Vec<&mut Box<dyn ErasedProcess>> =
                        processers.iter_mut().collect();
                    let rx = rx.clone();

                    while let Ok(seqs) = rx.recv() {
                        for seq in seqs.iter() {
                            for processer in processer_refs.iter_mut() {
                                (*processer).process(seq);
                            }
                        }
                    }

                    Ok(processers
                        .iter()
                        .map(|p| p.ref_into())
                        .collect::<Vec<Box<dyn ProcessedValue>>>())
                })
                .try_reduce_with(|left, right| {
                    left.into_iter().zip(right).map(|(a, b)| a.add(b)).collect()
                })
                .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::InvalidData, "empty")))
        })
    })?;

    Ok(v)
}
