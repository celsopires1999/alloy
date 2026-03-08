use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::inflation::AnnualInflation;
use crate::rounding::{format_amount, round_to_2_decimals};

/// BudgetAllocation validation error
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// The sum of allocations is not equal to the total amount
    InconsistentAllocations { expected: Decimal, actual: Decimal },
    /// Exchange rate with more than 4 decimal places
    ExchangeRateInvalidScale { scale: u32 },
    /// Allocation month is before reference month (retroactive allocation)
    RetroactiveAllocation {
        allocation_year: u32,
        reference_year: u32,
    },
    /// Inflation calculation error
    InflationCalculationError(String),
    /// Invalid date format
    InvalidDateFormat(String),
    /// Invalid amount format
    InvalidAmountFormat(String),
}

// Helper functions for validation
fn validate_date_format(date_str: &str) -> Result<NaiveDate, ValidationError> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        ValidationError::InvalidDateFormat(format!(
            "date '{}' must be in YYYY-MM-DD format: {}",
            date_str, e
        ))
    })?;

    // Check if it's the first day of the month
    if date.day() != 1 {
        return Err(ValidationError::InvalidDateFormat(format!(
            "date '{}' must be the first day of the month",
            date_str
        )));
    }

    Ok(date)
}

fn validate_amount_format(amount_str: &str) -> Result<Decimal, ValidationError> {
    let amount = Decimal::from_str_exact(amount_str).map_err(|e| {
        ValidationError::InvalidAmountFormat(format!(
            "amount '{}' must be a valid decimal number with at most 2 decimal places: {}",
            amount_str, e
        ))
    })?;

    // Check scale (max 2 decimal places)
    if amount.scale() > 2 {
        return Err(ValidationError::InvalidAmountFormat(format!(
            "amount '{}' has {} decimal places, maximum 2 allowed",
            amount_str,
            amount.scale()
        )));
    }

    Ok(amount)
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InconsistentAllocations { expected, actual } => {
                write!(
                    f,
                    "Inconsistent allocations: expected {}, but got {}",
                    expected, actual
                )
            }
            ValidationError::ExchangeRateInvalidScale { scale } => {
                write!(
                    f,
                    "Exchange rate has {} decimal places, maximum 4 allowed",
                    scale
                )
            }
            ValidationError::RetroactiveAllocation {
                allocation_year,
                reference_year,
            } => {
                write!(
                    f,
                    "Retroactive allocation not allowed: allocation year {} is before reference year {}",
                    allocation_year, reference_year
                )
            }
            ValidationError::InflationCalculationError(msg) => {
                write!(f, "Inflation calculation error: {}", msg)
            }
            ValidationError::InvalidDateFormat(msg) => {
                write!(f, "Invalid date format: {}", msg)
            }
            ValidationError::InvalidAmountFormat(msg) => {
                write!(f, "Invalid amount format: {}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Supported currency (unique for the entire company portfolio)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    EUR,
    USD,
    GBP,
    JPY,
    CHF,
    CAD,
    AUD,
    BRL,
}

impl Currency {
    /// Returns the currency symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::EUR => "€",
            Currency::USD => "$",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CHF => "Fr",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
            Currency::BRL => "R$",
        }
    }

    /// Returns the ISO code of the currency
    pub fn code(&self) -> &'static str {
        match self {
            Currency::EUR => "EUR",
            Currency::USD => "USD",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CHF => "CHF",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::BRL => "BRL",
        }
    }
}

/// Builder for Allocation with string-based inputs
pub struct AllocationBuilder {
    month: Option<String>,
    amount: Option<String>,
}

impl AllocationBuilder {
    /// Creates a new AllocationBuilder
    pub fn new() -> Self {
        Self {
            month: None,
            amount: None,
        }
    }

    /// Sets the month (format: YYYY-MM-DD, must be first day of month)
    pub fn with_month(mut self, month: &str) -> Self {
        self.month = Some(month.to_string());
        self
    }

    /// Sets the amount (format: decimal with ".", max 2 decimal places)
    pub fn with_amount(mut self, amount: &str) -> Self {
        self.amount = Some(amount.to_string());
        self
    }

    /// Builds the Allocation, validating input strings
    pub fn build(self) -> Result<Allocation, ValidationError> {
        let month_str = self
            .month
            .ok_or_else(|| ValidationError::InvalidDateFormat("month is required".to_string()))?;
        let amount_str = self.amount.ok_or_else(|| {
            ValidationError::InvalidAmountFormat("amount is required".to_string())
        })?;

        let month = validate_date_format(&month_str)?;
        let amount = validate_amount_format(&amount_str)?;

        Ok(Allocation { month, amount })
    }
}

impl Default for AllocationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocation with month and value (used for Reference and Portfolio)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// Month date (always the first day of the month) - PRIVATE
    month: NaiveDate,
    /// Value with decimal precision - PRIVATE
    amount: Decimal,
}

