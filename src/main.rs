use serde::Serialize;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::Result;
use yahoo_finance_api::{Quote, YahooConnector};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Time between pulls (seconds)
    #[clap(short, long, value_parser, default_value_t = 300)]
    delay: u64,

    /// List of stocks to track
    #[clap(
        short,
        long,
        value_parser,
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    stocks: Vec<String>,

    /// Output CSV file
    #[clap(short, long, value_parser, default_value = "out.csv")]
    csv: String,
}

#[derive(Serialize, Debug)]
struct CSVQuote {
    name: String,
    timestamp: u64,
    open: f64,
    close: f64,
    adjclose: f64,
    high: f64,
    low: f64,
    volume: u64,
}

impl CSVQuote {
    pub fn new(name: &str, quote: Quote) -> Self {
        Self {
            name: name.to_string(),
            timestamp: quote.timestamp,
            open: quote.open,
            close: quote.close,
            adjclose: quote.adjclose,
            high: quote.high,
            low: quote.low,
            volume: quote.volume,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let provider = YahooConnector::new();
    loop {
        let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
        for stock in &args.stocks {
            let quotes = provider.get_latest_quotes(stock, "1d").await;
            match quotes {
                Ok(q) => {
                    let last_quote = q.last_quote()?;
                    println!(
                        "received data for stock \"{}\" at timestamp {}",
                        stock, last_quote.timestamp
                    );
                    writer.serialize(CSVQuote::new(stock, last_quote))?;
                }
                Err(e) => eprintln!("fetching data for stock \"{}\" failed: {:?}", stock, e),
            }
        }
        tokio::fs::write(&args.csv, String::from_utf8(writer.into_inner()?)?).await?;

        match args.delay {
            0 => return Ok(()),
            _ => tokio::time::sleep(Duration::from_secs(args.delay)).await,
        }
    }
}
