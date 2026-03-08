use super::*;
use crate::inflation::AnnualInflation;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn test_currency_code_and_symbol() {
    assert_eq!(Currency::EUR.code(), "EUR");
    assert_eq!(Currency::EUR.symbol(), "€");
    assert_eq!(Currency::USD.code(), "USD");
    assert_eq!(Currency::USD.symbol(), "$");
}

#[test]
fn test_allocation_creation() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("1500.50").unwrap();
    let allocation = Allocation::new(month, amount);

    assert_eq!(allocation.month, month);
    assert_eq!(allocation.amount, amount);
}

#[test]
fn test_budget_allocation_creation() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("10000.00").unwrap();

    let budget = BudgetAllocation::new(
        "Marketing Budget Q1".to_string(),
        amount,
        Currency::EUR,
        month,
    );

    assert_eq!(budget.description, "Marketing Budget Q1");
    assert_eq!(budget.amount, amount);
    assert_eq!(budget.currency, Currency::EUR);
    assert_eq!(budget.reference_month, month);
    assert!(budget.reference_allocations.is_empty());
    assert!(budget.portfolio_allocations.is_empty());
}

#[test]
fn test_add_allocations() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let budget_amount = Decimal::from_str_exact("10000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Marketing Budget".to_string(),
        budget_amount,
        Currency::USD,
        month,
    );

    let ref_amount = Decimal::from_str_exact("10000.00").unwrap();
    let ref_allocation = Allocation::new(month, ref_amount);
    budget.add_reference_allocation(ref_allocation);

    assert_eq!(budget.reference_allocations.len(), 1);
    assert_eq!(budget.portfolio_allocations.len(), 0);
    assert_eq!(budget.total_reference_allocations(), ref_amount);
    assert!(budget.is_consistent());
}

#[test]
fn test_serialization() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("5000.75").unwrap();
    let mut budget = BudgetAllocation::new(
        "Operational Budget".to_string(),
        amount,
        Currency::EUR,
        month,
    );

    let allocation = Allocation::new(month, Decimal::from_str_exact("2500.00").unwrap());
    budget.add_reference_allocation(allocation);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&budget).unwrap();
    println!("Serialized Budget:\n{}", json);

    // Deserialize back
    let deserialized: BudgetAllocation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.description, budget.description);
    assert_eq!(deserialized.amount, budget.amount);
    assert_eq!(deserialized.currency, budget.currency);
    assert_eq!(deserialized.reference_allocations.len(), 1);
}

#[test]
fn test_consistency_check_valid() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("10000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Consistent Budget".to_string(),
        total_amount,
        Currency::EUR,
        month,
    );

    let ref_amount_1 = Decimal::from_str_exact("4000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount_1));

    let ref_amount_2 = Decimal::from_str_exact("6000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount_2));

    // Budget is consistent: 4000 + 6000 = 10000 (only reference_allocations matters)
    assert!(budget.is_consistent());
    assert!(budget.validate().is_ok());
}

#[test]
fn test_consistency_check_invalid() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("10000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Inconsistent Budget".to_string(),
        total_amount,
        Currency::EUR,
        month,
    );

    let ref_amount = Decimal::from_str_exact("3000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount));

    let ref_amount_2 = Decimal::from_str_exact("5000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount_2));

    // Budget is inconsistent: 3000 + 5000 = 8000 ≠ 10000
    assert!(!budget.is_consistent());
    assert!(budget.validate().is_err());

    // Check the error
    let error = budget.validate().unwrap_err();
    assert_eq!(
        error,
        ValidationError::InconsistentAllocations {
            expected: total_amount,
            actual: Decimal::from_str_exact("8000.00").unwrap(),
        }
    );
}

#[test]
fn test_generate_portfolio_allocations_with_shift() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("12000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget with Shift".to_string(),
        total_amount,
        Currency::USD,
        month,
    );

    let ref_amount_1 = Decimal::from_str_exact("5000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount_1));

    let ref_amount_2 = Decimal::from_str_exact("7000.00").unwrap();
    budget.add_reference_allocation(Allocation::new(month, ref_amount_2));

    // Generate portfolio_allocations with a shift of 3 months, without exchange_rate
    budget
        .generate_portfolio_allocations(3, None, None)
        .unwrap();

    // Portfolio_allocations should have 2 allocations
    assert_eq!(budget.portfolio_allocations.len(), 2);
    // Same total as reference
    assert_eq!(budget.total_portfolio_allocations(), total_amount);

    // Check shifted months (3 months ahead = June 2026)
    let expected_month = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    assert_eq!(budget.portfolio_allocations[0].month, expected_month);
    assert_eq!(budget.portfolio_allocations[1].month, expected_month);

    // Check values
    assert_eq!(budget.portfolio_allocations[0].amount, ref_amount_1);
    assert_eq!(budget.portfolio_allocations[1].amount, ref_amount_2);
}

#[test]
fn test_shift_negative() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("5000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Negative Shift".to_string(),
        total_amount,
        Currency::EUR,
        month,
    );

    budget.add_reference_allocation(Allocation::new(month, total_amount));
    budget
        .generate_portfolio_allocations(-2, None, None)
        .unwrap();

    // Portfolio_allocation should be 2 months behind (January 2026)
    let expected_month = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    assert_eq!(budget.portfolio_allocations[0].month, expected_month);
}

#[test]
fn test_shift_positive_13_months_year_change() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("8000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Positive Shift 13 Months".to_string(),
        total_amount,
        Currency::USD,
        month,
    );

    budget.add_reference_allocation(Allocation::new(month, total_amount));
    budget
        .generate_portfolio_allocations(13, None, None)
        .unwrap();

    // Portfolio_allocation should be in April 2027 (13 months ahead)
    let expected_month = NaiveDate::from_ymd_opt(2027, 4, 1).unwrap();
    assert_eq!(budget.portfolio_allocations[0].month, expected_month);
    assert_eq!(budget.portfolio_allocations[0].amount, total_amount);
}

