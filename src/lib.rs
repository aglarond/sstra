use std::f64;
use std::fmt;
use std::process;

use actix::prelude::*;
use async_trait::async_trait;
use chrono::format::ParseError;
use chrono::NaiveDate;
use yahoo_finance_api as yahoo;

pub struct StockPriceFetcher;
pub struct StockPriceProcessor;

pub struct StockQuery<'a> {
    pub symbol: String,
    pub period_start: String,
    pub mov_avg_num_days: &'a i32,
}

#[async_trait(?Send)]
impl<'a> Message for StockQuery<'a> {
    type Result = Result<StockPrices<'static>, std::io::Error>;
}

#[derive(Message)]
#[rtype(result = "Result<StockInfo, std::io::Error>")]
pub struct StockPrices<'a> {
    pub symbol: &'a String,
    pub period_start: &'a String,
    pub closing_prices: &'a Vec<f64>,
    pub mov_avg_num_days: &'a i32,
}

#[derive(Message)]
#[rtype(result = "Result<Self, std::io::Error>")]
pub struct StockInfo {
    pub symbol: String,
    pub period_start: String,
    pub closing_price: f64,
    pub price_difference: f64,
    pub min: f64,
    pub max: f64,
    pub simple_moving_average: f64,
}

impl Actor for StockPriceFetcher {
    type Context = Context<Self>;
}

impl Actor for StockPriceProcessor {
    type Context = Context<Self>;
}

#[async_trait(?Send)]
impl<'a> Handler<StockQuery<'a>> for StockPriceFetcher {
    type Result = Result<StockPrices<'static>, std::io::Error>;

    async fn handle(&mut self, msg: StockQuery<'static>, _ctx: &mut Self::Context) -> Self::Result {
        let prices = get_closing_prices(&msg.symbol, &msg.period_start)
            .await
            .unwrap();
        Ok(StockPrices {
            symbol: &msg.symbol.to_string(),
            period_start: &msg.period_start.to_string(),
            closing_prices: &prices,
            mov_avg_num_days: &msg.mov_avg_num_days,
        })
    }
}

#[async_trait(?Send)]
impl<'a> Handler<StockPrices<'a>> for StockPriceProcessor {
    type Result = Result<StockInfo, std::io::Error>;

    async fn handle(&mut self, msg: StockPrices<'a>, _ctx: &mut Self::Context) -> Self::Result {
        let prices = price_diff(&msg.closing_prices).await.unwrap();
        let price_difference: f64 = prices.0;
        let min = min(&msg.closing_prices).await.unwrap();
        let max = max(&msg.closing_prices).await.unwrap();
        let sma = *n_window_sma(*msg.mov_avg_num_days as usize, &msg.closing_prices)
            .await
            .unwrap()
            .last()
            .unwrap();
        Ok(StockInfo {
            symbol: msg.symbol.to_string(),
            period_start: msg.period_start.to_string(),
            closing_price: *msg.closing_prices.last().unwrap(),
            price_difference: price_difference,
            min: min,
            max: max,
            simple_moving_average: sma,
        })
    }
}

impl actix::Supervised for StockPriceFetcher {
    fn restarting(&mut self, _ctx: &mut Context<StockPriceFetcher>) {
        println!("restarting");
    }
}

impl actix::Supervised for StockPriceProcessor {
    fn restarting(&mut self, _ctx: &mut Context<StockPriceProcessor>) {
        println!("restarting");
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

async fn get_closing_prices(symbol: &str, period: &str) -> Option<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();
    let response = provider
        .get_quote_range(symbol, "1d", period)
        .await
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

pub async fn min(series: &[f64]) -> Option<f64> {
    Some(series.iter().fold(f64::INFINITY, |a, &b| a.min(b)))
}

pub async fn max(series: &[f64]) -> Option<f64> {
    Some(series.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)))
}

/// calculate the simple moving average of a series over a time period, n
pub async fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
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

/// `price_diff` returns the percent difference in stock price.
///
/// Returns a tuple of (percentage, absolute difference).
/// The second value is absolute, i.e. against itself.
pub async fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    let days = series.len();
    let percentage = percent_diff(series[0], series[days - 1]).unwrap();
    let absolute = series[days - 1] - series[0];

    Some((percentage, absolute))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_sma_over_3() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert_eq!(
            [2.0, 3.0, 4.0, 5.0, 6.0].to_vec(),
            tokio_test::block_on(n_window_sma(3, &x)).unwrap()
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
    fn calculates_price_difference() {
        let x = [1.0, 2.0, 3.0];
        assert_eq!((200.0, 2.0), tokio_test::block_on(price_diff(&x)).unwrap());
    }

    #[test]
    fn gets_min() {
        let x = [1.0, 2.0, 3.0, f64::NAN];
        assert_eq!(1.0, tokio_test::block_on(min(&x)).unwrap());
    }

    #[test]
    fn gets_max() {
        let x = [1.0, 2.0, 3.0, f64::NAN];
        assert_eq!(3.0, tokio_test::block_on(max(&x)).unwrap());
    }
}
