use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

pub const DECIMAL_PRECISION: u32 = 18;

pub fn parse_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str_exact(s)
}

pub fn from_f64(value: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(value)
}

pub fn to_f64(d: Decimal) -> Option<f64> {
    d.to_f64()
}

pub fn from_cents(cents: i64) -> Decimal {
    Decimal::from(cents) / Decimal::from(100)
}

pub fn to_cents(d: Decimal) -> i64 {
    (d * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

pub fn from_basis_points(bps: i64) -> Decimal {
    Decimal::from(bps) / Decimal::from(10000)
}

pub fn to_basis_points(d: Decimal) -> i64 {
    (d * Decimal::from(10000)).round().to_i64().unwrap_or(0)
}

pub fn from_percentage(pct: Decimal) -> Decimal {
    pct / Decimal::ONE_HUNDRED
}

pub fn to_percentage(d: Decimal) -> Decimal {
    d * Decimal::ONE_HUNDRED
}

pub fn format_currency(d: Decimal) -> String {
    format!("${:.2}", d)
}

pub fn format_percentage(d: Decimal) -> String {
    format!("{:.2}%", to_percentage(d))
}

pub fn format_price(d: Decimal) -> String {
    format!("{:.4}", d)
}

pub fn format_cents(d: Decimal) -> String {
    format!("{:.0}", d * Decimal::from(100))
}

pub fn round_to_tick(d: Decimal, tick_size: Decimal) -> Decimal {
    if tick_size == Decimal::ZERO {
        return d;
    }
    (d / tick_size).round() * tick_size
}

pub fn round_to_precision(d: Decimal, precision: u32) -> Decimal {
    d.round_dp(precision)
}

pub fn is_within_tolerance(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    (a - b).abs() <= tolerance
}

pub fn safe_divide(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator == Decimal::ZERO {
        Decimal::ZERO
    } else {
        numerator / denominator
    }
}

pub fn clamp(value: Decimal, min: Decimal, max: Decimal) -> Decimal {
    value.max(min).min(max)
}

pub fn min(a: Decimal, b: Decimal) -> Decimal {
    a.min(b)
}

pub fn max(a: Decimal, b: Decimal) -> Decimal {
    a.max(b)
}

pub fn abs(d: Decimal) -> Decimal {
    d.abs()
}

pub fn signum(d: Decimal) -> Decimal {
    d.signum()
}

pub fn is_positive(d: Decimal) -> bool {
    d > Decimal::ZERO
}

pub fn is_negative(d: Decimal) -> bool {
    d < Decimal::ZERO
}

pub fn is_zero(d: Decimal) -> bool {
    d == Decimal::ZERO
}

pub fn sum(decimals: &[Decimal]) -> Decimal {
    decimals.iter().fold(Decimal::ZERO, |acc, d| acc + d)
}

pub fn average(decimals: &[Decimal]) -> Decimal {
    if decimals.is_empty() {
        return Decimal::ZERO;
    }
    safe_divide(sum(decimals), Decimal::from(decimals.len()))
}

pub fn weighted_average(values: &[(Decimal, Decimal)]) -> Decimal {
    let total_weight: Decimal = values.iter().map(|(_, w)| *w).sum();
    if total_weight == Decimal::ZERO {
        return Decimal::ZERO;
    }
    let weighted_sum: Decimal = values.iter().map(|(v, w)| *v * *w).sum();
    weighted_sum / total_weight
}

pub fn calculate_profit(buy_price: Decimal, sell_price: Decimal, quantity: Decimal) -> Decimal {
    (sell_price - buy_price) * quantity
}

pub fn calculate_profit_percentage(buy_price: Decimal, sell_price: Decimal) -> Decimal {
    if buy_price == Decimal::ZERO {
        return Decimal::ZERO;
    }
    safe_divide(sell_price - buy_price, buy_price) * Decimal::ONE_HUNDRED
}

pub fn calculate_return_on_investment(profit: Decimal, investment: Decimal) -> Decimal {
    if investment == Decimal::ZERO {
        return Decimal::ZERO;
    }
    safe_divide(profit, investment) * Decimal::ONE_HUNDRED
}

pub fn calculate_compound_annual_growth_rate(
    start_value: Decimal,
    end_value: Decimal,
    years: Decimal,
) -> Decimal {
    if start_value == Decimal::ZERO || years == Decimal::ZERO {
        return Decimal::ZERO;
    }
    let ratio = safe_divide(end_value, start_value);
    let exponent = safe_divide(Decimal::ONE, years);
    let result = ratio.powd(exponent);
    (result - Decimal::ONE) * Decimal::ONE_HUNDRED
}

pub fn kalshi_price_to_decimal(kalshi_price: i64) -> Decimal {
    Decimal::from(kalshi_price) / Decimal::from(100)
}

pub fn decimal_to_kalshi_price(d: Decimal) -> i64 {
    (d * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

pub fn polymarket_price_to_decimal(price_str: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str_exact(price_str)
}

pub fn decimal_to_polymarket_price(d: Decimal) -> String {
    d.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        assert_eq!(
            parse_decimal("123.45").unwrap(),
            Decimal::from_str("123.45").unwrap()
        );
        assert!(parse_decimal("invalid").is_err());
    }

    #[test]
    fn test_from_to_cents() {
        assert_eq!(from_cents(12345), Decimal::from_str("123.45").unwrap());
        assert_eq!(to_cents(Decimal::from_str("123.45").unwrap()), 12345);
    }

    #[test]
    fn test_from_to_basis_points() {
        assert_eq!(from_basis_points(100), Decimal::from_str("0.01").unwrap());
        assert_eq!(to_basis_points(Decimal::from_str("0.01").unwrap()), 100);
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(Decimal::from(123)), "$123.00");
        assert_eq!(
            format_currency(Decimal::from_str("123.456").unwrap()),
            "$123.46"
        );
    }

    #[test]
    fn test_round_to_tick() {
        let tick = Decimal::from_str("0.01").unwrap();
        assert_eq!(
            round_to_tick(Decimal::from_str("123.456").unwrap(), tick),
            Decimal::from_str("123.46").unwrap()
        );
    }

    #[test]
    fn test_is_within_tolerance() {
        let a = Decimal::from_str("10.0").unwrap();
        let b = Decimal::from_str("10.1").unwrap();
        let tolerance = Decimal::from_str("0.2").unwrap();
        assert!(is_within_tolerance(a, b, tolerance));
    }

    #[test]
    fn test_safe_divide() {
        assert_eq!(
            safe_divide(Decimal::from(10), Decimal::from(2)),
            Decimal::from(5)
        );
        assert_eq!(safe_divide(Decimal::from(10), Decimal::ZERO), Decimal::ZERO);
    }

    #[test]
    fn test_sum_and_average() {
        let values = vec![Decimal::from(1), Decimal::from(2), Decimal::from(3)];
        assert_eq!(sum(&values), Decimal::from(6));
        assert_eq!(average(&values), Decimal::from(2));
    }

    #[test]
    fn test_calculate_profit() {
        let profit = calculate_profit(
            Decimal::from_str("0.45").unwrap(),
            Decimal::from_str("0.55").unwrap(),
            Decimal::from(100),
        );
        assert_eq!(profit, Decimal::from(10));
    }

    #[test]
    fn test_kalshi_price_conversions() {
        assert_eq!(
            kalshi_price_to_decimal(45),
            Decimal::from_str("0.45").unwrap()
        );
        assert_eq!(
            decimal_to_kalshi_price(Decimal::from_str("0.45").unwrap()),
            45
        );
    }
}