#[test]
fn test_shift_negative_13_months_year_change() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let total_amount = Decimal::from_str_exact("7500.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Negative Shift 13 Months".to_string(),
        total_amount,
        Currency::EUR,
        month,
    );

    budget.add_reference_allocation(Allocation::new(month, total_amount));
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
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mid_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();

    let amount1 = Decimal::from_str_exact("2000.00").unwrap();
    let amount2 = Decimal::from_str_exact("3000.00").unwrap();
    let amount3 = Decimal::from_str_exact("5000.00").unwrap();
    let total = Decimal::from_str_exact("10000.00").unwrap();

    let mut budget = BudgetAllocation::new(
        "Multi Allocation with Year Change".to_string(),
        total,
        Currency::GBP,
        start_date,
    );

    budget.add_reference_allocation(Allocation::new(start_date, amount1));
    budget.add_reference_allocation(Allocation::new(mid_date, amount2));
    budget.add_reference_allocation(Allocation::new(end_date, amount3));

    budget
        .generate_portfolio_allocations(13, None, None)
        .unwrap();

    // Check that we have 3 portfolio_allocations
    assert_eq!(budget.portfolio_allocations.len(), 3);

    // Check the shifted dates
    assert_eq!(
        budget.portfolio_allocations[0].month,
        NaiveDate::from_ymd_opt(2027, 2, 1).unwrap()
    );
    assert_eq!(
        budget.portfolio_allocations[1].month,
        NaiveDate::from_ymd_opt(2027, 7, 1).unwrap()
    );
    assert_eq!(
        budget.portfolio_allocations[2].month,
        NaiveDate::from_ymd_opt(2028, 1, 1).unwrap()
    );

    // Check that the values remain
    assert_eq!(budget.portfolio_allocations[0].amount, amount1);
    assert_eq!(budget.portfolio_allocations[1].amount, amount2);
    assert_eq!(budget.portfolio_allocations[2].amount, amount3);

    // Check the total
    assert_eq!(budget.total_portfolio_allocations(), total);
}

#[test]
fn test_shift_negative_13_months_multiple_allocations() {
    let start_date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
    let mid_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();

    let amount1 = Decimal::from_str_exact("3000.00").unwrap();
    let amount2 = Decimal::from_str_exact("2500.00").unwrap();
    let amount3 = Decimal::from_str_exact("4500.00").unwrap();
    let total = Decimal::from_str_exact("10000.00").unwrap();

    let mut budget = BudgetAllocation::new(
        "Multi Allocation Backward Year Change".to_string(),
        total,
        Currency::JPY,
        start_date,
    );

    budget.add_reference_allocation(Allocation::new(start_date, amount1));
    budget.add_reference_allocation(Allocation::new(mid_date, amount2));
    budget.add_reference_allocation(Allocation::new(end_date, amount3));

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
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget =
        BudgetAllocation::new("Budget with FX".to_string(), amount, Currency::EUR, month);

    budget.add_reference_allocation(Allocation::new(month, amount));

    // Generate portfolio_allocations with exchange_rate 1.0950
    let fx_rate = Decimal::from_str_exact("1.0950").unwrap();
    budget
        .generate_portfolio_allocations(3, Some(fx_rate), None)
        .unwrap();

    // Portfolio_allocation should have value multiplied by the rate
    let expected_amount = Decimal::from_str_exact("1095.00").unwrap();
    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount);
}

