use chrono::{DateTime, Utc, NaiveDate, Local, Datelike, Duration};

/// Získá aktuální datum v UTC
pub fn current_date_utc() -> NaiveDate {
    Utc::now().date_naive()
}

/// Získá aktuální datum v lokálním časovém pásmu
pub fn current_date_local() -> NaiveDate {
    Local::now().date_naive()
}

/// Získá aktuální DateTime v UTC
pub fn current_datetime_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Parsuje datum ze stringu ve formátu YYYY-MM-DD
pub fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| format!("Neplatný formát data: '{}'. Očekávaný formát: YYYY-MM-DD", date_str))
}

/// Parsuje datum ze stringu s více možnými formáty
pub fn parse_date_flexible(date_str: &str) -> Result<NaiveDate, String> {
    let formats = [
        "%Y-%m-%d",     // 2024-01-15
        "%d.%m.%Y",     // 15.01.2024
        "%d/%m/%Y",     // 15/01/2024
        "%Y/%m/%d",     // 2024/01/15
    ];
    
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }
    
    Err(format!(
        "Neplatný formát data: '{}'. Podporované formáty: YYYY-MM-DD, DD.MM.YYYY, DD/MM/YYYY, YYYY/MM/DD",
        date_str
    ))
}

/// Formátuje datum do ISO formátu (YYYY-MM-DD)
pub fn format_date_iso(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Formátuje datum do českého formátu (DD.MM.YYYY)
pub fn format_date_czech(date: &NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// Formátuje DateTime do ISO formátu s časovým pásmem
pub fn format_datetime_iso(datetime: &DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Formátuje DateTime do českého formátu
pub fn format_datetime_czech(datetime: &DateTime<Utc>) -> String {
    datetime.format("%d.%m.%Y %H:%M:%S").to_string()
}

/// Získá začátek týdne (pondělí) pro dané datum
pub fn start_of_week(date: NaiveDate) -> NaiveDate {
    let days_from_monday = date.weekday().num_days_from_monday();
    date - Duration::days(days_from_monday as i64)
}

/// Získá konec týdne (neděle) pro dané datum
pub fn end_of_week(date: NaiveDate) -> NaiveDate {
    let days_to_sunday = 6 - date.weekday().num_days_from_monday();
    date + Duration::days(days_to_sunday as i64)
}

/// Získá začátek měsíce pro dané datum
pub fn start_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
        .unwrap_or(date)
}

/// Získá konec měsíce pro dané datum
pub fn end_of_month(date: NaiveDate) -> NaiveDate {
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
    };
    
    next_month
        .map(|d| d - Duration::days(1))
        .unwrap_or(date)
}

/// Získá začátek roku pro dané datum
pub fn start_of_year(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), 1, 1)
        .unwrap_or(date)
}

/// Získá konec roku pro dané datum
pub fn end_of_year(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), 12, 31)
        .unwrap_or(date)
}

/// Vypočítá počet pracovních dnů mezi dvěma daty (pondělí-pátek)
pub fn business_days_between(start: NaiveDate, end: NaiveDate) -> i64 {
    if start > end {
        return 0;
    }
    
    let mut count = 0;
    let mut current = start;
    
    while current <= end {
        let weekday = current.weekday().num_days_from_monday();
        if weekday < 5 { // Pondělí (0) až Pátek (4)
            count += 1;
        }
        current = current + Duration::days(1);
    }
    
    count
}

/// Kontroluje, zda je datum pracovní den (pondělí-pátek)
pub fn is_business_day(date: NaiveDate) -> bool {
    let weekday = date.weekday().num_days_from_monday();
    weekday < 5
}

/// Kontroluje, zda je datum víkend
pub fn is_weekend(date: NaiveDate) -> bool {
    !is_business_day(date)
}

/// Získá následující pracovní den
pub fn next_business_day(date: NaiveDate) -> NaiveDate {
    let mut next = date + Duration::days(1);
    while !is_business_day(next) {
        next = next + Duration::days(1);
    }
    next
}

/// Získá předchozí pracovní den
pub fn previous_business_day(date: NaiveDate) -> NaiveDate {
    let mut prev = date - Duration::days(1);
    while !is_business_day(prev) {
        prev = prev - Duration::days(1);
    }
    prev
}

