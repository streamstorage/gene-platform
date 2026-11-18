// FASTQ file reader
// Corresponds to Sequence/FastQFile.java
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// NOTE: rapidgzip's reader is referred to by its full path
// (`rapidgzip_core::DecoderReader`) below because the name `DecoderReader` is
// already taken by the bzip2 reader imported above.

use super::{Sequence, SequenceFile};

// ---------------------------------------------------------------------------
// Decompression layer
// ---------------------------------------------------------------------------

/// Wrapper enum so we can store different reader types without trait objects.
/// Each variant wraps a `BufReader` around the appropriate decompression stream.
enum FastqReader {
    Plain(BufReader<File>, File),
    /// Parallel, multi-member gzip decompression via rapidgzip. The heavy
    /// lifting (inflate on a pool of background threads) happens behind a
    /// `Read + Send` handle, so it looks like any other buffered reader here.
    Gzip(Box<BufReader<rapidgzip_core::DecoderReader>>, GzipProgress),
}

/// Large so gzip refills (a decoder-thread handoff) are rare.
const READ_BUFFER: usize = 1 << 20;

impl BufRead for FastqReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            FastqReader::Plain(r, _) => r.fill_buf(),
            FastqReader::Gzip(r, _) => r.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            FastqReader::Plain(r, _) => r.consume(amt),
            FastqReader::Gzip(r, _) => r.consume(amt),
        }
    }
}

impl Read for FastqReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            FastqReader::Plain(r, _) => r.read(buf),
            FastqReader::Gzip(r, _) => r.read(buf),
        }
    }
}

impl FastqReader {
    pub fn progress(&self, file_size: u64) -> f64 {
        match self {
            // bzip2 / plain text read on this thread: seek a cloned file handle for the
            // compressed byte position, exactly as Java queries
            // `fis.getChannel().position()`.
            FastqReader::Plain(r, p) => {
                let buffered = r.buffer().len() as u64;
                let mut handle: &File = p;
                handle
                    .stream_position()
                    .map(|pos| (pos.saturating_sub(buffered) as f64 / file_size as f64) * 100.0)
                    .unwrap_or(0.0)
            }
            // gzip via rapidgzip. See [`GzipProgress`].
            FastqReader::Gzip(r, p) => {
                let buffered = r.buffer().len() as u64;
                p.percent(file_size, buffered).unwrap_or(0.0)
            }
        }
    }
}

/// Largest compression ratio a gzip trailer may imply before it is treated as
/// untrustworthy. FASTQ compresses around 3-5x; anything past this is a
/// truncated `ISIZE` rather than a real ratio.
const MAX_PLAUSIBLE_RATIO: u64 = 1000;

/// gzip records the uncompressed size modulo this, so a file larger than it
/// wraps and the trailer has to be unwrapped.
const ISIZE_MODULUS: u64 = 1 << 32;

/// Progress tracking for the gzip decoder.
///
/// The obvious measure — compressed bytes served to the decoder over the file
/// size — is badly wrong here, because rapidgzip's workers read *far* ahead of
/// what the parser has consumed. Measured on four cores, that estimate opens a
/// 99 MB `.gz` at 22%, reaches 100% at the halfway mark and sits there for the
/// rest of the run; a 16 MB `.gz` is pinned at 100% from the very first update.
/// The read-ahead is roughly a fixed window, so the smaller the file the more
/// useless the number.
///
/// So progress is measured on the *output* side instead: `consumed_bytes`, the
/// decompressed bytes the decoder has actually handed to the parser, which by
/// construction never runs ahead of the analysis. What that needs is a total to
/// divide by, and there are two ways to get one.
///
/// **The gzip trailer**, when it is trustworthy, is exact and free — see
/// [`gzip_trailer_size`]. Every `gzip`/`pigz`-compressed `.fastq.gz` is a
/// single member and lands here. Past 4 GiB the trailer has wrapped, and the
/// ratio estimate below picks which wrap it is — see [`unwrap_isize`].
///
/// **Otherwise** — concatenated members, BGZF — fall back to scaling the file
/// size by the ratio the decoder has achieved so far. That ratio is an
/// underestimate while the read-ahead is outstanding (the compressed tally is
/// ahead of the decompressed one) and converges up to the true ratio by EOF, so
/// the running maximum is taken and the bar can run ahead and stall near the
/// end.
///
/// The trailer only describes the *last* member, and a concatenated file
/// (`cat L001.gz L002.gz`) looks single-member until the decoder finishes the
/// first one. The ratio estimate catches that early: it only ever errs low, so
/// once it clearly exceeds the trailer the trailer is dropped for good.
struct GzipProgress {
    /// Furthest compressed offset the decoder has read, from [`CountingSource`].
    compressed: Arc<AtomicU64>,
    /// Lock-free telemetry from the decoder: decompressed bytes produced, and
    /// decompressed bytes actually returned through `Read`.
    handle: rapidgzip_core::DecoderHandle,
    /// Contended once per [`PROGRESS_INTERVAL`](crate::runner) records at most.
    estimate: std::sync::Mutex<GzipEstimate>,
}

