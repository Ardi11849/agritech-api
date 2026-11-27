use chrono::{Duration, NaiveDate, Utc};

/// Mengembalikan tanggal hari ini dalam zona waktu Indonesia Barat (WIB / UTC+7).
pub fn today_wib() -> NaiveDate {
    (Utc::now() + Duration::hours(7)).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_today_wib_not_empty() {
        let date = today_wib();
        assert!(date.year() >= 2026);
    }
}
