use super::*;
use crate::inflation::AnnualInflation;
use rust_decimal::Decimal;

#[test]
fn test_currency_code_and_symbol() {
    assert_eq!(Currency::EUR.code(), "EUR");
    assert_eq!(Currency::EUR.symbol(), "€");
    assert_eq!(Currency::USD.code(), "USD");
    assert_eq!(Currency::USD.symbol(), "$");
}

#[test]
fn test_allocation_creation_with_builder() {
    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("1500.50")
        .build()
        .unwrap();

    assert_eq!(allocation.get_month(), "2026-03-01");
    assert_eq!(allocation.get_amount(), "1500.50");
}

#[test]
fn test_allocation_invalid_date_format() {
    let result = Allocation::builder()
        .with_month("2026/03/01")
        .with_amount("1500.50")
        .build();

    assert!(result.is_err());
    match result {
        Err(ValidationError::InvalidDateFormat(_)) => (),
        _ => panic!("Expected InvalidDateFormat error"),
    }
}

#[test]
fn test_allocation_invalid_date_not_first_day() {
    let result = Allocation::builder()
        .with_month("2026-03-15")
        .with_amount("1500.50")
        .build();

    assert!(result.is_err());
    match result {
        Err(ValidationError::InvalidDateFormat(msg)) => {
            assert!(msg.contains("first day of the month"));
        }
        _ => panic!("Expected InvalidDateFormat error"),
    }
}

#[test]
fn test_allocation_invalid_amount_too_many_decimals() {
    let result = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("1500.505")
        .build();

    assert!(result.is_err());
    match result {
        Err(ValidationError::InvalidAmountFormat(msg)) => {
            assert!(msg.contains("maximum 2"));
        }
        _ => panic!("Expected InvalidAmountFormat error"),
    }
}

#[test]
fn test_budget_allocation_creation_with_builder() {
    let budget = BudgetAllocation::builder()
        .with_description("Marketing Budget Q1")
        .with_amount("10000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    assert_eq!(budget.get_description(), "Marketing Budget Q1");
    assert_eq!(budget.get_amount(), "10000.00");
    assert_eq!(budget.get_currency(), Currency::EUR);
    assert_eq!(budget.get_reference_month(), "2026-03-01");
    assert!(budget.get_reference_allocations().is_empty());
    assert!(budget.get_portfolio_allocations().is_empty());
}

#[test]
fn test_add_allocations() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Marketing Budget")
        .with_amount("10000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("10000.00")
        .build()
        .unwrap();

    budget.add_reference_allocation(allocation);

    assert_eq!(budget.get_reference_allocations().len(), 1);
    assert_eq!(budget.get_portfolio_allocations().len(), 0);
    assert_eq!(budget.total_reference_allocations(), "10000.00");
    assert!(budget.is_consistent());
}

#[test]
fn test_serialization() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Operational Budget")
        .with_amount("5000.75")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("2500.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&budget).unwrap();

    // Deserialize back
    let deserialized: BudgetAllocation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.get_description(), budget.get_description());
    assert_eq!(deserialized.get_amount(), budget.get_amount());
    assert_eq!(deserialized.get_currency(), budget.get_currency());
    assert_eq!(deserialized.get_reference_allocations().len(), 1);
}

#[test]
fn test_consistency_check_valid() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Consistent Budget")
        .with_amount("10000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("4000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("6000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    // Budget is consistent: 4000 + 6000 = 10000 (only reference_allocations matters)
    assert!(budget.is_consistent());
    assert!(budget.validate().is_ok());
}

#[test]
fn test_consistency_check_invalid() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Inconsistent Budget")
        .with_amount("10000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("3000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("5000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    // Budget is inconsistent: 3000 + 5000 = 8000 ≠ 10000
    assert!(!budget.is_consistent());
    assert!(budget.validate().is_err());

    // Check the error
    let error = budget.validate().unwrap_err();
    match error {
        ValidationError::InconsistentAllocations { expected, actual } => {
            assert_eq!(expected, Decimal::from_str_exact("10000.00").unwrap());
            assert_eq!(actual, Decimal::from_str_exact("8000.00").unwrap());
        }
        _ => panic!("Expected InconsistentAllocations error"),
    }
}

#[test]
fn test_generate_portfolio_allocations_with_shift() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget with Shift")
        .with_amount("12000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("5000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("7000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    // Generate portfolio_allocations with a shift of 3 months, without exchange_rate
    budget
        .generate_portfolio_allocations(3, None, None)
        .unwrap();

    // Portfolio_allocations should have 2 allocations
    assert_eq!(budget.get_portfolio_allocations().len(), 2);
    // Same total as reference
    assert_eq!(budget.total_portfolio_allocations(), "12000.00");

    // Check shifted months (3 months ahead = June 2026)
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2026-06-01"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_month(),
        "2026-06-01"
    );

    // Check values
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "5000.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_amount(),
        "7000.00"
    );
}

#[test]
fn test_shift_negative() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Negative Shift")
        .with_amount("5000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("5000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    budget
        .generate_portfolio_allocations(-2, None, None)
        .unwrap();

    // Portfolio_allocation should be 2 months behind (January 2026)
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2026-01-01"
    );
}