#[test]
fn test_exchange_rate_invalid_scale_in_generation() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget with invalid FX".to_string(),
        amount,
        Currency::EUR,
        month,
    );

    budget.add_reference_allocation(Allocation::new(month, amount));

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
    assert!(budget.portfolio_allocations.is_empty());
}

#[test]
fn test_exchange_rate_multiple_allocations() {
    let start_month = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mid_month = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    let amount1 = Decimal::from_str_exact("1000.00").unwrap();
    let amount2 = Decimal::from_str_exact("2000.00").unwrap();
    let total = Decimal::from_str_exact("3000.00").unwrap();

    let mut budget = BudgetAllocation::new(
        "Multi with FX".to_string(),
        total,
        Currency::USD,
        start_month,
    );

    budget.add_reference_allocation(Allocation::new(start_month, amount1));
    budget.add_reference_allocation(Allocation::new(mid_month, amount2));

    // Generate with exchange_rate of 0.9200 (conversion from USD to EUR)
    let fx_rate = Decimal::from_str_exact("0.9200").unwrap();
    budget
        .generate_portfolio_allocations(6, Some(fx_rate), None)
        .unwrap();

    // Check converted values
    let expected_amount1 = Decimal::from_str_exact("920.00").unwrap();
    let expected_amount2 = Decimal::from_str_exact("1840.00").unwrap();

    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount1);
    assert_eq!(budget.portfolio_allocations[1].amount, expected_amount2);

    // Total should be converted
    let expected_total = Decimal::from_str_exact("2760.00").unwrap();
    assert_eq!(budget.total_portfolio_allocations(), expected_total);
}

#[test]
fn test_exchange_rate_valid_scales() {
    let month = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget =
        BudgetAllocation::new("Budget FX Scales".to_string(), amount, Currency::GBP, month);

    budget.add_reference_allocation(Allocation::new(month, amount));

    // Test with 4 decimal places
    let fx_4dp = Decimal::from_str_exact("1.2750").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_4dp), None)
            .is_ok()
    );
    assert_eq!(budget.portfolio_allocations.len(), 1);

    // Test with 3 decimal places
    let fx_3dp = Decimal::from_str_exact("1.105").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_3dp), None)
            .is_ok()
    );
    assert_eq!(budget.portfolio_allocations.len(), 1);

    // Test with 1 decimal place
    let fx_1dp = Decimal::from_str_exact("1.5").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_1dp), None)
            .is_ok()
    );
    assert_eq!(budget.portfolio_allocations.len(), 1);

    // Test with 0 decimal places
    let fx_whole = Decimal::from_str_exact("2").unwrap();
    assert!(
        budget
            .generate_portfolio_allocations(0, Some(fx_whole), None)
            .is_ok()
    );
    assert_eq!(budget.portfolio_allocations.len(), 1);
}

#[test]
fn test_inflation_applied_single_year() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget with Inflation".to_string(),
        amount,
        Currency::EUR,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

    // Create AnnualInflation with 10% inflation for 2025
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for next year (2026) with inflation
    // shift = 12 months -> 2026-01-01
    budget
        .generate_portfolio_allocations(12, None, Some(&annual_inflation))
        .unwrap();

    // Expected amount: 1000 * 1.10 = 1100
    let expected_amount = Decimal::from_str_exact("1100.00").unwrap();
    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount);
    assert_eq!(
        budget.portfolio_allocations[0].month,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
}

#[test]
fn test_inflation_applied_multiple_years() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Multi Year Inflation".to_string(),
        amount,
        Currency::USD,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

    // Create AnnualInflation with 10% for 2025 and 5% for 2026
    let inflation_data = vec![(2025, "10.00".to_string()), (2026, "5.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for 2027 (shift = 24 months)
    // This needs to apply inflation for years 2025 and 2026
    budget
        .generate_portfolio_allocations(24, None, Some(&annual_inflation))
        .unwrap();

    // Expected amount: 1000 * (1 + 0.10) * (1 + 0.05) = 1000 * 1.10 * 1.05 = 1155
    let expected_amount = Decimal::from_str_exact("1155.00").unwrap();
    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount);
    assert_eq!(
        budget.portfolio_allocations[0].month,
        NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
    );
}

#[test]
fn test_inflation_no_inflation_same_year() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
    let amount = Decimal::from_str_exact("2000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Same Year".to_string(),
        amount,
        Currency::GBP,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

    // Create AnnualInflation
    let inflation_data = vec![(2025, "8.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations same year (shift = 3 months -> 2025-09-01)
    budget
        .generate_portfolio_allocations(3, None, Some(&annual_inflation))
        .unwrap();

    // No inflation should be applied (same year)
    assert_eq!(budget.portfolio_allocations[0].amount, amount);
}

#[test]
fn test_inflation_with_exchange_rate() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget FX + Inflation".to_string(),
        amount,
        Currency::USD,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

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
    let expected_amount = Decimal::from_str_exact("1650.00").unwrap();
    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount);
}

