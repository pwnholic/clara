use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tracing::{debug, info, trace, warn};

use crate::domain::{KalshiMarket, MatchedMarket, PolymarketMarket};

static NBA_TEAM_MAPPINGS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.insert("ATLANTA HAWKS", "ATL");
    m.insert("HAWKS", "ATL");
    m.insert("ATL", "ATL");

    m.insert("BOSTON CELTICS", "BOS");
    m.insert("CELTICS", "BOS");
    m.insert("BOS", "BOS");

    m.insert("BROOKLYN NETS", "BKN");
    m.insert("NETS", "BKN");
    m.insert("BKN", "BKN");
    m.insert("BRK", "BKN");

    m.insert("CHARLOTTE HORNETS", "CHA");
    m.insert("HORNETS", "CHA");
    m.insert("CHA", "CHA");
    m.insert("CHO", "CHA");

    m.insert("CHICAGO BULLS", "CHI");
    m.insert("BULLS", "CHI");
    m.insert("CHI", "CHI");

    m.insert("CLEVELAND CAVALIERS", "CLE");
    m.insert("CAVALIERS", "CLE");
    m.insert("CAVS", "CLE");
    m.insert("CLE", "CLE");

    m.insert("DETROIT PISTONS", "DET");
    m.insert("PISTONS", "DET");
    m.insert("DET", "DET");

    m.insert("INDIANA PACERS", "IND");
    m.insert("PACERS", "IND");
    m.insert("IND", "IND");

    m.insert("MIAMI HEAT", "MIA");
    m.insert("HEAT", "MIA");
    m.insert("MIA", "MIA");

    m.insert("MILWAUKEE BUCKS", "MIL");
    m.insert("BUCKS", "MIL");
    m.insert("MIL", "MIL");

    m.insert("NEW YORK KNICKS", "NYK");
    m.insert("KNICKS", "NYK");
    m.insert("NYK", "NYK");
    m.insert("NY", "NYK");

    m.insert("ORLANDO MAGIC", "ORL");
    m.insert("MAGIC", "ORL");
    m.insert("ORL", "ORL");

    m.insert("PHILADELPHIA 76ERS", "PHI");
    m.insert("76ERS", "PHI");
    m.insert("SIXERS", "PHI");
    m.insert("PHI", "PHI");

    m.insert("TORONTO RAPTORS", "TOR");
    m.insert("RAPTORS", "TOR");
    m.insert("TOR", "TOR");

    m.insert("WASHINGTON WIZARDS", "WAS");
    m.insert("WIZARDS", "WAS");
    m.insert("WAS", "WAS");
    m.insert("WSH", "WAS");

    m.insert("DALLAS MAVERICKS", "DAL");
    m.insert("MAVERICKS", "DAL");
    m.insert("MAVS", "DAL");
    m.insert("DAL", "DAL");

    m.insert("DENVER NUGGETS", "DEN");
    m.insert("NUGGETS", "DEN");
    m.insert("DEN", "DEN");

    m.insert("GOLDEN STATE WARRIORS", "GSW");
    m.insert("WARRIORS", "GSW");
    m.insert("GSW", "GSW");
    m.insert("GS", "GSW");

    m.insert("HOUSTON ROCKETS", "HOU");
    m.insert("ROCKETS", "HOU");
    m.insert("HOU", "HOU");

    m.insert("LOS ANGELES CLIPPERS", "LAC");
    m.insert("CLIPPERS", "LAC");
    m.insert("LAC", "LAC");
    m.insert("LA CLIPPERS", "LAC");

    m.insert("LOS ANGELES LAKERS", "LAL");
    m.insert("LAKERS", "LAL");
    m.insert("LAL", "LAL");
    m.insert("LA LAKERS", "LAL");

    m.insert("MEMPHIS GRIZZLIES", "MEM");
    m.insert("GRIZZLIES", "MEM");
    m.insert("MEM", "MEM");

    m.insert("MINNESOTA TIMBERWOLVES", "MIN");
    m.insert("TIMBERWOLVES", "MIN");
    m.insert("WOLVES", "MIN");
    m.insert("MIN", "MIN");

    m.insert("NEW ORLEANS PELICANS", "NOP");
    m.insert("PELICANS", "NOP");
    m.insert("NOP", "NOP");
    m.insert("NO", "NOP");

    m.insert("OKLAHOMA CITY THUNDER", "OKC");
    m.insert("THUNDER", "OKC");
    m.insert("OKC", "OKC");

    m.insert("PHOENIX SUNS", "PHX");
    m.insert("SUNS", "PHX");
    m.insert("PHX", "PHX");
    m.insert("PHO", "PHX");

    m.insert("PORTLAND TRAIL BLAZERS", "POR");
    m.insert("TRAIL BLAZERS", "POR");
    m.insert("BLAZERS", "POR");
    m.insert("POR", "POR");

    m.insert("SACRAMENTO KINGS", "SAC");
    m.insert("KINGS", "SAC");
    m.insert("SAC", "SAC");

    m.insert("SAN ANTONIO SPURS", "SAS");
    m.insert("SPURS", "SAS");
    m.insert("SAS", "SAS");

    m.insert("UTAH JAZZ", "UTA");
    m.insert("JAZZ", "UTA");
    m.insert("UTA", "UTA");

    m
});

pub struct MarketMatcher {
    time_tolerance_hours: i64,
}

