use std::f64;
use std::fmt;
use std::process;

use chrono::format::ParseError;
use chrono::NaiveDate;
use yahoo_finance_api as yahoo;

pub struct StockInfo {
    pub symbol: String,
    pub period_start: String,
    pub closing_price: f64,
    pub price_difference: f64,
    pub min: f64,
    pub max: f64,
    pub simple_moving_average: f64,
}

impl StockInfo {
    /// Create a new StockInfo.
    ///
    /// Work is done with the parameters passed into `new' so that the
    /// struct holds only the information relevant for display.
    pub fn new(
        symbol: String,
        period_start: String,
        closing_prices: Vec<f64>,
        price_difference: f64,
        mov_avg_num_days: i32,
    ) -> StockInfo {
        StockInfo {
            symbol: symbol,
            period_start: period_start,
            price_difference: price_difference,
            min: min(&closing_prices).unwrap(),
            max: max(&closing_prices).unwrap(),
            simple_moving_average: *n_window_sma(mov_avg_num_days as usize, &closing_prices)
                .unwrap()
                .last()
                .unwrap(),
            closing_price: *closing_prices.last().unwrap(),
        }
    }
}

impl fmt::Display for StockInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            self.period_start,
            self.symbol,
            self.closing_price,
            self.price_difference,
            self.min,
            self.max,
            self.simple_moving_average
        )
    }
}

pub fn get_closing_prices(symbol: &str, period: &str) -> Option<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();
    let response = provider
        .get_quote_range(symbol, "1d", period)
        .unwrap_or_else(|err| {
            eprintln!(
                "Encountered a problem calling the Yahoo! Finance API: {:?}",
                err
            );
            process::exit(1);
        });
    let quotes = response.quotes().unwrap();
    let closing_prices: Vec<f64> = quotes.iter().map(|quote| quote.adjclose).collect();

    Some(closing_prices)
}

pub fn count_days(from: &str, until: &str) -> Result<String, ParseError> {
    let past = NaiveDate::parse_from_str(&from, "%Y-%m-%d")?;
    let present = NaiveDate::parse_from_str(&until, "%Y-%m-%d")?;
    let period = format!("{}", NaiveDate::signed_duration_since(present, past));
    Ok(period.split("P").collect::<Vec<&str>>()[1]
        .replace("D", "d")
        .to_string())
}

pub fn min(series: &[f64]) -> Option<f64> {
    Some(series.iter().fold(f64::INFINITY, |a, &b| a.min(b)))
}

pub fn max(series: &[f64]) -> Option<f64> {
    Some(series.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)))
}

/// calculate the simple moving average of a series over a time period, n
pub fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    let mut averages = Vec::<f64>::new();
    for subset in series.windows(n) {
        let length: f64 = subset.len() as f64;
        let avg = subset.iter().sum::<f64>() / length;
        averages.push(avg);
    }
    Some(averages)
}

pub fn percent_diff(first: f64, second: f64) -> Option<f64> {
    let diff = second - first;
    Some((diff * 100.0) / first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_sma_over_3() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert_eq!(
            [2.0, 3.0, 4.0, 5.0, 6.0].to_vec(),
            n_window_sma(3, &x).unwrap()
        );
    }

    #[test]
    fn calculates_percent_difference() {
        assert_eq!(50.0, percent_diff(100.0, 150.0).unwrap())
    }

    #[test]
    fn calculates_period() {
        let begin_date = String::from("2020-11-01");
        let end_date = String::from("2020-12-01");
        assert_eq!(
            String::from("30d"),
            count_days(&begin_date, &end_date).unwrap()
        );
    }

    #[test]
    fn gets_min() {
        let x = [1.0, 2.0, 3.0, f64::NAN];
        assert_eq!(1.0, min(&x).unwrap());
    }

    #[test]
    fn gets_max() {
        let x = [1.0, 2.0, 3.0, f64::NAN];
        assert_eq!(3.0, max(&x).unwrap());
    }
}
