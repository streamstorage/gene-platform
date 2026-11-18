/// Phred quality encoding detection.
///
/// Replicates the logic from `Sequence/QualityEncoding/PhredEncoding.java`.

#[derive(Debug, Clone, Copy)]
pub struct PhredEncoding {
    pub name: &'static str,
    pub offset: u8,
}

// These constants match the Java SANGER_ENCODING_OFFSET and
// ILLUMINA_1_3_ENCODING_OFFSET fields exactly.
const SANGER_ENCODING_OFFSET: u8 = 33;
const ILLUMINA_1_3_ENCODING_OFFSET: u8 = 64;

impl PhredEncoding {
    /// The Sanger / Illumina 1.9 (Phred+33) encoding.
    ///
    /// This is the encoding produced by construction for BAM/SAM input:
    /// BAM stores raw Phred values which the reader converts to ASCII by
    /// adding 33, and SAM quality strings are Phred+33 by specification.
    pub const SANGER: PhredEncoding = PhredEncoding {
        name: "Sanger / Illumina 1.9",
        offset: SANGER_ENCODING_OFFSET,
    };
}

impl PhredEncoding {
    /// Marker for `--phred64`: tells [`resolve`] to read the quality as
    /// Phred+64, naming Illumina 1.3 vs 1.5 from the lowest character.
    pub const PHRED64: PhredEncoding = PhredEncoding {
        name: "Illumina 1.5",
        offset: ILLUMINA_1_3_ENCODING_OFFSET,
    };
}

/// Lowest-quality-character starting value for modules that track it. Java
/// uses 1000, an impossible char, so "no data seen" can be told apart.
pub const NO_QUALITY_SEEN: u16 = 1000;

/// Resolve the quality encoding for a data source.
///
/// `hint` comes from [`crate::modules::QCModule::set_phred_encoding`]: the
/// encoding fixed by the file format (BAM/SAM), [`PhredEncoding::PHRED64`]
/// for `--phred64`, or `None` for the Phred+33 default.
pub fn resolve(hint: Option<PhredEncoding>, lowest_char: u16) -> Result<PhredEncoding, String> {
    match hint {
        Some(e) if e.offset == ILLUMINA_1_3_ENCODING_OFFSET => detect(lowest_char, true),
        Some(e) => Ok(e),
        None => detect(lowest_char, false),
    }
}

/// Replicates `PhredEncoding.getFastQEncodingOffset(char)` from FastQC 0.13:
/// Phred+33 unless `phred64` is set, with a feasibility check either way.
pub fn detect(lowest_char: u16, phred64: bool) -> Result<PhredEncoding, String> {
    let c = char::from_u32(lowest_char as u32).unwrap_or('?');
    if lowest_char < 33 {
        return Err(format!(
            "No known encodings with chars < 33 (Yours was '{}' with value {})",
            c, lowest_char
        ));
    }
    if !phred64 {
        return Ok(PhredEncoding::SANGER);
    }
    if lowest_char < ILLUMINA_1_3_ENCODING_OFFSET as u16 {
        return Err(format!(
            "Phred64 encoding is incompatible with having ASCII char '{}' with value {}) in the file",
            c, lowest_char
        ));
    }
    // Illumina 1.3 allowed quality 1 (ASCII 65); from 1.5 the minimum was 2.
    if lowest_char == ILLUMINA_1_3_ENCODING_OFFSET as u16 + 1 {
        Ok(PhredEncoding {
            name: "Illumina 1.3",
            offset: ILLUMINA_1_3_ENCODING_OFFSET,
        })
    } else {
        Ok(PhredEncoding::PHRED64)
    }
}

/// Java warns when Phred+33 data has nothing below Q31, which suggests the
/// file may really be Phred+64.
pub fn phred64_suspicion(lowest_char: u16) -> Option<String> {
    (lowest_char >= ILLUMINA_1_3_ENCODING_OFFSET as u16 && lowest_char != NO_QUALITY_SEEN).then(
        || {
            format!(
            "Using Phred33 encoding your lowest quality is {} could this file be Phred64 encoded?",
            lowest_char - SANGER_ENCODING_OFFSET as u16
        )
        },
    )
}
