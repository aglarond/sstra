# Simple Stock Tracking Rust App (sstra)

sstra is a command-line tool that can get basic information about
various stocks using the Yahoo! Finance API. The information is then
displayed in a CSV format for easy parsing and piping to other
applications.

The program was written as part of Manning's LiveProject
[Building a Stock-Tracking CLI With Async Streams in Rust
](https://liveproject.manning.com).

## Usage

See the `--help` text for descriptions of each option.

Call the program with a `--from` date and a list of `--symbols` and it
will collect and display information about closing price movement of
those stocks over the time period beginning at that date and ending
today.

Example for retrieving the stock price percentage change since June 1,
2020 for Microsoft, Google, Apple, Uber, and IBM:

```
$ cargo run --release -- --from "2020-06-01" --symbols=MSFT,GOOG,AAPL,UBER,IBM
period start,symbol,price,change %,min,max,30d avg
2020-06-01,MSFT,$219.42,11.63%,$134.37,$231.05,$214.85
2020-06-01,GOOG,$1747.90,9.38%,$1056.62,$1827.99,$1774.68
2020-06-01,AAPL,$128.70,53.99%,$55.74,$133.95,$120.50
2020-06-01,UBER,$50.63,34.58%,$14.82,$54.86,$50.03
2020-06-01,IBM,$125.55,-22.86%,$90.99,$132.08,$121.22
```

For extra output as the program executes, use the `--debug` flag to
write these logs to stderr.

## Building

Thanks to the Rust ecosystem's excellent tooling, building `sstra` is
trivial:

For normal development and use:
```
cargo build
```

For a release version:
```
cargo build --release
```

## LICENSE

MIT
