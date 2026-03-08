use super::*;
use rust_decimal::Decimal;

#[test]
fn test_annual_inflation_entry_creation_with_builder() {
    let entry = AnnualInflationEntry::builder()
        .with_year(2023)
        .with_inflation("1.22")
        .build()
        .unwrap();

    assert_eq!(entry.get_year(), 2023);
    assert_eq!(entry.get_inflation(), "1.22");
}

#[test]
fn test_annual_inflation_creation_valid() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let result = AnnualInflation::new(entries);
    assert!(result.is_ok());

    let inflation = result.unwrap();
    assert_eq!(inflation.entries().len(), 3);
}

#[test]
fn test_calculate_multiplier_basic() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let inflation = AnnualInflation::new(entries).unwrap();
    let multiplier = inflation.calculate_multiplier(2023, 2025).unwrap();

    // Expected: 1.0122 * 1.0323 * 1.0432 ≈ 1.0900
    // (1 + 1.22/100) * (1 + 3.23/100) * (1 + 4.32/100)
    let expected = Decimal::from_str_exact("1.0900").unwrap();
    assert_eq!(multiplier, expected);
}

#[test]
fn test_calculate_multiplier_single_year() {
    let entries = vec![(2023, "1.22".to_string()), (2024, "3.23".to_string())];

    let inflation = AnnualInflation::new(entries).unwrap();
    let multiplier = inflation.calculate_multiplier(2023, 2023).unwrap();

    // Expected: 1.0122 (with rounding up)
    let expected = Decimal::from_str_exact("1.0122").unwrap();
    assert_eq!(multiplier, expected);
}

#[test]
fn test_calculate_multiplier_year_not_found() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let inflation = AnnualInflation::new(entries).unwrap();
    let result = inflation.calculate_multiplier(2020, 2025);

    assert!(result.is_err());
    match result {
        Err(InflationError::YearNotFound(year)) => {
            assert_eq!(year, 2020);
        }
        _ => panic!("Expected InflationError::YearNotFound(2020)"),
    }
}

#[test]
fn test_calculate_multiplier_end_year_not_found() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let inflation = AnnualInflation::new(entries).unwrap();
    let result = inflation.calculate_multiplier(2023, 2026);

    assert!(result.is_err());
    match result {
        Err(InflationError::YearNotFound(year)) => {
            assert_eq!(year, 2026);
        }
        _ => panic!("Expected InflationError::YearNotFound(2026)"),
    }
}

#[test]
fn test_years_not_ordered() {
    let entries = vec![
        (2025, "4.32".to_string()),
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
    ];

    let result = AnnualInflation::new(entries);
    assert!(result.is_err());
    match result {
        Err(InflationError::YearsNotOrdered) => {
            // OK
        }
        _ => panic!("Expected InflationError::YearsNotOrdered"),
    }
}

#[test]
fn test_invalid_inflation_value_negative() {
    let entries = vec![(2023, "1.22".to_string()), (2024, "-3.23".to_string())];

    let result = AnnualInflation::new(entries);
    assert!(result.is_err());
    match result {
        Err(InflationError::InvalidInflationValue { year, .. }) => {
            assert_eq!(year, 2024);
        }
        _ => panic!("Expected InflationError::InvalidInflationValue"),
    }
}

#[test]
fn test_invalid_inflation_value_parse_error() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "not_a_number".to_string()),
    ];

    let result = AnnualInflation::new(entries);
    assert!(result.is_err());
    match result {
        Err(InflationError::InvalidInflationValue { year, .. }) => {
            assert_eq!(year, 2024);
        }
        _ => panic!("Expected InflationError::InvalidInflationValue"),
    }
}

#[test]
fn test_serialization() {
    let entries = vec![
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let inflation = AnnualInflation::new(entries).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&inflation).unwrap();

    // Deserialize back
    let deserialized: AnnualInflation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.entries().len(), inflation.entries().len());
    assert_eq!(
        deserialized.entries()[0].get_year(),
        inflation.entries()[0].get_year()
    );
    assert_eq!(
        deserialized.entries()[0].get_inflation(),
        inflation.entries()[0].get_inflation()
    );
}

#[test]
fn test_multiple_year_ranges() {
    let entries = vec![
        (2020, "2.10".to_string()),
        (2021, "4.70".to_string()),
        (2022, "8.40".to_string()),
        (2023, "1.22".to_string()),
        (2024, "3.23".to_string()),
        (2025, "4.32".to_string()),
    ];

    let inflation = AnnualInflation::new(entries).unwrap();

    // Test 1: 2020-2022
    let mult_2020_2022 = inflation.calculate_multiplier(2020, 2022).unwrap();
    // (1 + 2.10/100) * (1 + 4.70/100) * (1 + 8.40/100)
    assert!(mult_2020_2022 > Decimal::ONE);

    // Test 2: 2023-2025 (the requirement case)
    let mult_2023_2025 = inflation.calculate_multiplier(2023, 2025).unwrap();
    assert_eq!(mult_2023_2025, Decimal::from_str_exact("1.0900").unwrap());

    // Test 3: 2020-2025
    let mult_2020_2025 = inflation.calculate_multiplier(2020, 2025).unwrap();
    assert!(mult_2020_2025 > mult_2023_2025);
}

#[test]
fn test_entries_getter() {
    let entries = vec![(2023, "1.22".to_string()), (2024, "3.23".to_string())];

    let inflation = AnnualInflation::new(entries).unwrap();
    let retrieved = inflation.entries();

    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0].get_year(), 2023);
    assert_eq!(retrieved[1].get_year(), 2024);
}

#[test]
fn test_ceiling_rounding() {
    let entries = vec![(2023, "1.111".to_string()), (2024, "2.222".to_string())];

    let inflation = AnnualInflation::new(entries).unwrap();
    let multiplier = inflation.calculate_multiplier(2023, 2024).unwrap();

    // (1 + 1.111/100) * (1 + 2.222/100) = 1.01111 * 1.02222 ≈ 1.033577...
    // With arithmetic rounding to 4 decimal places: 1.0336
    assert_eq!(multiplier, Decimal::from_str_exact("1.0336").unwrap());
}

#[test]
fn test_zero_inflation() {
    let entries = vec![(2023, "0".to_string()), (2024, "0".to_string())];

    let inflation = AnnualInflation::new(entries).unwrap();
    let multiplier = inflation.calculate_multiplier(2023, 2024).unwrap();

    // (1 + 0/100) * (1 + 0/100) = 1.0000
    assert_eq!(multiplier, Decimal::from_str_exact("1.0000").unwrap());
}
