//! RSS 2.0 の `pubDate` / `lastBuildDate` が要求する RFC 822 形式の日時文字列への変換。
//!
//! DB から取得する日時は RFC3339 (UTC, 例: `2026-01-01T00:00:00Z`) 文字列を想定しており、
//! 日時計算用の外部 crate (chrono 等) を追加せず、標準ライブラリのみで変換する。
//! オフセット付き (`+09:00` 等) の入力はサポートしない (UTC 前提)。

const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
/// 平年における各月の最大日数 (`DAYS_IN_MONTH[month - 1]`)。2月はうるう年かどうかで別途判定する。
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// `year`/`month` (1-indexed) におけるその月の最大日数。`month` は `1..=12` を想定する。
fn max_day_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[(month - 1) as usize]
    }
}

/// `YYYY-MM-DDTHH:MM:SS[.fff]Z` 形式の RFC3339 (UTC) 文字列を
/// `Wed, 01 Jan 2026 00:00:00 GMT` 形式の RFC 822 文字列に変換する。
///
/// パースに失敗した場合 (フォーマット不一致・UTC以外のオフセット等) は `None` を返す。
pub fn rfc3339_to_rfc822(input: &str) -> Option<String> {
    if !input.ends_with('Z') || input.len() < 20 {
        return None;
    }

    let year: i32 = input.get(0..4)?.parse().ok()?;
    if input.as_bytes().get(4) != Some(&b'-') {
        return None;
    }
    let month: u32 = input.get(5..7)?.parse().ok()?;
    if input.as_bytes().get(7) != Some(&b'-') {
        return None;
    }
    let day: u32 = input.get(8..10)?.parse().ok()?;
    if input.as_bytes().get(10) != Some(&b'T') {
        return None;
    }
    let hour: u32 = input.get(11..13)?.parse().ok()?;
    if input.as_bytes().get(13) != Some(&b':') {
        return None;
    }
    let minute: u32 = input.get(14..16)?.parse().ok()?;
    if input.as_bytes().get(16) != Some(&b':') {
        return None;
    }
    let second: u32 = input.get(17..19)?.parse().ok()?;

    if !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > max_day_in_month(year, month) {
        return None;
    }

    let weekday = day_of_week(year, month, day);

    Some(format!(
        "{}, {day:02} {} {year} {hour:02}:{minute:02}:{second:02} GMT",
        WEEKDAY_NAMES[weekday as usize],
        MONTH_NAMES[(month - 1) as usize],
    ))
}

/// Sakamoto's algorithm によるグレゴリオ暦の曜日計算。0 = 日曜 〜 6 = 土曜。
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let idx = (month - 1) as usize;
    (((y + y / 4 - y / 100 + y / 400 + T[idx] + day as i32) % 7 + 7) % 7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_known_date() {
        // 2026-01-01 は木曜日
        assert_eq!(
            rfc3339_to_rfc822("2026-01-01T00:00:00Z"),
            Some("Thu, 01 Jan 2026 00:00:00 GMT".to_string())
        );
    }

    #[test]
    fn converts_date_with_fractional_seconds() {
        assert_eq!(
            rfc3339_to_rfc822("2026-02-15T12:30:45.123Z"),
            Some("Sun, 15 Feb 2026 12:30:45 GMT".to_string())
        );
    }

    #[test]
    fn handles_leap_day() {
        // 2024-02-29 は木曜日
        assert_eq!(
            rfc3339_to_rfc822("2024-02-29T09:00:00Z"),
            Some("Thu, 29 Feb 2024 09:00:00 GMT".to_string())
        );
    }

    #[test]
    fn rejects_non_utc_offset() {
        assert_eq!(rfc3339_to_rfc822("2026-01-01T00:00:00+09:00"), None);
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(rfc3339_to_rfc822("not-a-date"), None);
        assert_eq!(rfc3339_to_rfc822(""), None);
    }

    #[test]
    fn rejects_nonexistent_day_in_month() {
        // 2026年は平年なので 2/30, 2/29 は存在しない
        assert_eq!(rfc3339_to_rfc822("2026-02-30T00:00:00Z"), None);
        assert_eq!(rfc3339_to_rfc822("2026-02-29T00:00:00Z"), None);
        // 4月は30日までしかない
        assert_eq!(rfc3339_to_rfc822("2026-04-31T00:00:00Z"), None);
    }

    #[test]
    fn rejects_leap_day_in_non_leap_year() {
        // 2100年はうるう年ではない (400 で割り切れない世紀年)
        assert_eq!(rfc3339_to_rfc822("2100-02-29T00:00:00Z"), None);
    }

    #[test]
    fn accepts_leap_day_in_leap_year_divisible_by_400() {
        // 2000年は 400 で割り切れるためうるう年
        assert_eq!(
            rfc3339_to_rfc822("2000-02-29T00:00:00Z"),
            Some("Tue, 29 Feb 2000 00:00:00 GMT".to_string())
        );
    }
}
