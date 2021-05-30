pub mod opts {
    use chrono::prelude::*;
    use clap::Clap;
    #[derive(Clap, Debug)]
    #[clap(
        version = "1.0",
        author = "Alexei Pastuchov",
        about = "Milestone 1: a simple tracker"
    )]
    pub struct Opts {
        #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
        pub symbols: String,
        #[clap(short, long)]
        from: String,
    }

    pub fn parse() -> Opts {
        Opts::parse()
    }

    impl Opts {
        pub fn datetime(&self) -> DateTime<Utc> {
            Utc.datetime_from_str(self.from.as_str(), "%Y-%m-%d %H:%M:%S")
                .unwrap()
        }
        pub fn tickers(&self) -> std::str::Split<&str> {
            self.symbols.split(",")
        }
    }
}

pub mod stock {
    use chrono::{DateTime, Utc};
    use yahoo_finance_api::{Quote, YResponse, YahooConnector, YahooError};

    pub struct QuoteHistory<'a> {
        ticker: &'a str,
        max: Option<f64>,
        min: Option<f64>,
        last_price: Option<f64>,
        price_diff: Option<(f64, f64)>,
        sma: Option<Vec<f64>>,
    }

    fn request_quote_history(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        ticker: &str,
    ) -> Result<YResponse, YahooError> {
        YahooConnector::new().get_quote_history(ticker, start, end)
    }

    fn sorted_quotes(quotes: Vec<Quote>) -> Vec<f64> {
        let mut sorted_quotes: Vec<Quote> = quotes;
        sorted_quotes.sort_by_cached_key(|k| k.timestamp);
        sorted_quotes
            .iter()
            .map(|q| q.adjclose as f64)
            .collect::<Vec<f64>>()
    }

    fn calculate_quote_history<'a>(
        ticker: &'a str,
        closes: Vec<f64>,
    ) -> QuoteHistory<'a> {
        QuoteHistory {
            ticker: ticker,
            max: max(&closes),
            min: min(&closes),
            last_price: Some(*closes.last().unwrap_or(&0.0)),
            price_diff: price_diff(&closes),
            sma: n_window_sma(30, &closes),
        }
    }

    ///
    /// Calculates the absolute and relative (price) change between the beginning and ending of an f64 closes. The relative (price) change is relative to the beginning.
    ///
    /// # Returns
    ///
    /// A tuple `(absolute, relative)` difference.
    ///
    fn price_diff(closes: &[f64]) -> Option<(f64, f64)> {
        if !closes.is_empty() {
            // unwrap is safe here even if first == last
            let (first, last) =
                (closes.first().unwrap(), closes.last().unwrap());
            let abs_diff = last - first;
            let first = if *first == 0.0 { 1.0 } else { *first };
            let rel_diff = abs_diff / first;
            Some((abs_diff, rel_diff))
        } else {
            None
        }
    }

    ///
    /// Window function to create a simple moving average
    ///
    fn n_window_sma(n: usize, closes: &[f64]) -> Option<Vec<f64>> {
        if !closes.is_empty() && n > 1 {
            Some(
                closes
                    .windows(n)
                    .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                    .collect(),
            )
        } else {
            None
        }
    }

    ///
    /// Find the maximum in a closes of f64
    ///
    fn max(closes: &[f64]) -> Option<f64> {
        if closes.is_empty() {
            None
        } else {
            Some(closes.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }

    ///
    /// Find the minimum in a closes of f64
    ///
    fn min(closes: &[f64]) -> Option<f64> {
        if closes.is_empty() {
            None
        } else {
            Some(closes.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }

    pub fn get_quote_history(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        ticker: &str,
    ) -> Result<QuoteHistory, YahooError> {
        match request_quote_history(start, end, ticker) {
            Ok(quotes) => match quotes.quotes() {
                Ok(quotes) => {
                    Ok(calculate_quote_history(ticker, sorted_quotes(quotes)))
                }
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub fn print_quote_history(
        start: DateTime<Utc>,
        qhr: Result<QuoteHistory, YahooError>,
    ) {
        match qhr {
            Ok(qh) => {
                println!(
                    "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                    start.to_rfc3339(),
                    qh.ticker,
                    qh.last_price.unwrap_or(0.0),
                    qh.price_diff.unwrap_or((0.0, 0.0)).1 * 100.0,
                    qh.min.unwrap_or(0.0),
                    qh.max.unwrap_or(0.0),
                    qh.sma.unwrap_or([0.0].to_vec()).last().unwrap_or(&0.0)
                )
            }
            Err(err) => eprint!("no quotes found for the symbol {}", err),
        }
    }
}
