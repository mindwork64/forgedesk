use chrono::{SecondsFormat, Utc};

/// All persisted timestamps are UTC RFC 3339 with millisecond precision.
/// Display-time conversion into the system timezone is a UI concern.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Compact timestamp used inside generated backup identifiers.
pub fn timestamp_for_id() -> String {
    Utc::now().format("%Y%m%d-%H%M%S").to_string()
}
