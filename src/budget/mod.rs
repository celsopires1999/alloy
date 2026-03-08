use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::inflation::AnnualInflation;
use crate::rounding::round_to_2_decimals;

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

/// Allocation with month and value (used for Reference and Portfolio)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// Month date (always the first day of the month)
    pub month: NaiveDate,
    /// Value with decimal precision
    pub amount: Decimal,
}

impl Allocation {
    /// Creates a new allocation
    pub fn new(month: NaiveDate, amount: Decimal) -> Self {
        Self { month, amount }
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
    /// Budget description
    pub description: String,
    /// Total value with decimal precision (distributed in reference_allocations)
    pub amount: Decimal,
    /// Reference currency (EUR, USD, etc.)
    pub currency: Currency,
    /// Reference month (first day of the month)
    pub reference_month: NaiveDate,
    /// Reference allocations: the sum must be equal to `amount`
    pub reference_allocations: Vec<Allocation>,
    /// Portfolio allocations (derived from reference_allocations with shift)
    pub portfolio_allocations: Vec<Allocation>,
}

impl BudgetAllocation {
    /// Creates a new budget allocation
    pub fn new(
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

    /// Adds a reference allocation
    pub fn add_reference_allocation(&mut self, allocation: Allocation) {
        self.reference_allocations.push(allocation);
    }

    /// Returns the total of reference allocations
    pub fn total_reference_allocations(&self) -> Decimal {
        self.reference_allocations.iter().map(|a| a.amount).sum()
    }

    /// Returns the total of portfolio allocations
    pub fn total_portfolio_allocations(&self) -> Decimal {
        self.portfolio_allocations.iter().map(|a| a.amount).sum()
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
                    ref_alloc.amount * rate
                } else {
                    ref_alloc.amount
                };

                let allocation_month = Self::add_months(ref_alloc.month, shift);
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
        self.total_reference_allocations() == self.amount
    }

    /// Validates the consistency of the object
    /// Returns an error if the sum of `reference_allocations` is not equal to `amount`
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.is_consistent() {
            Ok(())
        } else {
            Err(ValidationError::InconsistentAllocations {
                expected: self.amount,
                actual: self.total_reference_allocations(),
            })
        }
    }
}

#[cfg(test)]
mod tests;
