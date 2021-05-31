use std::process;

use actix::Actor;
use chrono::Utc;
use clap::{load_yaml, App};

use sstra::*;

static MOV_AVG_NUM_DAYS: i32 = 30;

#[actix_rt::main]
async fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();
    let now = Utc::now().format("%Y-%m-%d").to_string();

    let from_in: &str = matches.value_of("from").unwrap();
    let from_split: Vec<&str> = from_in.split('T').collect();
    let from = from_split[0];
    let symbols: Vec<&str> = matches.values_of("symbols").unwrap().collect();

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

    let addr = StockPriceFetcher.start();
    for stock in symbols {
        let symbol = stock.to_uppercase();
        let closing_prices = get_closing_prices(&symbol, &period).await.unwrap();
        let prices = price_diff(&closing_prices).await.unwrap();
        let price_difference: f64 = prices.0;
        let result = addr
            .send(
                StockInfo::new(
                    symbol,
                    from.to_string(),
                    closing_prices.to_vec(),
                    price_difference,
                    MOV_AVG_NUM_DAYS,
                )
                .await,
            )
            .await;
        match result {
            Ok(res) => println!("{}", res.unwrap()),
            Err(err) => eprintln!("{}", err),
        }
    }
}
