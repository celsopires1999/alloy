/// Rounding utilities for precise decimal calculations
///
/// Centralizes rounding strategy across the application to ensure consistency
/// in financial calculations and allocations.
use rust_decimal::Decimal;

/// Formats a Decimal value as a string with exactly 2 decimal places
///
/// Ensures consistent string representation for API responses.
/// Example: Decimal(1095) → "1095.00", Decimal(1095.5) → "1095.50"
///
/// # Arguments
/// * `amount` - The Decimal value to format
///
/// # Returns
/// * String representation with exactly 2 decimal places
pub fn format_amount(amount: Decimal) -> String {
    format!("{:.2}", amount)
}

/// Rounds a Decimal value to 2 decimal places using RoundHalfUp strategy
///
/// This is the standard commercial rounding: 0.5 and above rounds up.
/// Example: 1.125 → 1.13, 1.124 → 1.12
///
/// # Arguments
/// * `amount` - The Decimal value to round
///
/// # Returns
/// * Rounded Decimal value with exactly 2 decimal places
pub fn round_to_2_decimals(amount: Decimal) -> Decimal {
    let scaled = amount * Decimal::from(100);
    let rounded = (scaled + Decimal::from_str_exact("0.5").unwrap()).trunc();
    rounded / Decimal::from(100)
}

/// Rounds a Decimal value to 4 decimal places using RoundHalfUp strategy
///
/// This is the standard commercial rounding: 0.5 and above rounds up.
/// Used for inflation multipliers and intermediate calculations.
/// Example: 1.03505 → 1.0351, 1.03504 → 1.0350
///
/// # Arguments
/// * `multiplier` - The Decimal value to round
///
/// # Returns
/// * Rounded Decimal value with exactly 4 decimal places
pub fn round_to_4_decimals(multiplier: Decimal) -> Decimal {
    let scaled = multiplier * Decimal::from(10000);
    let rounded = (scaled + Decimal::from_str_exact("0.5").unwrap()).trunc();
    rounded / Decimal::from(10000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_to_2_decimals_round_half_up() {
        // 0.5 rounds up
        assert_eq!(
            round_to_2_decimals(Decimal::from_str_exact("1.125").unwrap()),
            Decimal::from_str_exact("1.13").unwrap()
        );

        // Below 0.5 rounds down
        assert_eq!(
            round_to_2_decimals(Decimal::from_str_exact("1.124").unwrap()),
            Decimal::from_str_exact("1.12").unwrap()
        );

        // Already has 2 decimals
        assert_eq!(
            round_to_2_decimals(Decimal::from_str_exact("1.23").unwrap()),
            Decimal::from_str_exact("1.23").unwrap()
        );

        // Complex case from inflation + exchange rate
        assert_eq!(
            round_to_2_decimals(Decimal::from_str_exact("6740.625").unwrap()),
            Decimal::from_str_exact("6740.63").unwrap()
        );
    }

    #[test]
    fn test_round_to_4_decimals_round_half_up() {
        // 0.5 rounds up
        assert_eq!(
            round_to_4_decimals(Decimal::from_str_exact("1.03505").unwrap()),
            Decimal::from_str_exact("1.0351").unwrap()
        );

        // Below 0.5 rounds down
        assert_eq!(
            round_to_4_decimals(Decimal::from_str_exact("1.03504").unwrap()),
            Decimal::from_str_exact("1.0350").unwrap()
        );

        // Already has 4 decimals
        assert_eq!(
            round_to_4_decimals(Decimal::from_str_exact("1.2345").unwrap()),
            Decimal::from_str_exact("1.2345").unwrap()
        );

        // Multiplier product: 1.0350 * 1.0420 = 1.07847, rounded to 1.0785
        let multiplier =
            Decimal::from_str_exact("1.0350").unwrap() * Decimal::from_str_exact("1.0420").unwrap();
        assert_eq!(
            round_to_4_decimals(multiplier),
            Decimal::from_str_exact("1.0785").unwrap()
        );
    }
}
