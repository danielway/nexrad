use crate::result::{Error, Result};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Returns a typed reference to the next `T` in `input` and advances `input` past it.
pub(crate) fn take_ref<'a, T>(input: &mut &'a [u8]) -> Result<&'a T>
where
    T: zerocopy::FromBytes + zerocopy::KnownLayout + zerocopy::Immutable,
{
    let (v, rest) = T::ref_from_prefix(*input).map_err(|_e| Error::UnexpectedEof)?;
    *input = rest;
    Ok(v)
}

/// Given a "modified" Julian date (date count since 1/1/1970) and a count of milliseconds since
/// midnight on that date, return an appropriate DateTime.
pub(crate) fn get_datetime(
    modified_julian_date: u16,
    past_midnight: Duration,
) -> Option<DateTime<Utc>> {
    let count_start = NaiveDate::from_ymd_opt(1970, 1, 1)?;
    let date = count_start + Duration::days(modified_julian_date as i64 - 1);
    let time = NaiveTime::from_num_seconds_from_midnight_opt(0, 0)? + past_midnight;

    Some(DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(date, time),
        Utc,
    ))
}
