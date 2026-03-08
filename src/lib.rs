mod budget;
pub mod inflation;
mod rounding;

pub use budget::{Allocation, BudgetAllocation, Currency};
pub use inflation::{AnnualInflation, AnnualInflationEntry, InflationError};