#[derive(Default)]
struct GzipEstimate {
    /// Running maximum of the ratio-scaled size estimate.
    ratio_total: u64,
    /// Total decompressed size from the gzip trailer while it is still
    /// believed; taken once the file turns out to have several members.
    trailer: Option<u64>,
}

impl GzipProgress {
    /// Progress through the file as a percentage, or `None` before the decoder
    /// has produced enough to estimate a total. `buffered` is decompressed
    /// output already read but not yet parsed.
    fn percent(&self, file_size: u64, buffered: u64) -> Option<f64> {
        if file_size == 0 {
            return None;
        }
        let stats = self.handle.stats();
        let parsed = stats.consumed_bytes.saturating_sub(buffered);
        // The decoder hands a chunk over before counting it, so the reader can
        // briefly be ahead of `decompressed_bytes`.
        let produced = stats.decompressed_bytes.max(stats.consumed_bytes);
        let compressed = self.compressed.load(Ordering::Relaxed);
        if compressed == 0 || produced == 0 {
            return None;
        }

        let mut estimate = self
            .estimate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let ratio = produced as f64 / compressed as f64;
        estimate.ratio_total = estimate
            .ratio_total
            .max((file_size as f64 * ratio) as u64)
            .max(produced);

        // Output is emitted in member order, so once a member has completed,
        // output that no longer matches the trailer must be a later member's.
        if let Some(trailer) = estimate.trailer {
            let past_first_member = match stats.member_count {
                0 => false,
                1 => produced % ISIZE_MODULUS != trailer,
                _ => true,
            };
            // The quarter is headroom for the ratio varying along the file.
            let wrapped = unwrap_isize(trailer, 0, estimate.ratio_total);
            if past_first_member || estimate.ratio_total > wrapped + wrapped / 4 {
                estimate.trailer = None;
            }
        }
        let total = match estimate.trailer {
            Some(trailer) => unwrap_isize(trailer, parsed, estimate.ratio_total),
            None => estimate.ratio_total,
        };
        if total == 0 {
            return None;
        }

        Some((parsed as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
    }
}

/// The true uncompressed size given a trailer recorded modulo 4 GiB: the
/// candidate `trailer + k * 4 GiB` nearest the `hint` (the ratio estimate),
/// and never less than the `parsed` bytes already seen.
///
/// The hint only has to be within 2 GiB of the truth to pick the right wrap,
/// which a ratio measured over the first few hundred MB comfortably is.
fn unwrap_isize(trailer: u64, parsed: u64, hint: u64) -> u64 {
    let nearest = (hint.saturating_sub(trailer) + ISIZE_MODULUS / 2) / ISIZE_MODULUS;
    let at_least = parsed.saturating_sub(trailer).div_ceil(ISIZE_MODULUS);
    trailer + nearest.max(at_least) * ISIZE_MODULUS
}

/// The uncompressed size recorded in a gzip file's `ISIZE` trailer (the last
/// four bytes, little-endian), when it is worth believing.
///
/// Exact for a single-member file, which is what `gzip` and `pigz` produce and
/// so what almost every `.fastq.gz` is. Sizes over 4 GiB wrap, which
/// [`unwrap_isize`] undoes.
///
/// For concatenated members the trailer describes only the *last* one, so it
/// has to be rejected: requiring the implied ratio to be at least 1:1 catches
/// that (a trailing member is a small fraction of the file, and BGZF's empty
/// end-of-file member records zero), as well as the pathological case of a
/// stored-not-deflated stream. A trailing member that is still larger than the
/// file gets past this, and is caught by [`GzipProgress::percent`].
/// [`MAX_PLAUSIBLE_RATIO`] catches a truncated file whose
/// last four bytes are not a trailer at all.
fn gzip_trailer_size(path: &Path, file_size: u64) -> Option<u64> {
    // Smaller than the smallest possible member: header, empty deflate block,
    // trailer.
    if file_size < 20 {
        return None;
    }
    let mut file = File::open(path).ok()?;
    file.seek(io::SeekFrom::End(-4)).ok()?;
    let mut trailer = [0u8; 4];
    file.read_exact(&mut trailer).ok()?;
    let total = u32::from_le_bytes(trailer) as u64;
    (total >= file_size && total <= file_size.saturating_mul(MAX_PLAUSIBLE_RATIO)).then_some(total)
}

// ---------------------------------------------------------------------------
// Compression detection
// ---------------------------------------------------------------------------

/// Detect compression from the first two bytes (magic numbers).
/// Returns "gz", "bz2", or "none".
fn detect_compression_from_magic(path: impl AsRef<Path>) -> io::Result<&'static str> {
    let mut f = File::open(path)?;
    // Sniffing a pipe would consume the bytes the reader then needs.
    if !f.metadata()?.is_file() {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "not a file"));
    }
    let mut magic = [0u8; 2];
    let n = f.read(&mut magic)?;
    if n >= 2 {
        // Java detects gzip via file extension or MIME type probing which
        // checks magic bytes 1f 8b internally. We replicate by checking magic directly.
        if magic[0] == 0x1f && magic[1] == 0x8b {
            return Ok("gz");
        }
        // Java only checks .bz2 extension, but we also check magic bytes
        // 42 5a ('BZ') for robustness.
        if magic[0] == 0x42 && magic[1] == 0x5a {
            return Ok("bz2");
        }
    }
    Ok("none")
}