impl Allocation {
    /// Creates a new allocation using a builder
    pub fn builder() -> AllocationBuilder {
        AllocationBuilder::new()
    }

    /// (Private) Creates a new allocation with internal types
    fn new(month: NaiveDate, amount: Decimal) -> Self {
        Self { month, amount }
    }

    /// Returns the month as a string in format YYYY-MM-DD
    pub fn get_month(&self) -> String {
        self.month.format("%Y-%m-%d").to_string()
    }

    /// Returns the amount as a string with decimal separator "." and exactly 2 decimal places
    pub fn get_amount(&self) -> String {
        format_amount(self.amount)
    }

    /// (Internal) Gets the month as NaiveDate
    pub(crate) fn month_internal(&self) -> NaiveDate {
        self.month
    }

    /// (Internal) Gets the amount as Decimal
    pub(crate) fn amount_internal(&self) -> Decimal {
        self.amount
    }
}

/// Main entity: Budget Allocation
///
/// Represents a budget with reference and portfolio allocations.
/// - The total `amount` is distributed ONLY in `reference_allocations`
/// - `portfolio_allocations` are derived from `reference_allocations` with a month shift via `generate_portfolio_allocations()`
/// - The reference currency can be EUR, USD, etc.
/// - The portfolio currency is unique for the entire company (documented in portfolio_allocations)
/// - `exchange_rate` is optional (maximum 4 decimal places) and used exclusively in `generate_portfolio_allocations()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// Budget description - PRIVATE
    description: String,
    /// Total value with decimal precision (distributed in reference_allocations) - PRIVATE
    amount: Decimal,
    /// Reference currency (EUR, USD, etc.) - PRIVATE
    currency: Currency,
    /// Reference month (first day of the month) - PRIVATE
    reference_month: NaiveDate,
    /// Reference allocations: the sum must be equal to `amount` - PRIVATE
    reference_allocations: Vec<Allocation>,
    /// Portfolio allocations (derived from reference_allocations with shift) - PRIVATE
    portfolio_allocations: Vec<Allocation>,
}

/// Builder for BudgetAllocation with string-based inputs
pub struct BudgetAllocationBuilder {
    description: Option<String>,
    amount: Option<String>,
    currency: Option<Currency>,
    reference_month: Option<String>,
}

impl BudgetAllocationBuilder {
    /// Creates a new BudgetAllocationBuilder
    pub fn new() -> Self {
        Self {
            description: None,
            amount: None,
            currency: None,
            reference_month: None,
        }
    }

    /// Sets the description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Sets the amount (format: decimal with ".", max 2 decimal places)
    pub fn with_amount(mut self, amount: &str) -> Self {
        self.amount = Some(amount.to_string());
        self
    }

    /// Sets the currency
    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = Some(currency);
        self
    }

    /// Sets the reference month (format: YYYY-MM-DD, must be first day of month)
    pub fn with_reference_month(mut self, reference_month: &str) -> Self {
        self.reference_month = Some(reference_month.to_string());
        self
    }

    /// Builds the BudgetAllocation, validating input strings
    pub fn build(self) -> Result<BudgetAllocation, ValidationError> {
        let description = self.description.ok_or_else(|| {
            ValidationError::InvalidDateFormat("description is required".to_string())
        })?;
        let amount_str = self.amount.ok_or_else(|| {
            ValidationError::InvalidAmountFormat("amount is required".to_string())
        })?;
        let currency = self.currency.ok_or_else(|| {
            ValidationError::InvalidDateFormat("currency is required".to_string())
        })?;
        let reference_month_str = self.reference_month.ok_or_else(|| {
            ValidationError::InvalidDateFormat("reference_month is required".to_string())
        })?;

        let amount = validate_amount_format(&amount_str)?;
        let reference_month = validate_date_format(&reference_month_str)?;

        Ok(BudgetAllocation {
            description,
            amount,
            currency,
            reference_month,
            reference_allocations: Vec::new(),
            portfolio_allocations: Vec::new(),
        })
    }
}

impl Default for BudgetAllocationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BudgetAllocation {
    /// Creates a new budget allocation using a builder
    pub fn builder() -> BudgetAllocationBuilder {
        BudgetAllocationBuilder::new()
    }

    /// (Private) Creates a new budget allocation with internal types - used internally
    #[allow(dead_code)]
    fn new(
        description: String,
        amount: Decimal,
        currency: Currency,
        reference_month: NaiveDate,
    ) -> Self {
        Self {
            description,
            amount,
            currency,
            reference_month,
            reference_allocations: Vec::new(),
            portfolio_allocations: Vec::new(),
        }
    }

    /// Returns the description
    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    /// Returns the amount as a string with decimal separator "." and exactly 2 decimal places
    pub fn get_amount(&self) -> String {
        format_amount(self.amount)
    }