#[test]
fn test_shift_positive_13_months_year_change() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Positive Shift 13 Months")
        .with_amount("8000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("8000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    budget
        .generate_portfolio_allocations(13, None, None)
        .unwrap();

    // Portfolio_allocation should be in April 2027 (13 months ahead)
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2027-04-01"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "8000.00"
    );
}

#[test]
fn test_shift_negative_13_months_year_change() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Negative Shift 13 Months")
        .with_amount("7500.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("7500.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    let result = budget.generate_portfolio_allocations(-13, None, None);

    // Retroactive allocation should return error
    assert!(result.is_err());
    match result {
        Err(ValidationError::RetroactiveAllocation {
            allocation_year,
            reference_year,
        }) => {
            assert_eq!(allocation_year, 2025);
            assert_eq!(reference_year, 2026);
        }
        _ => panic!("Expected RetroactiveAllocation error"),
    }
}

#[test]
fn test_shift_positive_13_months_multiple_allocations() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Multi Allocation with Year Change")
        .with_amount("10000.00")
        .with_currency(Currency::GBP)
        .with_reference_month("2026-01-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-01-01")
        .with_amount("2000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-06-01")
        .with_amount("3000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    let allocation3 = Allocation::builder()
        .with_month("2026-12-01")
        .with_amount("5000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation3);

    budget
        .generate_portfolio_allocations(13, None, None)
        .unwrap();

    // Check that we have 3 portfolio_allocations
    assert_eq!(budget.get_portfolio_allocations().len(), 3);

    // Check the shifted dates
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2027-02-01"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_month(),
        "2027-07-01"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[2].get_month(),
        "2028-01-01"
    );

    // Check that the values remain
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "2000.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_amount(),
        "3000.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[2].get_amount(),
        "5000.00"
    );

    // Check the total
    assert_eq!(budget.total_portfolio_allocations(), "10000.00");
}

#[test]
fn test_shift_negative_13_months_multiple_allocations() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Multi Allocation Backward Year Change")
        .with_amount("10000.00")
        .with_currency(Currency::JPY)
        .with_reference_month("2026-02-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-02-01")
        .with_amount("3000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-06-01")
        .with_amount("2500.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    let allocation3 = Allocation::builder()
        .with_month("2026-11-01")
        .with_amount("4500.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation3);

    let result = budget.generate_portfolio_allocations(-13, None, None);

    // Retroactive allocation should return error
    assert!(result.is_err());
    match result {
        Err(ValidationError::RetroactiveAllocation {
            allocation_year,
            reference_year,
        }) => {
            assert_eq!(allocation_year, 2025);
            assert_eq!(reference_year, 2026);
        }
        _ => panic!("Expected RetroactiveAllocation error"),
    }
}

#[test]
fn test_exchange_rate_applied_in_generation() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget with FX")
        .with_amount("1000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Generate portfolio_allocations with exchange_rate 1.0950
    let fx_rate = Decimal::from_str_exact("1.0950").unwrap();
    budget
        .generate_portfolio_allocations(3, Some(fx_rate), None)
        .unwrap();

    // Portfolio_allocation should have value multiplied by the rate
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1095.00"
    );
}

#[test]
fn test_exchange_rate_invalid_scale_in_generation() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget with invalid FX")
        .with_amount("1000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Try to generate with exchange_rate with 5 decimal places
    let invalid_fx = Decimal::from_str_exact("1.09501").unwrap();
    let result = budget.generate_portfolio_allocations(3, Some(invalid_fx), None);

    assert!(result.is_err());
    match result {
        Err(ValidationError::ExchangeRateInvalidScale { scale }) => {
            assert_eq!(scale, 5);
        }
        _ => panic!("Expected ValidationError::ExchangeRateInvalidScale"),
    }

    // Portfolio_allocations should not have been changed
    assert!(budget.get_portfolio_allocations().is_empty());
}

