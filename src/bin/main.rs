use std::process;
use std::sync::Once;

use chrono::Utc;
use clap::{load_yaml, App};

use sstra::*;

static mut BENCHMARK: Vec<f64> = Vec::<f64>::new();
static INIT: Once = Once::new();
static MOV_AVG_NUM_DAYS: i32 = 30;

/// `price_diff` returns the percent difference in stock price.
///
/// The first value is relative, i.e. against a benchmark.
/// The second value is absolute, i.e. against itself.
fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    let days = series.len();
    let absolute = percent_diff(series[0], series[days - 1]).unwrap();

    // To calculate the relative, we need a benchmark.
    // Y! Finance uses "^GSPC" as the symbol for the S&P 500.
    // We only want to call this once per invocation.
    let benchmark: f64 = unsafe {
        INIT.call_once(|| {
            BENCHMARK = get_closing_prices("^GSPC", format!("{}d", days).as_str()).unwrap();
        });
        percent_diff(BENCHMARK[0], BENCHMARK[days - 1]).unwrap()
    };

    Some((absolute - benchmark, absolute))
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let now = Utc::now().format("%Y-%m-%d").to_string();

    let from: &str = matches.value_of("from").unwrap();
    let stocks: Vec<&str> = matches.values_of("stocks").unwrap().collect();

    if matches.is_present("debug") {
        eprintln!("Calculating the period from {} until {}...", from, now);
    }
    let period = count_days(&from, &now).unwrap_or_else(|err| {
        eprintln!("{}, please enter a date in the form YYYY-MM-DD.", err);
        process::exit(1);
    });

    // this is what we need to do to cast a string to an integer
    let days: i32 = period.split("d").collect::<Vec<&str>>()[0]
        .parse()
        .unwrap_or(0);

    if days < MOV_AVG_NUM_DAYS {
        eprintln!(
            "Please select a start date more than {} days in the past.",
            MOV_AVG_NUM_DAYS
        );
        process::exit(1);
    }

    if matches.is_present("debug") {
        eprintln!("Gathering info from the past {} for:", period);
    }
    if !matches.is_present("no-headers") {
        println!(
            "period start,symbol,price,change %,min,max,{}d avg",
            MOV_AVG_NUM_DAYS
        );
    }
    for stock in stocks {
        let symbol = stock.to_uppercase();
        let closing_prices = get_closing_prices(&symbol, &period).unwrap();
        let prices = price_diff(&closing_prices).unwrap();
        let price_difference: f64;
        if matches.is_present("relative") {
            price_difference = prices.0;
        } else {
            price_difference = prices.1;
        }
        println!(
            "{}",
            StockInfo::new(
                symbol,
                from.to_string(),
                closing_prices,
                price_difference,
                MOV_AVG_NUM_DAYS
            )
        )
    }
}