/// Vytvoří rozsah dat pro časové filtrování
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    /// Vytvoří nový rozsah dat
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, String> {
        if start > end {
            Err("Počáteční datum nemůže být později než koncové datum".to_string())
        } else {
            Ok(DateRange { start, end })
        }
    }
    
    /// Vytvoří rozsah pro aktuální týden
    pub fn current_week() -> Self {
        let today = current_date_utc();
        DateRange {
            start: start_of_week(today),
            end: end_of_week(today),
        }
    }
    
    /// Vytvoří rozsah pro aktuální měsíc
    pub fn current_month() -> Self {
        let today = current_date_utc();
        DateRange {
            start: start_of_month(today),
            end: end_of_month(today),
        }
    }
    
    /// Vytvoří rozsah pro aktuální rok
    pub fn current_year() -> Self {
        let today = current_date_utc();
        DateRange {
            start: start_of_year(today),
            end: end_of_year(today),
        }
    }
    
    /// Vytvoří rozsah pro posledních N dní
    pub fn last_days(days: i64) -> Self {
        let today = current_date_utc();
        DateRange {
            start: today - Duration::days(days - 1),
            end: today,
        }
    }
    
    /// Kontroluje, zda datum spadá do rozsahu
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }
    
    /// Získá počet dní v rozsahu
    pub fn days_count(&self) -> i64 {
        (self.end - self.start).num_days() + 1
    }
    
    /// Získá počet pracovních dní v rozsahu
    pub fn business_days_count(&self) -> i64 {
        business_days_between(self.start, self.end)
    }
}

/// Relativní časové období
pub enum RelativePeriod {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    ThisYear,
    LastYear,
}

impl RelativePeriod {
    /// Převede relativní období na rozsah dat
    pub fn to_date_range(&self) -> DateRange {
        let today = current_date_utc();
        
        match self {
            RelativePeriod::Today => DateRange {
                start: today,
                end: today,
            },
            RelativePeriod::Yesterday => {
                let yesterday = today - Duration::days(1);
                DateRange {
                    start: yesterday,
                    end: yesterday,
                }
            },
            RelativePeriod::ThisWeek => DateRange::current_week(),
            RelativePeriod::LastWeek => {
                let last_week_start = start_of_week(today) - Duration::days(7);
                DateRange {
                    start: last_week_start,
                    end: last_week_start + Duration::days(6),
                }
            },
            RelativePeriod::ThisMonth => DateRange::current_month(),
            RelativePeriod::LastMonth => {
                let last_month = if today.month() == 1 {
                    NaiveDate::from_ymd_opt(today.year() - 1, 12, today.day())
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year() - 1, 12, 31).unwrap())
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month() - 1, today.day())
                        .unwrap_or_else(|| end_of_month(NaiveDate::from_ymd_opt(today.year(), today.month() - 1, 1).unwrap()))
                };
                DateRange {
                    start: start_of_month(last_month),
                    end: end_of_month(last_month),
                }
            },
            RelativePeriod::ThisYear => DateRange::current_year(),
            RelativePeriod::LastYear => {
                let last_year = today.year() - 1;
                DateRange {
                    start: NaiveDate::from_ymd_opt(last_year, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(last_year, 12, 31).unwrap(),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2024-01-15").is_ok());
        assert!(parse_date("2024-12-31").is_ok());
        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_parse_date_flexible() {
        assert!(parse_date_flexible("2024-01-15").is_ok());
        assert!(parse_date_flexible("15.01.2024").is_ok());
        assert!(parse_date_flexible("15/01/2024").is_ok());
        assert!(parse_date_flexible("2024/01/15").is_ok());
        assert!(parse_date_flexible("invalid").is_err());
    }

    #[test]
    fn test_business_days() {
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Pondělí
        let friday = NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(); // Pátek
        let saturday = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(); // Sobota
        
        assert!(is_business_day(monday));
        assert!(is_business_day(friday));
        assert!(!is_business_day(saturday));
        assert!(is_weekend(saturday));
        
        assert_eq!(business_days_between(monday, friday), 5);
    }

    #[test]
    fn test_date_range() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let range = DateRange::new(start, end).unwrap();
        
        assert_eq!(range.days_count(), 31);
        assert!(range.contains(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()));
    }
} 