#[test]
fn test_exchange_rate_multiple_allocations() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Multi with FX")
        .with_amount("3000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2026-01-01")
        .build()
        .unwrap();

    let allocation1 = Allocation::builder()
        .with_month("2026-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2026-06-01")
        .with_amount("2000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    // Generate with exchange_rate of 0.9200 (conversion from USD to EUR)
    let fx_rate = Decimal::from_str_exact("0.9200").unwrap();
    budget
        .generate_portfolio_allocations(6, Some(fx_rate), None)
        .unwrap();

    // Check converted values
    assert_eq!(budget.get_portfolio_allocations()[0].get_amount(), "920.00");
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_amount(),
        "1840.00"
    );

    // Total should be converted
    assert_eq!(budget.total_portfolio_allocations(), "2760.00");
}

#[test]
fn test_exchange_rate_valid_scales() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget FX Scales")
        .with_amount("1000.00")
        .with_currency(Currency::GBP)
        .with_reference_month("2026-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2026-03-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Test with 4 decimal places
    let fx_4dp = Decimal::from_str_exact("1.2750").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_4dp), None)
            .is_ok()
    );
    assert_eq!(budget.get_portfolio_allocations().len(), 1);

    // Test with 3 decimal places
    let fx_3dp = Decimal::from_str_exact("1.105").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_3dp), None)
            .is_ok()
    );
    assert_eq!(budget.get_portfolio_allocations().len(), 1);

    // Test with 1 decimal place
    let fx_1dp = Decimal::from_str_exact("1.5").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_1dp), None)
            .is_ok()
    );
    assert_eq!(budget.get_portfolio_allocations().len(), 1);

    // Test with 0 decimal places
    let fx_whole = Decimal::from_str_exact("2").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_whole), None)
            .is_ok()
    );
    assert_eq!(budget.get_portfolio_allocations().len(), 1);
}

#[test]
fn test_inflation_applied_single_year() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget with Inflation")
        .with_amount("1000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation with 10% inflation for 2025
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for next year (2026) with inflation
    // shift = 12 months -> 2026-01-01
    budget
        .generate_portfolio_allocations(12, None, Some(&annual_inflation))
        .unwrap();

    // Expected amount: 1000 * 1.10 = 1100
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1100.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2026-01-01"
    );
}

#[test]
fn test_inflation_applied_multiple_years() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Multi Year Inflation")
        .with_amount("1000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation with 10% for 2025 and 5% for 2026
    let inflation_data = vec![(2025, "10.00".to_string()), (2026, "5.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for 2027 (shift = 24 months)
    // This needs to apply inflation for years 2025 and 2026
    budget
        .generate_portfolio_allocations(24, None, Some(&annual_inflation))
        .unwrap();

    // Expected amount: 1000 * (1 + 0.10) * (1 + 0.05) = 1000 * 1.10 * 1.05 = 1155
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1155.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2027-01-01"
    );
}

#[test]
fn test_inflation_no_inflation_same_year() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Same Year")
        .with_amount("2000.00")
        .with_currency(Currency::GBP)
        .with_reference_month("2025-06-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-06-01")
        .with_amount("2000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation
    let inflation_data = vec![(2025, "8.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations same year (shift = 3 months -> 2025-09-01)
    budget
        .generate_portfolio_allocations(3, None, Some(&annual_inflation))
        .unwrap();

    // No inflation should be applied (same year)
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "2000.00"
    );
}

#[test]
fn test_inflation_with_exchange_rate() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget FX + Inflation")
        .with_amount("1000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation with 10% for 2025
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Exchange rate: 1.5
    let fx_rate = Decimal::from_str_exact("1.5").unwrap();

    // Generate portfolio allocations for 2026 with both FX and inflation
    budget
        .generate_portfolio_allocations(12, Some(fx_rate), Some(&annual_inflation))
        .unwrap();

    // Expected: (1000 * 1.5) * 1.10 = 1500 * 1.10 = 1650
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1650.00"
    );
}

#[test]
fn test_inflation_none_uses_no_inflation() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget No Inflation Param")
        .with_amount("1000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Generate portfolio allocations for 2026 without providing inflation
    // Even though year changed, no inflation is applied
    budget
        .generate_portfolio_allocations(12, None, None)
        .unwrap();

    // No inflation applied, amount remains the same
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1000.00"
    );
}

#[test]
fn test_inflation_multiple_allocations_different_shifts() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Multi Allocation Inflation")
        .with_amount("3000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    // Two allocations at different times in 2025
    let allocation1 = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation1);

    let allocation2 = Allocation::builder()
        .with_month("2025-06-01")
        .with_amount("2000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation2);

    // Create AnnualInflation with 10% for 2025
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for 2026 (shift = 12 months)
    budget
        .generate_portfolio_allocations(12, None, Some(&annual_inflation))
        .unwrap();

    // Both allocations should have inflation applied
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "1100.00"
    );
    assert_eq!(
        budget.get_portfolio_allocations()[1].get_amount(),
        "2200.00"
    );

    // Check that total inflation is applied
    assert_eq!(budget.total_portfolio_allocations(), "3300.00");
}

