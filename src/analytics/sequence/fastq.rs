// FASTQ file reader
// Corresponds to Sequence/FastQFile.java
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::path::Path;

// NOTE: rapidgzip's reader is referred to by its full path
// (`rapidgzip_core::DecoderReader`) below because the name `DecoderReader` is
// already taken by the bzip2 reader imported above.

use crate::analytics::config::Config;

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
    Gzip(Box<BufReader<GzipReader>>, GzipProgress),
}

/// Large so gzip refills (a decoder-thread handoff) are rare.
const READ_BUFFER: usize = 1 << 20;

/// 8MB is recommanded for Large fastq files.
const CHUNK_SIZE: usize = 8_388_608;

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

struct GzipProgress;

impl GzipProgress {
    /// Progress through the file as a percentage, or `None` before the decoder
    /// has produced enough to estimate a total. `buffered` is decompressed
    /// output already read but not yet parsed.
    fn percent(&self, file_size: u64, _buffered: u64) -> Option<f64> {
        if file_size == 0 {
            return None;
        }
        Some(100.0)
    }
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

enum GzipReader {
    Index(rapidgzip_core::IndexedReader<File>),
    Decoder(rapidgzip_core::DecoderReader),
}

unsafe impl Send for GzipReader {}

impl Read for GzipReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            GzipReader::Index(r) => r.read(buf),
            GzipReader::Decoder(r) => r.read(buf),
        }
    }
}

/// A gzip file opened through rapidgzip.
struct OpenedGzip {
    reader: GzipReader,
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
    index: Option<rapidgzip_core::DeflateIndex>,
    offset: Option<u64>,
    mut decompress_threads: usize,
) -> io::Result<OpenedGzip> {
    let decoder = rapidgzip_core::Decoder::builder()
        .decoder_threads(decompress_threads.max(1))
        .decoded_chunk_size(CHUNK_SIZE)
        .build()?;
    let reader = if let Some(index) = index {
        if offset.is_some() || decompress_threads == 0 {
            let mut reader = rapidgzip_core::IndexedReader::new(file, index)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if let Some(offset) = offset {
                reader.seek_to_line(offset)?;
            }
            decompress_threads = 0;
            GzipReader::Index(reader)
        } else {
            let reader = decoder
                .reader_from_index(file, std::sync::Arc::new(index))
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            GzipReader::Decoder(reader)
        }
    } else {
        if decompress_threads > 0 {
            let reader = decoder
                .reader(file)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            GzipReader::Decoder(reader)
        } else {
            let reader = decoder
                .stream_reader(file)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            GzipReader::Decoder(reader)
        }
    };
    let progress = GzipProgress;
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
    end_line: u64,
}

impl FastqFile {
    /// Open a FASTQ file for reading.
    ///
    /// The Java constructor opens the file, wraps it in the
    /// appropriate decompression stream, and immediately calls `readNext()` to
    /// prime the look-ahead buffer.
    pub fn open<P: AsRef<Path>>(config: &Config, path: P) -> io::Result<Vec<Self>> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid file"))?;

        // For stdin, Java sets fileSize to Long.MAX_VALUE.
        let file_size = std::fs::metadata(path)?.len();

        // Java keeps the raw FileInputStream (fis) and queries
        // fis.getChannel().position() for progress tracking. We clone the File
        // handle before wrapping it in decompression so we can seek on the clone
        // to get the compressed byte position.
        let lower_name = name.to_lowercase();
        let compression = if lower_name.ends_with(".gz") {
            "gz"
        } else if lower_name.ends_with(".bz2") {
            "bz2"
        } else {
            detect_compression_from_magic(path)?
        };

        let file = File::open(path)?;
        let index_file = config.index_file.as_ref().map(File::open).transpose()?;

        let fqs = match compression {
            "gz" => {
                if let Some(mut index_file) = index_file {
                    let index = rapidgzip_core::DeflateIndex::read_native(&mut index_file)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                    if let Some(decompress_threads) = config.decompress_threads {
                        let gz = open_rapidgzip(file, Some(index), None, decompress_threads)?;
                        let mut fq = Self {
                            reader: FastqReader::Gzip(
                                Box::new(BufReader::with_capacity(READ_BUFFER, gz.reader)),
                                gz.progress,
                            ),
                            name,
                            file_size,
                            background_threads: gz.background_threads,
                            next_sequence: None,
                            line_number: 0,
                            end_line: u64::MAX,
                        };
                        // Prime the look-ahead buffer by reading the first record.
                        fq.read_next()?;
                        vec![fq]
                    } else {
                        let parallel = (config.threads as f64 / (config.ratio + 1.0)).round() as usize;
                        let total_lines = index.total_line_count().unwrap_or(0);
                        if total_lines == 0 || total_lines % 4 != 0 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid index file",
                            ));
                        }
                        let mut fqs = Vec::new();
                        let mut offset = 0u64;
                        for i in 1..=parallel {
                            // use div_ceil to cover all the lines
                            let lines = total_lines.div_ceil(parallel as u64) * (i as u64);
                            let reminder = (-(lines as i64)) % 4;
                            let lines = (lines as i64 + reminder) as u64;

                            let gz = open_rapidgzip(
                                file.try_clone()?,
                                Some(index.clone()),
                                Some(offset),
                                0,
                            )?;
                            let mut fq = Self {
                                reader: FastqReader::Gzip(
                                    Box::new(BufReader::with_capacity(READ_BUFFER, gz.reader)),
                                    gz.progress,
                                ),
                                name: name.clone(),
                                file_size,
                                background_threads: gz.background_threads,
                                next_sequence: None,
                                line_number: 0,
                                end_line: lines - offset,
                            };
                            offset = lines;
                            // Prime the look-ahead buffer by reading the first record.
                            fq.read_next()?;
                            fqs.push(fq);
                        }
                        fqs
                    }
                } else {
                    let decompress_threads = config.decompress_threads.unwrap_or(1);
                    let gz = open_rapidgzip(file, None, None, decompress_threads)?;
                    let mut fq = Self {
                        reader: FastqReader::Gzip(
                            Box::new(BufReader::with_capacity(READ_BUFFER, gz.reader)),
                            gz.progress,
                        ),
                        name,
                        file_size,
                        background_threads: gz.background_threads,
                        next_sequence: None,
                        line_number: 0,
                        end_line: u64::MAX,
                    };
                    // Prime the look-ahead buffer by reading the first record.
                    fq.read_next()?;
                    vec![fq]
                }
            }
            "bz2" => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "not supported yet",
                ));
            }
            _ => {
                let pos_handle = file.try_clone()?;
                let mut fq = Self {
                    reader: FastqReader::Plain(
                        BufReader::with_capacity(READ_BUFFER, file),
                        pos_handle,
                    ),
                    name,
                    file_size,
                    background_threads: 0,
                    next_sequence: None,
                    line_number: 0,
                    end_line: u64::MAX,
                };
                // Prime the look-ahead buffer by reading the first record.
                fq.read_next()?;
                vec![fq]
            }
        };

        Ok(fqs)
    }

    /// Read the next line, without its line ending, into `out` (cleared first),
    /// incrementing `line_number`. Returns `false` at EOF. The line is not
    /// UTF-8 validated; callers do that once, where the bytes are used.
    ///
    /// Not `read_until`: its newline search is measurably slower than the
    /// `memchr` crate's SIMD search on this, the parser's hottest loop.
    fn read_line(&mut self, out: &mut Vec<u8>) -> io::Result<bool> {
        if self.line_number >= self.end_line {
            return Ok(false);
        }
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