#[test]
fn test_inflation_none_uses_no_inflation() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget No Inflation Param".to_string(),
        amount,
        Currency::EUR,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

    // Generate portfolio allocations for 2026 without providing inflation
    // Even though year changed, no inflation is applied
    budget
        .generate_portfolio_allocations(12, None, None)
        .unwrap();

    // No inflation applied, amount remains the same
    assert_eq!(budget.portfolio_allocations[0].amount, amount);
}

#[test]
fn test_inflation_multiple_allocations_different_shifts() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    let amount1 = Decimal::from_str_exact("1000.00").unwrap();
    let amount2 = Decimal::from_str_exact("2000.00").unwrap();
    let total = Decimal::from_str_exact("3000.00").unwrap();

    let mut budget = BudgetAllocation::new(
        "Multi Allocation Inflation".to_string(),
        total,
        Currency::USD,
        ref_month,
    );

    // Two allocations at different times in 2025
    budget.add_reference_allocation(Allocation::new(ref_month, amount1)); // Jan 2025
    let ref_month_june = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
    budget.add_reference_allocation(Allocation::new(ref_month_june, amount2)); // Jun 2025

    // Create AnnualInflation with 10% for 2025
    let inflation_data = vec![(2025, "10.00".to_string())];
    let annual_inflation = AnnualInflation::new(inflation_data).unwrap();

    // Generate portfolio allocations for 2026 (shift = 12 months)
    budget
        .generate_portfolio_allocations(12, None, Some(&annual_inflation))
        .unwrap();

    // Both allocations should have inflation applied
    let expected_amount1 = Decimal::from_str_exact("1100.00").unwrap(); // 1000 * 1.10
    let expected_amount2 = Decimal::from_str_exact("2200.00").unwrap(); // 2000 * 1.10

    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount1);
    assert_eq!(budget.portfolio_allocations[1].amount, expected_amount2);

    // Check that total inflation is applied
    let expected_total = Decimal::from_str_exact("3300.00").unwrap();
    assert_eq!(budget.total_portfolio_allocations(), expected_total);
}

#[test]
fn test_inflation_missing_year_in_data() {
    let ref_month = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let amount = Decimal::from_str_exact("1000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Budget Missing Inflation Year".to_string(),
        amount,
        Currency::EUR,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

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
    let ref_month = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
    let amount = Decimal::from_str_exact("5000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "Complex Inflation Scenario".to_string(),
        amount,
        Currency::USD,
        ref_month,
    );

    budget.add_reference_allocation(Allocation::new(ref_month, amount));

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
    //    = 6250 * 1.0350 * 1.0420
    //    = 6250 * 1.06876 = 6680.75
    // Mais precisamente: 6250 * 1.0350 = 6468.75; 6468.75 * 1.0420 = 6740.625
    // Rounded to 2 decimal places: 6740.63
    let expected_amount = Decimal::from_str_exact("6740.63").unwrap();
    assert_eq!(budget.portfolio_allocations[0].amount, expected_amount);

    assert_eq!(
        budget.portfolio_allocations[0].month,
        NaiveDate::from_ymd_opt(2027, 3, 1).unwrap()
    );
}

#[test]
fn test_brl_budget_with_inflation_2025() {
    // Budget
    let ref_month = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
    let amount = Decimal::from_str_exact("10000.00").unwrap();
    let mut budget = BudgetAllocation::new(
        "BRL Budget with 2025 inflation".to_string(),
        amount,
        Currency::BRL,
        ref_month,
    );

    // Allocations
    let allocs = vec![
        (
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            Decimal::from_str_exact("3000.00").unwrap(),
        ),
        (
            NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            Decimal::from_str_exact("4000.00").unwrap(),
        ),
        (
            NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            Decimal::from_str_exact("2000.00").unwrap(),
        ),
        (
            NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
            Decimal::from_str_exact("1000.00").unwrap(),
        ),
    ];
    for (month, value) in &allocs {
        budget.add_reference_allocation(Allocation::new(*month, *value));
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
    for (i, (month, value)) in allocs.iter().enumerate() {
        let expected = crate::rounding::round_to_2_decimals(*value * multiplier);
        let alloc = &budget.portfolio_allocations[i];
        assert_eq!(alloc.month, *month);
        assert_eq!(
            alloc.amount, expected,
            "Incorrect adjusted allocation for {:?}",
            month
        );
    }

    // Check total
    let expected_total = allocs
        .iter()
        .map(|(_, v)| crate::rounding::round_to_2_decimals(*v * multiplier))
        .sum::<Decimal>();
    let actual_total = budget.total_portfolio_allocations();
    assert_eq!(actual_total, expected_total, "Incorrect adjusted total sum");
}