#[test]
fn test_inflation_missing_year_in_data() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Budget Missing Inflation Year")
        .with_amount("1000.00")
        .with_currency(Currency::EUR)
        .with_reference_month("2025-01-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-01-01")
        .with_amount("1000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation with only 2025, missing 2026
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Try to generate portfolio allocations for 2027 (needs 2025 and 2026)
    let result = budget.generate_portfolio_allocations(24, None, Some(&annual_inflation));

    // Should fail because 2026 is not in the inflation data
    assert!(result.is_err());
    match result {
        Err(ValidationError::InflationCalculationError(msg)) => {
            assert!(msg.contains("Year 2026 not found"));
        }
        _ => panic!("Expected InflationCalculationError"),
    }
}

#[test]
fn test_inflation_complex_scenario() {
    let mut budget = BudgetAllocation::builder()
        .with_description("Complex Inflation Scenario")
        .with_amount("5000.00")
        .with_currency(Currency::USD)
        .with_reference_month("2025-03-01")
        .build()
        .unwrap();

    let allocation = Allocation::builder()
        .with_month("2025-03-01")
        .with_amount("5000.00")
        .build()
        .unwrap();
    budget.add_reference_allocation(allocation);

    // Create AnnualInflation for multiple years
    let inflation_data = vec![
        (2025, "3.50".to_string()),
        (2026, "4.20".to_string()),
        (2027, "2.80".to_string()),
    ];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // FX rate
    let fx_rate = Decimal::from_str_exact("1.25").unwrap();

    // Generate portfolio allocations 24 months ahead (2027-03-01)
    // This should apply inflation for years 2025 and 2026
    budget
        .generate_portfolio_allocations(24, Some(fx_rate), Some(&annual_inflation))
        .unwrap();

    // Expected calculation:
    // 1. Apply FX: 5000 * 1.25 = 6250
    // 2. Apply inflation (2025 and 2026): 6250 * (1 + 3.50/100) * (1 + 4.20/100)
    //    = 6250 * 1.0350 = 6468.75; 6468.75 * 1.0420 = 6740.625
    // Rounded to 2 decimal places: 6740.63
    assert_eq!(
        budget.get_portfolio_allocations()[0].get_amount(),
        "6740.63"
    );

    assert_eq!(
        budget.get_portfolio_allocations()[0].get_month(),
        "2027-03-01"
    );
}

#[test]
fn test_brl_budget_with_inflation_2025() {
    // Budget
    let mut budget = BudgetAllocation::builder()
        .with_description("BRL Budget with 2025 inflation")
        .with_amount("10000.00")
        .with_currency(Currency::BRL)
        .with_reference_month("2025-10-01")
        .build()
        .unwrap();

    // Allocations
    let allocations = vec![
        ("2026-07-01", "3000.00"),
        ("2026-08-01", "4000.00"),
        ("2026-09-01", "2000.00"),
        ("2026-10-01", "1000.00"),
    ];

    for (month, amount) in &allocations {
        let allocation = Allocation::builder()
            .with_month(month)
            .with_amount(amount)
            .build()
            .unwrap();
        budget.add_reference_allocation(allocation);
    }

    // Inflation of 2025: 4.21%
    let inflation_data = vec![(2025, "4.21".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Portfolio Allocations (without FX, shift 0)
    budget
        .generate_portfolio_allocations(0, None, Some(&annual_inflation))
        .unwrap();

    // Expected multiplier: 1 + 4.21/100 = 1.0421
    let multiplier = Decimal::from_str_exact("1.0421").unwrap();

    // Check allocation
    for (i, (month, amount_str)) in allocations.iter().enumerate() {
        let value = Decimal::from_str_exact(amount_str).unwrap();
        let expected = crate::rounding::round_to_2_decimals(value * multiplier);
        let alloc = &budget.get_portfolio_allocations()[i];
        assert_eq!(alloc.get_month(), *month);
        assert_eq!(
            alloc.get_amount(),
            expected.to_string(),
            "Incorrect adjusted allocation for {}",
            month
        );
    }

    // Check total
    let expected_total: Decimal = allocations
        .iter()
        .map(|(_, amount_str)| {
            let value = Decimal::from_str_exact(amount_str).unwrap();
            crate::rounding::round_to_2_decimals(value * multiplier)
        })
        .sum();
    let actual_total = Decimal::from_str_exact(&budget.total_portfolio_allocations()).unwrap();
    assert_eq!(actual_total, expected_total, "Incorrect adjusted total sum");
}
