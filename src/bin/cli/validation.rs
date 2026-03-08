use rust_decimal::Decimal;
use std::str::FromStr;

pub fn validate_year(year_str: &str) -> Result<u32, String> {
    year_str
        .trim()
        .parse::<u32>()
        .map_err(|_| "Ano deve ser um número válido".to_string())
}

pub fn validate_rate(rate_str: &str) -> Result<Decimal, String> {
    let rate_str = rate_str.trim();

    let decimal = Decimal::from_str(rate_str)
        .map_err(|_| "Taxa deve ser um número decimal válido".to_string())?;

    if decimal.is_sign_negative() {
        return Err("Taxa não pode ser negativa".to_string());
    }

    // Check if it has more than 2 decimal places
    let decimal_str = decimal.to_string();
    if let Some(dot_pos) = decimal_str.find('.') {
        let decimal_places = decimal_str.len() - dot_pos - 1;
        if decimal_places > 2 {
            return Err("Taxa pode ter no máximo 2 casas decimais".to_string());
        }
    }

    Ok(decimal)
}

pub fn validate_year_is_unique(year: u32, existing_years: &[u32]) -> Result<(), String> {
    if existing_years.contains(&year) {
        return Err(format!("Ano {} já existe nos dados", year));
    }
    Ok(())
}

pub fn validate_years_ascending(years: &[u32]) -> Result<(), String> {
    let mut prev = 0u32;

    for &year in years {
        if year <= prev {
            return Err(format!(
                "Anos devem estar em ordem ascendente. {} não é maior que {}",
                year, prev
            ));
        }
        prev = year;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_rate_success() {
        assert!(validate_rate("1.22").is_ok());
        assert!(validate_rate("3.23").is_ok());
        assert!(validate_rate("0").is_ok());
        assert!(validate_rate("0.5").is_ok());
    }

    #[test]
    fn test_validate_rate_too_many_decimals() {
        assert!(validate_rate("1.234").is_err());
        assert!(validate_rate("3.2345").is_err());
    }

    #[test]
    fn test_validate_rate_negative() {
        assert!(validate_rate("-1.22").is_err());
    }

    #[test]
    fn test_validate_years_ascending() {
        assert!(validate_years_ascending(&[2023, 2024, 2025]).is_ok());
        assert!(validate_years_ascending(&[2023, 2023, 2024]).is_err());
        assert!(validate_years_ascending(&[2025, 2024, 2023]).is_err());
    }
}