// ---------------------------------------------------------------------------
// gzip decompression (parallel & multi-member, via rapidgzip)
// ---------------------------------------------------------------------------

/// A gzip source that tracks how far into the file the decoder has read, so the
/// FASTQ reader can report progress while rapidgzip owns the file. Read
/// positionally by the parallel decoder, or sequentially when decoding on the
/// reading thread. The decoder re-reads some ranges, so a sum of bytes served
/// would overcount.
struct CountingSource {
    file: File,
    counter: Arc<AtomicU64>,
}

impl rapidgzip_core::ReadAt for CountingSource {
    fn len(&self) -> io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        let read = rapidgzip_core::ReadAt::read_at(&self.file, offset, buffer)?;
        self.counter
            .fetch_max(offset + read as u64, Ordering::Relaxed);
        Ok(read)
    }
}

impl Read for CountingSource {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.file.read(buffer)?;
        self.counter.fetch_add(read as u64, Ordering::Relaxed);
        Ok(read)
    }
}

/// A gzip file opened through rapidgzip.
struct OpenedGzip {
    reader: rapidgzip_core::DecoderReader,
    progress: GzipProgress,
    /// Threads decoding in the background, beside the one reading.
    background_threads: usize,
}

/// Open a gzip file through rapidgzip.
///
/// `threads` decoders run in the background on a regular file. With
/// `threads == 0`, or a FIFO or other non-regular file that the positional
/// decoder cannot read, the stream is decoded on the reading thread instead,
/// inside its `read` calls.
///
/// See [`GzipProgress`] for how the returned telemetry becomes a progress
/// estimate. The decoder streams with backpressure, so peak memory is bounded
/// by the in-flight-chunk budget regardless of input size.
fn open_rapidgzip(
    file: File,
    decompress_threads: usize,
    trailer_total: Option<u64>,
) -> io::Result<OpenedGzip> {
    let compressed = Arc::new(AtomicU64::new(0));
    let source = CountingSource {
        file,
        counter: Arc::clone(&compressed),
    };
    let decoder = rapidgzip_core::Decoder::builder()
        .decoder_threads(decompress_threads.max(1))
        .build()?;
    // TODO: support Random Access with index
    let reader = if decompress_threads > 0 {
        decoder.reader(source)
    } else {
        decoder.stream_reader(source)
    }
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let progress = GzipProgress {
        compressed,
        handle: reader.handle(),
        estimate: std::sync::Mutex::new(GzipEstimate {
            ratio_total: 0,
            trailer: trailer_total,
        }),
    };
    Ok(OpenedGzip {
        reader,
        progress,
        background_threads: decompress_threads,
    })
}