impl MarketMatcher {
    pub fn new(time_tolerance_hours: i64) -> Self {
        Self {
            time_tolerance_hours,
        }
    }

    pub fn match_markets(
        &self,
        kalshi_markets: Vec<KalshiMarket>,
        poly_markets: Vec<PolymarketMarket>,
    ) -> Vec<MatchedMarket> {
        info!(
            kalshi_count = kalshi_markets.len(),
            poly_count = poly_markets.len(),
            "Starting market matching"
        );

        let mut matched = Vec::new();
        let mut used_poly_ids: HashSet<String> = HashSet::new();

        for k_market in &kalshi_markets {
            let k_title = self.normalize_title(&k_market.title);
            let k_category = k_market.category.to_uppercase();

            for p_market in &poly_markets {
                if used_poly_ids.contains(&p_market.condition_id) {
                    continue;
                }

                if k_category != p_market.category.to_uppercase() {
                    continue;
                }

                let p_question = self.normalize_title(&p_market.question);

                let similarity = self.calculate_similarity(&k_title, &p_question);

                if similarity >= 0.7 {
                    let confidence = rust_decimal::Decimal::from_f64_retain(similarity)
                        .unwrap_or(rust_decimal::Decimal::ZERO);

                    matched.push(MatchedMarket::new(
                        k_market.clone(),
                        p_market.clone(),
                        confidence,
                    ));
                    used_poly_ids.insert(p_market.condition_id.clone());

                    debug!(
                        kalshi_ticker = %k_market.ticker,
                        poly_condition = %p_market.condition_id,
                        similarity = similarity,
                        "Markets matched"
                    );
                    break;
                }
            }
        }

        info!(matched_count = matched.len(), "Market matching complete");

        matched
    }

    pub fn normalize_team_name(&self, name: &str) -> String {
        let name_upper = name.trim().to_uppercase();
        NBA_TEAM_MAPPINGS
            .get(name_upper.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                trace!(team_name = %name, "Unknown team name, returning uppercase");
                name_upper
            })
    }

    fn normalize_title(&self, title: &str) -> String {
        let mut normalized = title.to_uppercase();

        for (full, abbr) in NBA_TEAM_MAPPINGS.iter() {
            normalized = normalized.replace(full, abbr);
        }

        normalized
            .replace("VS", "-")
            .replace("VS.", "-")
            .replace(" @ ", "-")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        let a_words: HashSet<&str> = a.split_whitespace().collect();
        let b_words: HashSet<&str> = b.split_whitespace().collect();

        if a_words.is_empty() || b_words.is_empty() {
            return 0.0;
        }

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        intersection as f64 / union as f64
    }

    pub fn get_subscription_info(
        &self,
        matched_markets: &[MatchedMarket],
    ) -> (Vec<String>, Vec<String>) {
        let mut kalshi_tickers = Vec::new();
        let mut poly_token_ids = Vec::new();
        let mut seen_kalshi: HashSet<String> = HashSet::new();
        let mut seen_poly: HashSet<String> = HashSet::new();

        for mm in matched_markets {
            let k_id = mm.kalshi_ticker.clone();
            if !k_id.is_empty() && !seen_kalshi.contains(&k_id) {
                kalshi_tickers.push(k_id.clone());
                seen_kalshi.insert(k_id);
            }

            let p_id_yes = mm.polymarket_market.token_id_yes.clone();
            if !seen_poly.contains(&p_id_yes) {
                poly_token_ids.push(p_id_yes.clone());
                seen_poly.insert(p_id_yes);
            }

            let p_id_no = mm.polymarket_market.token_id_no.clone();
            if !seen_poly.contains(&p_id_no) {
                poly_token_ids.push(p_id_no.clone());
                seen_poly.insert(p_id_no);
            }
        }

        info!(
            kalshi_subscriptions = kalshi_tickers.len(),
            poly_subscriptions = poly_token_ids.len(),
            "Subscription info generated"
        );

        (kalshi_tickers, poly_token_ids)
    }
}

impl Default for MarketMatcher {
    fn default() -> Self {
        Self::new(24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_team_name() {
        let matcher = MarketMatcher::default();

        assert_eq!(matcher.normalize_team_name("Lakers"), "LAL");
        assert_eq!(matcher.normalize_team_name("Los Angeles Lakers"), "LAL");
        assert_eq!(matcher.normalize_team_name("LAL"), "LAL");
        assert_eq!(matcher.normalize_team_name("lakers"), "LAL");
        assert_eq!(matcher.normalize_team_name("Celtics"), "BOS");
        assert_eq!(matcher.normalize_team_name("Boston Celtics"), "BOS");
    }

    #[test]
    fn test_normalize_title() {
        let matcher = MarketMatcher::default();

        let normalized = matcher.normalize_title("Lakers vs Grizzlies");
        assert!(normalized.contains("LAL"));
        assert!(normalized.contains("MEM"));
    }

    #[test]
    fn test_calculate_similarity() {
        let matcher = MarketMatcher::default();

        let sim = matcher.calculate_similarity("LAL VS MEM", "LAL VS MEM");
        assert!((sim - 1.0).abs() < 0.01);

        let sim = matcher.calculate_similarity("LAL VS MEM", "BOS VS NYK");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_default() {
        let matcher = MarketMatcher::default();
        assert_eq!(matcher.time_tolerance_hours, 24);
    }
}
