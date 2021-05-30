use chrono::Utc;
use simple_stock_tracing_cli::{opts, stock};
fn main() {
    let opts = opts::parse();
    let start = opts.datetime();
    let end = Utc::now();

    for ticker in opts.tickers() {
        let quote_history = stock::get_quote_history(start, end, ticker);
        stock::print_quote_history(start, quote_history)
    }
}
