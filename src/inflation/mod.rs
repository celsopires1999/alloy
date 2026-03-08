use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::rounding::{format_amount, round_to_4_decimals};

/// AnnualInflation validation error
#[derive(Debug, Clone, PartialEq)]
pub enum InflationError {
    /// Year not found in the inflation list
    YearNotFound(u32),
    /// Years are not in ascending order
    YearsNotOrdered,
    /// Invalid inflation value (must be positive)
    InvalidInflationValue { year: u32, reason: String },
}

impl fmt::Display for InflationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InflationError::YearNotFound(year) => {
                write!(f, "Year {} not found in inflation data", year)
            }
            InflationError::YearsNotOrdered => {
                write!(f, "Years are not in ascending order")
            }
            InflationError::InvalidInflationValue { year, reason } => {
                write!(f, "Invalid inflation value for year {}: {}", year, reason)
            }
        }
    }
}

impl std::error::Error for InflationError {}

/// Builder for AnnualInflationEntry with string-based inputs
pub struct AnnualInflationEntryBuilder {
    year: Option<u32>,
    inflation: Option<String>,
}

impl AnnualInflationEntryBuilder {
    /// Creates a new AnnualInflationEntryBuilder
    pub fn new() -> Self {
        Self {
            year: None,
            inflation: None,
        }
    }

    /// Sets the year
    pub fn with_year(mut self, year: u32) -> Self {
        self.year = Some(year);
        self
    }

    /// Sets the inflation rate (format: decimal with "." separator)
    pub fn with_inflation(mut self, inflation: &str) -> Self {
        self.inflation = Some(inflation.to_string());
        self
    }

    /// Builds the AnnualInflationEntry, validating input strings
    pub fn build(self) -> Result<AnnualInflationEntry, InflationError> {
        let year = self
            .year
            .ok_or_else(|| InflationError::InvalidInflationValue {
                year: 0,
                reason: "year is required".to_string(),
            })?;
        let inflation_str =
            self.inflation
                .ok_or_else(|| InflationError::InvalidInflationValue {
                    year,
                    reason: "inflation is required".to_string(),
                })?;

        let inflation = Decimal::from_str_exact(&inflation_str).map_err(|_| {
            InflationError::InvalidInflationValue {
                year,
                reason: format!("Failed to parse '{}' as decimal", inflation_str),
            }
        })?;

        if inflation < Decimal::ZERO {
            return Err(InflationError::InvalidInflationValue {
                year,
                reason: "Inflation value must be non-negative".to_string(),
            });
        }

        Ok(AnnualInflationEntry { year, inflation })
    }
}

impl Default for AnnualInflationEntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Annual inflation entry (year and inflation rate)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnualInflationEntry {
    /// Year of the inflation - PRIVATE
    year: u32,
    /// Inflation rate as Decimal (e.g., 1.22 for 1.22%) - PRIVATE
    inflation: Decimal,
}

impl AnnualInflationEntry {
    /// Creates a new annual inflation entry using a builder
    pub fn builder() -> AnnualInflationEntryBuilder {
        AnnualInflationEntryBuilder::new()
    }

    /// (Private) Creates a new annual inflation entry with internal types
    #[allow(dead_code)]
    fn new(year: u32, inflation: Decimal) -> Self {
        Self { year, inflation }
    }

    /// Returns the year
    pub fn get_year(&self) -> u32 {
        self.year
    }

    /// Returns the inflation rate as a string
    pub fn get_inflation(&self) -> String {
        format_amount(self.inflation)
    }

    /// (Internal) Gets the year value
    pub(crate) fn year_internal(&self) -> u32 {
        self.year
    }

    /// (Internal) Gets the inflation as Decimal
    pub(crate) fn inflation_internal(&self) -> Decimal {
        self.inflation
    }
}

/// Builder for AnnualInflation with string-based inputs
pub struct AnnualInflationBuilder {
    entries: Vec<(u32, String)>,
}

