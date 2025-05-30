use chrono::{NaiveDate, Utc};
use regex::Regex;
use std::sync::OnceLock;

/// Validuje email adresu
pub fn validate_email(email: &str) -> bool {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });
    regex.is_match(email)
}

/// Validuje datum ve formátu YYYY-MM-DD
pub fn validate_date_format(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| format!("Neplatný formát data: '{}'. Očekávaný formát: YYYY-MM-DD", date_str))
}

/// Validuje, že datum není v budoucnosti
pub fn validate_date_not_future(date: NaiveDate) -> Result<(), String> {
    let today = Utc::now().date_naive();
    if date > today {
        Err(format!("Datum {} nemůže být v budoucnosti", date))
    } else {
        Ok(())
    }
}

/// Validuje rozsah hodin (musí být pozitivní a rozumný)
pub fn validate_hours(hours: f64) -> Result<(), String> {
    if hours <= 0.0 {
        Err("Počet hodin musí být větší než 0".to_string())
    } else if hours > 24.0 {
        Err("Počet hodin nemůže být větší než 24 za den".to_string())
    } else {
        Ok(())
    }
}

/// Validuje ID (musí být pozitivní)
pub fn validate_positive_id(id: i32, field_name: &str) -> Result<(), String> {
    if id <= 0 {
        Err(format!("{} musí být pozitivní číslo", field_name))
    } else {
        Ok(())
    }
}

/// Validuje délku textu
pub fn validate_text_length(text: &str, field_name: &str, min_len: usize, max_len: usize) -> Result<(), String> {
    let len = text.len();
    if len < min_len {
        Err(format!("{} musí mít alespoň {} znaků", field_name, min_len))
    } else if len > max_len {
        Err(format!("{} může mít maximálně {} znaků", field_name, max_len))
    } else {
        Ok(())
    }
}

/// Validuje projekt identifier (alfanumerické znaky, pomlčky, podtržítka)
pub fn validate_project_identifier(identifier: &str) -> Result<(), String> {
    static IDENTIFIER_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = IDENTIFIER_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()
    });
    
    if identifier.is_empty() {
        return Err("Identifikátor projektu nemůže být prázdný".to_string());
    }
    
    if identifier.len() > 100 {
        return Err("Identifikátor projektu může mít maximálně 100 znaků".to_string());
    }
    
    if !regex.is_match(identifier) {
        return Err("Identifikátor projektu může obsahovat pouze písmena, číslice, pomlčky a podtržítka".to_string());
    }
    
    Ok(())
}

/// Validuje URL
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Ok(()); // Prázdná URL je v pořádku
    }
    
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL musí začínat http:// nebo https://".to_string());
    }
    
    // Základní validace URL struktury
    if url.len() > 2000 {
        return Err("URL je příliš dlouhá (max 2000 znaků)".to_string());
    }
    
    Ok(())
}

/// Validuje procenta (0-100)
pub fn validate_percentage(value: i32, field_name: &str) -> Result<(), String> {
    if value < 0 || value > 100 {
        Err(format!("{} musí být mezi 0 a 100", field_name))
    } else {
        Ok(())
    }
}

/// Validuje prioritu (obvykle 1-5)
pub fn validate_priority(priority: i32) -> Result<(), String> {
    if priority < 1 || priority > 10 {
        Err("Priorita musí být mezi 1 a 10".to_string())
    } else {
        Ok(())
    }
}

/// Validuje rozsah dat (from <= to)
pub fn validate_date_range(from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) -> Result<(), String> {
    if let (Some(from), Some(to)) = (from_date, to_date) {
        if from > to {
            return Err("Datum 'od' nemůže být později než datum 'do'".to_string());
        }
    }
    Ok(())
}

/// Validuje limit pro stránkování
pub fn validate_pagination_limit(limit: i32) -> Result<(), String> {
    if limit < 1 {
        Err("Limit musí být alespoň 1".to_string())
    } else if limit > 100 {
        Err("Limit nemůže být větší než 100".to_string())
    } else {
        Ok(())
    }
}

/// Validuje offset pro stránkování
pub fn validate_pagination_offset(offset: i32) -> Result<(), String> {
    if offset < 0 {
        Err("Offset nemůže být záporný".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name+tag@domain.co.uk"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_validate_date_format() {
        assert!(validate_date_format("2024-01-15").is_ok());
        assert!(validate_date_format("2024-12-31").is_ok());
        assert!(validate_date_format("invalid-date").is_err());
        assert!(validate_date_format("2024-13-01").is_err());
    }

    #[test]
    fn test_validate_hours() {
        assert!(validate_hours(8.0).is_ok());
        assert!(validate_hours(0.5).is_ok());
        assert!(validate_hours(24.0).is_ok());
        assert!(validate_hours(0.0).is_err());
        assert!(validate_hours(-1.0).is_err());
        assert!(validate_hours(25.0).is_err());
    }

    #[test]
    fn test_validate_project_identifier() {
        assert!(validate_project_identifier("my-project").is_ok());
        assert!(validate_project_identifier("project_123").is_ok());
        assert!(validate_project_identifier("PROJECT-NAME").is_ok());
        assert!(validate_project_identifier("").is_err());
        assert!(validate_project_identifier("project with spaces").is_err());
        assert!(validate_project_identifier("project@domain").is_err());
    }

    #[test]
    fn test_validate_percentage() {
        assert!(validate_percentage(0, "test").is_ok());
        assert!(validate_percentage(50, "test").is_ok());
        assert!(validate_percentage(100, "test").is_ok());
        assert!(validate_percentage(-1, "test").is_err());
        assert!(validate_percentage(101, "test").is_err());
    }
} 