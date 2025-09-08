use chrono::NaiveDate;
use serde::Deserialize;
use std::error::Error;
use std::io::Read;

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

    const FORMAT: &str = "%Y-%m-%d"; // matches your CSV date format

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub fn load_csv_from_reader<R: Read>(reader: R) -> Result<Vec<Candle>, csv::Error> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut candles = Vec::new();
    for result in rdr.deserialize() {
        let record: Candle = result?;
        candles.push(record);
    }
    Ok(candles)
}

pub fn load_csv(path: &str) -> Result<Vec<Candle>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?; //opens file under the hood and builds csv reader  
    let mut data = Vec::new(); //vec to collect parsed candle rows 
    // rep the price movement of a financial instrument (ex. stock, crypto)
    //iterate over deserialized recor mapping each row into candle type (date/time of interval, highest price, lowest price, end price, vol of units trated in interval)
    for result in rdr.deserialize() {
        let record: Candle = result?;
        data.push(record);
    }

    Ok(data)
}
