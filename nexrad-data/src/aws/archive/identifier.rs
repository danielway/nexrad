use crate::result::{AwsError, Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// The length of the fixed `SSSSYYYYMMDD_HHMMSS` prefix shared by every archive object name.
const PREFIX_LEN: usize = 19;

/// The name of an object in the NEXRAD archive bucket, validated on construction.
///
/// Archive object names follow the grammar
///
/// ```text
/// SSSS YYYYMMDD _ HHMMSS [ _Vnn ] [ _MDM ] [ .gz ]
/// ```
///
/// where `SSSS` is the four-character radar site identifier (ASCII uppercase letters and digits),
/// `YYYYMMDD_HHMMSS` is the volume's UTC start time, `_Vnn` is the Archive II version suffix
/// (absent on the earliest files), `_MDM` marks a metadata object rather than a volume file, and
/// `.gz` marks a gzip-compressed object. For example: `KTLX20230520_201643_V06`,
/// `KDMX20220305_232324_V06_MDM`, `KTLX20100501_000444_V03.gz` and `KTLX19910605_162126.gz`.
///
/// Equality, ordering and hashing use the name only. Within a site, name order is chronological.
#[derive(Clone)]
pub struct Identifier {
    name: String,
    date_time: DateTime<Utc>,
    versioned: bool,
    metadata: bool,
    gzip: bool,
}

impl Identifier {
    /// Parses and validates an archive object name such as `KTLX20230520_201643_V06`.
    ///
    /// # Errors
    ///
    /// Returns [`AwsError::InvalidIdentifier`] if the name does not follow the archive grammar
    /// described on [`Identifier`].
    pub fn parse(name: &str) -> Result<Self> {
        let invalid = |reason: &'static str| {
            Error::Aws(AwsError::InvalidIdentifier {
                name: name.to_string(),
                reason,
            })
        };

        if !name.is_ascii() {
            return Err(invalid("name is not ASCII"));
        }
        if name.len() < PREFIX_LEN {
            return Err(invalid("name is shorter than a site and date-time"));
        }

        let (prefix, mut rest) = name.split_at(PREFIX_LEN);
        let site = &prefix[..4];
        if !site
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            return Err(invalid("site is not four uppercase letters or digits"));
        }

        let date_time = &prefix[4..];
        if date_time.as_bytes()[8] != b'_'
            || !date_time
                .bytes()
                .enumerate()
                .all(|(index, byte)| index == 8 || byte.is_ascii_digit())
        {
            return Err(invalid("date-time is not YYYYMMDD_HHMMSS"));
        }
        let date_time = NaiveDateTime::parse_from_str(date_time, "%Y%m%d_%H%M%S")
            .map_err(|_| invalid("date-time is not a valid calendar time"))?
            .and_utc();

        let mut versioned = false;
        if let Some(suffix) = rest.strip_prefix("_V") {
            let digits = suffix.get(..2).unwrap_or(suffix);
            if digits.len() != 2 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid("version is not _V followed by two digits"));
            }
            versioned = true;
            rest = &suffix[2..];
        }

        let metadata = match rest.strip_prefix("_MDM") {
            Some(suffix) => {
                rest = suffix;
                true
            }
            None => false,
        };

        let gzip = match rest.strip_prefix(".gz") {
            Some(suffix) => {
                rest = suffix;
                true
            }
            None => false,
        };

        if !rest.is_empty() {
            return Err(invalid("unrecognized suffix"));
        }

        Ok(Self {
            name: name.to_string(),
            date_time,
            versioned,
            metadata,
            gzip,
        })
    }

    /// The full object name, e.g. `KTLX20230520_201643_V06`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The radar site this object was produced at, e.g. `KTLX`.
    pub fn site(&self) -> &str {
        &self.name[..4]
    }

    /// The volume's start time, in UTC.
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }

    /// The Archive II version suffix without its leading underscore, e.g. `V06`, or `None` for
    /// names that carry no version.
    pub fn version(&self) -> Option<&str> {
        self.versioned
            .then(|| &self.name[PREFIX_LEN + 1..PREFIX_LEN + 4])
    }

    /// Whether this names a metadata object (`_MDM` suffix) rather than a volume file.
    pub fn is_metadata(&self) -> bool {
        self.metadata
    }

    /// Whether the name carries a `.gz` suffix, marking a gzip-compressed object.
    pub fn is_gzip(&self) -> bool {
        self.gzip
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self> {
        Self::parse(name)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Identifier").field(&self.name).finish()
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Identifier {}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
