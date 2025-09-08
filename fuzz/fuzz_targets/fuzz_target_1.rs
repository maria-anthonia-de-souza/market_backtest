#![no_main]

use libfuzzer_sys::fuzz_target;
use market_backtest::data;
//turns raw bytes slice into something that implements the reader trait so we can use csv::Reader
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let reader = Cursor::new(data);
    // Call your CSV loader
    let _ = data::load_csv_from_reader(reader);

});
