use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

//
// --------------------
// Candle Struct & Loader
// --------------------
#[derive(Debug, Deserialize)]
pub struct Candle {
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// Custom date format module for Serde
mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMATS: &[&str] = &["%Y-%m-%d", "%m/%d/%Y"];

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        for fmt in FORMATS {
            if let Ok(date) = NaiveDate::parse_from_str(&s, fmt) {
                return Ok(date);
            }
        }
        Err(serde::de::Error::custom(format!(
            "Invalid date format: {}",
            s
        )))
    }
}

/// Load candles from any reader
pub fn load_csv_from_reader<R: Read>(reader: R) -> Result<Vec<Candle>, csv::Error> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut candles = Vec::new();
    for result in rdr.deserialize() {
        let record: Candle = result?;
        candles.push(record);
    }
    Ok(candles)
}

/// Load candles from file path
pub fn load_csv(path: &Path) -> Result<Vec<Candle>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in rdr.deserialize() {
        let record: Candle = result?;
        data.push(record);
    }
    Ok(data)
}

#[derive(Debug, Deserialize)]
pub struct MaturityValue(#[serde(deserialize_with = "csv::invalid_option")] Option<f64>);

//
// --------------------
// Risk-free Rate Struct & Loader
// --------------------
#[derive(Debug, Deserialize)]
pub struct RiskFreeRateRow {
    #[serde(with = "date_format")]
    pub date: NaiveDate,

    // Flatten other columns dynamically and parse as Option<f64> directly
    #[serde(flatten)]
    pub maturities: HashMap<String, MaturityValue>,
}

/// Load a daily series of risk-free returns from a Treasury CSV.
/// `maturity` is the column to use (e.g., "1 Mo").
pub fn load_risk_free_series<P: AsRef<Path>>(
    path: P,
    maturity: &str,
) -> Result<Vec<f64>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);
    let mut rf_returns = Vec::new();

    for result in rdr.deserialize::<RiskFreeRateRow>() {
        let row = result?;

        if let Some(rate) = row.maturities.get(maturity).and_then(|rate| rate.0) {
            let annual = rate / 100.0; // convert % to decimal
            let daily = (1.0 + annual).powf(1.0 / 252.0) - 1.0;
            rf_returns.push(daily);
        }
    }

    Ok(rf_returns)
}

// use chrono::NaiveDate;
// use serde::Deserialize;
// use std::error::Error;
// use std::io::Read;
// use std::path::Path;

// #[derive(Debug, Deserialize)]
// pub struct Candle {
//     #[serde(with = "date_format")]
//     pub date: NaiveDate,
//     pub open: f64,
//     pub high: f64,
//     pub low: f64,
//     pub close: f64,
//     pub volume: f64,
// }

// // Custom date format module for Serde
// mod date_format {
//     use chrono::NaiveDate;
//     use serde::{self, Deserialize, Deserializer};

//     const FORMAT: &str = "%Y-%m-%d"; // matches your CSV date format

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
//     }
// }

// pub fn load_csv_from_reader<R: Read>(reader: R) -> Result<Vec<Candle>, csv::Error> {
//     let mut rdr = csv::Reader::from_reader(reader);
//     let mut candles = Vec::new();
//     for result in rdr.deserialize() {
//         let record: Candle = result?;
//         candles.push(record);
//     }
//     Ok(candles)
// }

// pub fn load_csv(path: &Path) -> Result<Vec<Candle>, Box<dyn Error>> {
//     let mut rdr = csv::Reader::from_path(path)?; //opens file under the hood and builds csv reader
//     let mut data = Vec::new(); //vec to collect parsed candle rows
//     // rep the price movement of a financial instrument (ex. stock, crypto)
//     //iterate over deserialized recor mapping each row into candle type (date/time of interval, highest price, lowest price, end price, vol of units trated in interval)
//     for result in rdr.deserialize() {
//         let record: Candle = result?;
//         data.push(record);
//     }

//     Ok(data)
// }

// use chrono::NaiveDate;
// use serde::Deserialize;
// use std::error::Error;
// use std::io::Read;
// use std::path::Path;

// #[derive(Debug, Deserialize)]
// pub struct Candle {
//     #[serde(with = "date_format")]
//     pub date: NaiveDate,
//     pub open: f64,
//     pub high: f64,
//     pub low: f64,
//     pub close: f64,
//     pub volume: f64,
// }

// // Custom date format module for Serde
// mod date_format {
//     use chrono::NaiveDate;
//     use serde::{self, Deserialize, Deserializer};

//     const FORMAT: &str = "%Y-%m-%d";

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
//     }
// }

// pub fn load_csv_from_reader<R: Read>(reader: R) -> Result<Vec<Candle>, csv::Error> {
//     let mut rdr = csv::Reader::from_reader(reader);
//     let mut candles = Vec::new();
//     for result in rdr.deserialize() {
//         let record: Candle = result?;
//         candles.push(record);
//     }
//     Ok(candles)
// }

// pub fn load_csv(path: &Path) -> Result<Vec<Candle>, Box<dyn Error>> {
//     let mut rdr = csv::Reader::from_path(path)?;
//     let mut data = Vec::new();
//     for result in rdr.deserialize() {
//         let record: Candle = result?;
//         data.push(record);
//     }
//     Ok(data)
// }

// // ----------------------
// // Risk-free rate loader
// // ----------------------
// #[derive(Debug, Deserialize)]
// pub struct RiskFreeRateRow {
//     pub date: String,
//     #[serde(rename = "1 Mo")]
//     pub one_month: f64, // % format
// }

// // Read first row of CSV and return risk-free rate as decimal
// pub fn load_risk_free_rate(path: &Path) -> Result<f64, Box<dyn Error>> {
//     let mut rdr = csv::Reader::from_path(path)?;
//     let mut iter = rdr.deserialize::<RiskFreeRateRow>();

//     if let Some(result) = iter.next() {
//         let row = result?;
//         Ok(row.one_month / 100.0) // convert percentage to decimal
//     } else {
//         Err("Risk-free rate CSV is empty".into())
//     }
// }