// ---------------------------------------------------------------------------
// FastQFile
// ---------------------------------------------------------------------------

/// FASTQ file reader that supports plain text, gzip, and bzip2 compressed files.
///
/// Mirrors `Sequence.FastQFile` in Java. Uses a look-ahead design
/// where `readNext()` is called at construction time and after each `next()` call,
/// so `hasNext()` can report whether more sequences are available. In Rust we use
/// `Option<Sequence>` stored in `next_sequence` for the same pattern.
pub struct FastqFile {
    reader: FastqReader,
    name: String,
    file_size: u64,
    /// Decoder threads working in the background for this file.
    background_threads: usize,

    /// The next sequence ready to be returned (look-ahead buffer).
    next_sequence: Option<Sequence>,

    /// Current line number for error messages, incremented on every
    /// `readLine()` call exactly as in Java.
    line_number: u64,

    /// The lowest raw quality character seen so far (for Phred encoding detection).
    pub lowest_char: u8,
}

impl FastqFile {
    /// Open a FASTQ file for reading.
    ///
    /// The Java constructor opens the file, wraps it in the
    /// appropriate decompression stream, and immediately calls `readNext()` to
    /// prime the look-ahead buffer.
    pub fn open<P: AsRef<Path>>(decompress_threads: usize, path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        // For stdin, Java sets fileSize to Long.MAX_VALUE.
        let file_size = std::fs::metadata(path)?.len();

        // Java keeps the raw FileInputStream (fis) and queries
        // fis.getChannel().position() for progress tracking. We clone the File
        // handle before wrapping it in decompression so we can seek on the clone
        // to get the compressed byte position.
        let (reader, background_threads) = {
            let lower_name = name.to_lowercase();
            let compression = if lower_name.ends_with(".gz") {
                "gz"
            } else if lower_name.ends_with(".bz2") {
                "bz2"
            } else {
                detect_compression_from_magic(path)?
            };

            let file = File::open(path)?;

            match compression {
                "gz" => {
                    let trailer = gzip_trailer_size(path, file_size);
                    let gz = open_rapidgzip(file, decompress_threads, trailer)?;
                    (
                        FastqReader::Gzip(
                            Box::new(BufReader::with_capacity(READ_BUFFER, gz.reader)),
                            gz.progress,
                        ),
                        gz.background_threads,
                    )
                }
                "bz2" => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "not supported yet",
                    ));
                }
                _ => {
                    let pos_handle = file.try_clone()?;
                    (
                        FastqReader::Plain(BufReader::with_capacity(READ_BUFFER, file), pos_handle),
                        0,
                    )
                }
            }
        };

        let mut fq = Self {
            reader,
            name,
            file_size,
            background_threads,
            next_sequence: None,
            line_number: 0,
            lowest_char: 255,
        };

        // Prime the look-ahead buffer by reading the first record.
        fq.read_next()?;

        Ok(fq)
    }

    /// Read the next line, without its line ending, into `out` (cleared first),
    /// incrementing `line_number`. Returns `false` at EOF. The line is not
    /// UTF-8 validated; callers do that once, where the bytes are used.
    ///
    /// Not `read_until`: its newline search is measurably slower than the
    /// `memchr` crate's SIMD search on this, the parser's hottest loop.
    fn read_line(&mut self, out: &mut Vec<u8>) -> io::Result<bool> {
        out.clear();
        let mut read_any = false;
        loop {
            let available = match self.reader.fill_buf() {
                Ok(available) => available,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            if available.is_empty() {
                break;
            }
            read_any = true;
            if let Some(newline) = memchr::memchr(b'\n', available) {
                out.extend_from_slice(&available[..newline]);
                self.reader.consume(newline + 1);
                break;
            }
            let len = available.len();
            out.extend_from_slice(available);
            self.reader.consume(len);
        }
        self.line_number += 1;
        if !read_any {
            return Ok(false);
        }
        while out.last() == Some(&b'\r') {
            out.pop();
        }
        Ok(true)
    }

    /// Read the next FASTQ record into `self.next_sequence`.
    ///
    /// This mirrors `readNext()` in the Java code, including:
    /// - Skipping blank lines between records
    /// - Validating the '@' prefix on the ID line
    /// - Validating the '+' prefix on the mid-line
    /// - Colorspace detection on the first record only
    /// - CASAVA filter detection via `:Y:` in the read ID
    fn read_next(&mut self) -> io::Result<()> {
        // -- ID line (skip blank lines) --
        // The Java code loops reading lines until it finds a non-empty
        // one or hits EOF. Blank lines between records are silently skipped.
        let mut id_bytes = Vec::new();
        loop {
            if !self.read_line(&mut id_bytes)? {
                // EOF
                self.next_sequence = None;
                return Ok(());
            }
            if !id_bytes.is_empty() {
                break;
            }
        }

        if !id_bytes.starts_with(b"@") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("ID line didn't start with '@' at line {}", self.line_number),
            ));
        }
        let id = String::from_utf8(id_bytes).map_err(|_| invalid_utf8())?;

        // -- Sequence line --
        let mut seq_bytes = Vec::new();
        if !self.read_line(&mut seq_bytes)? {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Ran out of data in the middle of a fastq entry. Your file is probably truncated",
            ));
        }

        // -- Mid-line ('+' line), read into what will hold the qualities --
        let mut quality_bytes = Vec::with_capacity(seq_bytes.len());
        if !self.read_line(&mut quality_bytes)? {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Ran out of data in the middle of a fastq entry. Your file is probably truncated",
            ));
        }
        check_utf8(&quality_bytes)?;
        if !quality_bytes.starts_with(b"+") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Midline '{}' didn't start with '+' at {}",
                    String::from_utf8_lossy(&quality_bytes),
                    self.line_number
                ),
            ));
        }

        // -- Quality line --
        if !self.read_line(&mut quality_bytes)? {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Ran out of data in the middle of a fastq entry. Your file is probably truncated",
            ));
        }
        check_utf8(&quality_bytes)?;

        // Track lowest quality character for Phred encoding detection.
        if let Some(lowest) = quality_bytes.iter().copied().min() {
            self.lowest_char = self.lowest_char.min(lowest);
        }

        // Build the Sequence
        let sequence = Sequence::new(id, seq_bytes, quality_bytes);

        self.next_sequence = Some(sequence);

        Ok(())
    }
}