impl AnnualInflationBuilder {
    /// Creates a new AnnualInflationBuilder
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an inflation entry as (year, inflation_string)
    pub fn add_entry(mut self, year: u32, inflation: &str) -> Self {
        self.entries.push((year, inflation.to_string()));
        self
    }

    /// Builds the AnnualInflation, validating all entries
    pub fn build(self) -> Result<AnnualInflation, InflationError> {
        AnnualInflation::new(self.entries)
    }
}

impl Default for AnnualInflationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Annual Inflation Entity
///
/// Stores a list of year/inflation pairs and provides a method to calculate
/// the multiplication index between two years.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualInflation {
    /// List of annual inflation entries - PRIVATE
    entries: Vec<AnnualInflationEntry>,
}

impl AnnualInflation {
    /// Creates a new annual inflation builder
    pub fn builder() -> AnnualInflationBuilder {
        AnnualInflationBuilder::new()
    }

    /// Creates a new annual inflation entity with parameterizable data
    ///
    /// # Arguments
    /// * `entries` - Vector of tuples (year, inflation_string) where inflation_string is a percentage value
    ///
    /// # Returns
    /// * `Result<AnnualInflation, InflationError>` - The created entity or validation error
    pub fn new(entries: Vec<(u32, String)>) -> Result<Self, InflationError> {
        let mut parsed_entries: Vec<AnnualInflationEntry> = Vec::new();

        // Convert strings to Decimal and validate
        for (year, inflation_str) in entries {
            let inflation = Decimal::from_str_exact(&inflation_str).map_err(|_| {
                InflationError::InvalidInflationValue {
                    year,
                    reason: format!("Failed to parse '{}' as decimal", inflation_str),
                }
            })?;

            // Validate that the value is positive
            if inflation < Decimal::ZERO {
                return Err(InflationError::InvalidInflationValue {
                    year,
                    reason: "Inflation value must be non-negative".to_string(),
                });
            }

            parsed_entries.push(AnnualInflationEntry { year, inflation });
        }

        // Validate that the years are in ascending order
        for i in 1..parsed_entries.len() {
            if parsed_entries[i].year <= parsed_entries[i - 1].year {
                return Err(InflationError::YearsNotOrdered);
            }
        }

        Ok(AnnualInflation {
            entries: parsed_entries,
        })
    }

    /// Calculates the multiplication index between two years
    ///
    /// Calculates (1 + i1/100) * (1 + i2/100) * ... * (1 + in/100)
    /// where i1, i2, ..., in are the inflations for the years in the interval [start_year, end_year]
    ///
    /// # Arguments
    /// * `start_year` - Start year (inclusive)
    /// * `end_year` - End year (inclusive)
    ///
    /// # Returns
    /// * `Result<Decimal, InflationError>` - The multiplication index rounded to 4 decimal places,
    ///   or error if any year is not found
    pub fn calculate_multiplier(
        &self,
        start_year: u32,
        end_year: u32,
    ) -> Result<Decimal, InflationError> {
        // Validate that both years exist
        if !self.entries.iter().any(|e| e.year_internal() == start_year) {
            return Err(InflationError::YearNotFound(start_year));
        }
        if !self.entries.iter().any(|e| e.year_internal() == end_year) {
            return Err(InflationError::YearNotFound(end_year));
        }

        // Calculate the multiplier
        let mut multiplier = Decimal::ONE;
        for entry in &self.entries {
            if entry.year_internal() >= start_year && entry.year_internal() <= end_year {
                // Multiply by (1 + inflation/100)
                let factor = Decimal::ONE + (entry.inflation_internal() / Decimal::from(100));
                multiplier *= factor;
            }
        }

        // Round to 4 decimal places with RoundHalfUp strategy
        let rounded = round_to_4_decimals(multiplier);

        Ok(rounded)
    }

    /// Returns the list of inflation entries
    pub fn entries(&self) -> &[AnnualInflationEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests;
