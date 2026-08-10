//! UTC ISO-8601 timestamps (`YYYY-MM-DDTHH:MM:SSZ`) without a time crate.
//!
//! No fractional seconds, `Z` suffix — lexicographically sortable for `ORDER BY`.

use std::time::{SystemTime, UNIX_EPOCH};

/// The current time as an ISO-8601 UTC string.
pub fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    iso_from_unix(secs)
}

/// Format a unix timestamp (seconds) as ISO-8601 UTC.
pub fn iso_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);
    let hour = sod / 3600;
    let min = (sod % 3600) / 60;
    let sec = sod % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Howard Hinnant's civil-from-days algorithm (proleptic Gregorian, days since 1970-01-01).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_formats_correctly() {
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn a_known_timestamp() {
        // 1700000000 = 2023-11-14T22:13:20Z (a well-known round-ish unix time).
        assert_eq!(iso_from_unix(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn now_iso_is_well_formed() {
        let s = now_iso();
        assert!(s.ends_with('Z'), "{s}");
        assert_eq!(s.len(), 20, "{s}"); // YYYY-MM-DDTHH:MM:SSZ
    }
}