impl SequenceFile for FastqFile {
    fn next(&mut self) -> Option<io::Result<Sequence>> {
        // Java's `next()` returns the current `nextSequence` then calls
        // `readNext()` to prime the next one. We do the same.
        let current = self.next_sequence.take()?;
        if let Err(e) = self.read_next() {
            // Store nothing for next time; the error is returned on the *next* call
            // would be confusing. Instead, return the error now and let the current
            // sequence be lost (matching Java which throws from next()).
            // Actually, Java's next() calls readNext() but returns the previous value.
            // If readNext() throws, the exception propagates out of next().
            // We replicate: return the error, dropping `current`.
            return Some(Err(e));
        }
        Some(Ok(current))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn background_threads(&self) -> usize {
        self.background_threads
    }

    /// Java reads `fis.getChannel().position()` which gives the
    /// *compressed* byte position, then divides by file size (also compressed).
    /// For plain files this is exact; for compressed files it gives a rough
    /// estimate based on compressed bytes consumed.
    fn percent_complete(&self) -> f64 {
        if self.next_sequence.is_none() {
            return 100.0;
        }
        self.reader.progress(self.file_size)
    }
}

/// The error `BufRead::read_line` gives for a line that is not UTF-8.
fn invalid_utf8() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
    )
}

fn check_utf8(bytes: &[u8]) -> io::Result<()> {
    std::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|_| invalid_utf8())
}
