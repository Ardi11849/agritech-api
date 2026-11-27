use rust_decimal::Decimal;

/// Helper types and mathematical precision functions for agricultural and financial calculations
pub type Money = Decimal;
pub type Weight = Decimal;

pub fn round_to_two_decimals(val: Decimal) -> Decimal {
    val.round_dp(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_precision_rounding() {
        let val = Decimal::from_str("123.4567").expect("Valid decimal");
        let expected = Decimal::from_str("123.46").expect("Valid decimal");
        assert_eq!(round_to_two_decimals(val), expected);
    }
}
