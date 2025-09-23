use alloc::string::String;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};

use super::error::IdEventError;

// Wrapper for seconds-since-epoch timestamps
pub(crate) struct Timestamp(pub i64);

impl TryFrom<Timestamp> for String {
    type Error = IdEventError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        Ok(Utc
            .timestamp_opt(value.0, 0)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)?
            .to_rfc3339_opts(SecondsFormat::Secs, true))
    }
}

impl TryFrom<Timestamp> for DateTime<Utc> {
    type Error = IdEventError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        Utc.timestamp_opt(value.0, 0)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)
    }
}