    /// Returns the currency
    pub fn get_currency(&self) -> Currency {
        self.currency
    }

    /// Returns the reference month as a string in format YYYY-MM-DD
    pub fn get_reference_month(&self) -> String {
        self.reference_month.format("%Y-%m-%d").to_string()
    }

    /// Returns all reference allocations (public access to Vec of Allocations)
    pub fn get_reference_allocations(&self) -> &[Allocation] {
        &self.reference_allocations
    }

    /// Returns all portfolio allocations (public access to Vec of Allocations)
    pub fn get_portfolio_allocations(&self) -> &[Allocation] {
        &self.portfolio_allocations
    }

    /// Adds a reference allocation
    pub fn add_reference_allocation(&mut self, allocation: Allocation) {
        self.reference_allocations.push(allocation);
    }

    /// Returns the total of reference allocations as a string with exactly 2 decimal places
    pub fn total_reference_allocations(&self) -> String {
        let total: Decimal = self
            .reference_allocations
            .iter()
            .map(|a| a.amount_internal())
            .sum();
        format_amount(total)
    }

    /// Returns the total of portfolio allocations as a string with exactly 2 decimal places
    pub fn total_portfolio_allocations(&self) -> String {
        let total: Decimal = self
            .portfolio_allocations
            .iter()
            .map(|a| a.amount_internal())
            .sum();
        format_amount(total)
    }

    /// Calculates the portfolio_allocation month from a reference month
    /// Adds `shift` months to the given month
    fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
        let (year, month, day) = (date.year(), date.month(), date.day());
        let total_months = (year as i32 * 12 + month as i32 - 1) + months;
        let new_year = total_months.div_euclid(12) as i32;
        let new_month = total_months.rem_euclid(12) + 1;
        NaiveDate::from_ymd_opt(new_year, new_month as u32, day as u32).unwrap()
    }

    /// Generates `portfolio_allocations` based on `reference_allocations` and `shift`
    /// Optionally applies `exchange_rate` to the values (maximum 4 decimal places)
    /// Each portfolio_allocation will have the month shifted by `shift` months
    /// If `exchange_rate` is provided, the value will be multiplied by the rate
    /// If `annual_inflation` is provided, inflation will be applied based on year differences
    pub fn generate_portfolio_allocations(
        &mut self,
        shift: i32,
        exchange_rate: Option<Decimal>,
        annual_inflation: Option<&AnnualInflation>,
    ) -> Result<(), ValidationError> {
        // Validate exchange_rate if provided
        if let Some(rate) = exchange_rate {
            if rate.scale() > 4 {
                return Err(ValidationError::ExchangeRateInvalidScale {
                    scale: rate.scale(),
                });
            }
        }

        let ref_year = self.reference_month.year();

        self.portfolio_allocations = self
            .reference_allocations
            .iter()
            .map(|ref_alloc| {
                let mut amount = if let Some(rate) = exchange_rate {
                    ref_alloc.amount_internal() * rate
                } else {
                    ref_alloc.amount_internal()
                };

                let allocation_month = Self::add_months(ref_alloc.month_internal(), shift);
                let alloc_year = allocation_month.year();

                // Check for retroactive allocation
                if alloc_year < ref_year {
                    return Err(ValidationError::RetroactiveAllocation {
                        allocation_year: alloc_year as u32,
                        reference_year: ref_year as u32,
                    });
                }

                // Apply inflation if needed
                if alloc_year > ref_year {
                    if let Some(inflation) = annual_inflation {
                        let start_year = ref_year as u32;
                        let end_year = (alloc_year - 1) as u32;

                        let multiplier = inflation
                            .calculate_multiplier(start_year, end_year)
                            .map_err(|e| {
                                ValidationError::InflationCalculationError(e.to_string())
                            })?;

                        amount = amount * multiplier;
                    }
                }

                // Round amount to 2 decimal places with RoundHalfUp strategy
                amount = round_to_2_decimals(amount);

                Ok(Allocation::new(allocation_month, amount))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    /// Clears the portfolio allocations
    pub fn clear_portfolio_allocations(&mut self) {
        self.portfolio_allocations.clear();
    }

    /// Checks if the object is consistent:
    /// - The sum of `reference_allocations` must be equal to `amount`
    pub fn is_consistent(&self) -> bool {
        let total: Decimal = self
            .reference_allocations
            .iter()
            .map(|a| a.amount_internal())
            .sum();
        total == self.amount
    }

    /// Validates the consistency of the object
    /// Returns an error if the sum of `reference_allocations` is not equal to `amount`
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.is_consistent() {
            Ok(())
        } else {
            let actual: Decimal = self
                .reference_allocations
                .iter()
                .map(|a| a.amount_internal())
                .sum();
            Err(ValidationError::InconsistentAllocations {
                expected: self.amount,
                actual,
            })
        }
    }
}

#[cfg(test)]
mod tests;
