use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use tracing::{debug, trace};

use crate::domain::Side;

#[derive(Debug, Clone)]
pub struct ArbitrageCalculation {
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub entry_side: Side,
    pub exit_side: Side,
    pub gross_profit: Decimal,
    pub net_profit: Decimal,
    pub profit_margin: Decimal,
    pub kalshi_fee: Decimal,
    pub amount: Decimal,
    pub total_cost: Decimal,
    pub guaranteed_return: Decimal,
}

pub struct ArbitrageCalculator {
    min_profit_margin: Decimal,
    kalshi_fee_rate: Decimal,
}

impl ArbitrageCalculator {
    pub fn new(min_profit_margin: Decimal, kalshi_fee_rate: Decimal) -> Self {
        Self {
            min_profit_margin,
            kalshi_fee_rate,
        }
    }

    pub fn calculate(
        &self,
        kalshi_price: Decimal,
        poly_price: Decimal,
        side: Side,
    ) -> Option<ArbitrageCalculation> {
        if !self.validate_price(kalshi_price) || !self.validate_price(poly_price) {
            trace!(
                kalshi_price = %kalshi_price,
                poly_price = %poly_price,
                "Invalid prices for calculation"
            );
            return None;
        }

        let implied_prob_sum = kalshi_price + poly_price;
        if implied_prob_sum >= Decimal::ONE {
            trace!(
                implied_prob_sum = %implied_prob_sum,
                "No arbitrage: implied probability sum >= 1"
            );
            return None;
        }

        let amount = Decimal::ONE_HUNDRED;
        let kalshi_bet = amount * kalshi_price;
        let poly_bet = amount * poly_price;
        let kalshi_fee = self.calculate_kalshi_fee(kalshi_price, side);
        let total_cost = kalshi_bet + poly_bet + kalshi_fee;
        let guaranteed_return = amount;
        let gross_profit = guaranteed_return - kalshi_bet - poly_bet;
        let net_profit = guaranteed_return - total_cost;
        let profit_margin = if total_cost > Decimal::ZERO {
            (net_profit / total_cost) * Decimal::ONE_HUNDRED
        } else {
            Decimal::ZERO
        };

        if profit_margin < self.min_profit_margin {
            debug!(
                profit_margin = %profit_margin,
                min_profit_margin = %self.min_profit_margin,
                "Profit margin below threshold"
            );
            return None;
        }

        let (entry_price, exit_price, entry_side, exit_side) = match side {
            Side::Yes => (kalshi_price, poly_price, Side::Yes, Side::No),
            Side::No => (kalshi_price, poly_price, Side::No, Side::Yes),
        };

        debug!(
            kalshi_price = %kalshi_price,
            poly_price = %poly_price,
            profit_margin = %profit_margin,
            net_profit = %net_profit,
            "Arbitrage opportunity found"
        );

        Some(ArbitrageCalculation {
            entry_price,
            exit_price,
            entry_side,
            exit_side,
            gross_profit,
            net_profit,
            profit_margin,
            kalshi_fee,
            amount,
            total_cost,
            guaranteed_return,
        })
    }

    pub fn calculate_profit(
        &self,
        entry_price: Decimal,
        exit_price: Decimal,
        amount: Decimal,
    ) -> Decimal {
        if entry_price <= Decimal::ZERO || exit_price <= Decimal::ZERO || amount <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let implied_prob_sum = entry_price + exit_price;
        if implied_prob_sum >= Decimal::ONE {
            return Decimal::ZERO;
        }

        let entry_cost = amount * entry_price;
        let exit_cost = amount * exit_price;
        let total_cost = entry_cost + exit_cost;
        let guaranteed_return = amount;

        guaranteed_return - total_cost
    }

    pub fn calculate_kalshi_fee(&self, price: Decimal, side: Side) -> Decimal {
        if price <= Decimal::ZERO || price >= Decimal::ONE {
            return Decimal::ZERO;
        }

        let contracts = Decimal::ONE_HUNDRED;
        let raw_fee = self.kalshi_fee_rate * contracts * price * (Decimal::ONE - price);

        let fee_cents = raw_fee * Decimal::ONE_HUNDRED;
        let rounded_fee_cents = fee_cents.ceil();
        rounded_fee_cents / Decimal::ONE_HUNDRED
    }

    fn validate_price(&self, price: Decimal) -> bool {
        let min_price = Decimal::new(1, 2);
        let max_price = Decimal::new(99, 2);
        price >= min_price && price <= max_price
    }

    pub fn calculate_profit_margin(&self, net_profit: Decimal, total_cost: Decimal) -> Decimal {
        if total_cost <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        (net_profit / total_cost) * Decimal::ONE_HUNDRED
    }

    pub fn is_profitable(&self, profit_margin: Decimal) -> bool {
        profit_margin >= self.min_profit_margin
    }
}

impl Default for ArbitrageCalculator {
    fn default() -> Self {
        Self::new(Decimal::new(5, 2), Decimal::new(7, 2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_calculate_kalshi_fee() {
        let calc = ArbitrageCalculator::default();

        let fee = calc.calculate_kalshi_fee(dec("0.5"), Side::Yes);
        assert!(fee >= dec("1.75") && fee <= dec("1.77"), "fee was {}", fee);

        let fee_zero = calc.calculate_kalshi_fee(Decimal::ZERO, Side::Yes);
        assert_eq!(fee_zero, Decimal::ZERO);

        let fee_one = calc.calculate_kalshi_fee(Decimal::ONE, Side::Yes);
        assert_eq!(fee_one, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_profit() {
        let calc = ArbitrageCalculator::default();

        let profit = calc.calculate_profit(dec("0.45"), dec("0.50"), dec("100"));
        assert!(profit > Decimal::ZERO);

        let no_profit = calc.calculate_profit(dec("0.60"), dec("0.50"), dec("100"));
        assert_eq!(no_profit, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_opportunity() {
        let calc = ArbitrageCalculator::new(dec("1.0"), dec("0.07"));

        let result = calc.calculate(dec("0.45"), dec("0.50"), Side::Yes);
        assert!(result.is_some());

        let opp = result.unwrap();
        assert!(opp.profit_margin > Decimal::ZERO);
        assert!(opp.net_profit > Decimal::ZERO);
    }

    #[test]
    fn test_no_opportunity_when_sum_exceeds_one() {
        let calc = ArbitrageCalculator::default();

        let result = calc.calculate(dec("0.60"), dec("0.50"), Side::Yes);
        assert!(result.is_none());
    }

    #[test]
    fn test_profit_margin_below_threshold() {
        let calc = ArbitrageCalculator::new(dec("10.0"), dec("0.07"));

        let result = calc.calculate(dec("0.49"), dec("0.50"), Side::Yes);
        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_profit_margin() {
        let calc = ArbitrageCalculator::default();

        let margin = calc.calculate_profit_margin(dec("5"), dec("100"));
        assert_eq!(margin, dec("5"));

        let margin_zero = calc.calculate_profit_margin(dec("5"), Decimal::ZERO);
        assert_eq!(margin_zero, Decimal::ZERO);
    }

    #[test]
    fn test_validate_price() {
        let calc = ArbitrageCalculator::default();

        assert!(calc.validate_price(dec("0.5")));
        assert!(calc.validate_price(dec("0.01")));
        assert!(calc.validate_price(dec("0.99")));
        assert!(!calc.validate_price(Decimal::ZERO));
        assert!(!calc.validate_price(Decimal::ONE));
        assert!(!calc.validate_price(dec("1.5")));
    }
}
