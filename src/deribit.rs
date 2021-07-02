use rayon::prelude::*;
use serde_json::Value;
use chrono::Datelike;
use std::{collections::HashMap, error::Error};

#[derive(Clone, Debug)]
pub struct Instrument {
    pub name: String,
    pub oi: f64,
    pub volume: f64,
}

fn get_month(num: &u32) -> &str {
    let months = [
        (1, "JAN"),
        (2, "FEB"),
        (3, "MAR"),
        (4, "APR"),
        (5, "MAY"),
        (6, "JUN"),
        (7, "JUL"),
        (8, "AUG"),
        (9, "SEP"),
        (10, "OCT"),
        (11, "OCT"),
        (12, "DEC"),
    ];

    let months_map: HashMap<u32, &str> = months.iter().cloned().collect();
    months_map[num]
}

fn get_instruments_for_month() -> Result<Vec<String>, Box<dyn Error>> {
    let curr_month = chrono::Local::now().month();
    let url = "https://www.deribit.com/api/v2/public/get_instruments?currency=BTC&expired=false&kind=option";
    let resp: Value = ureq::get(url).call()?.into_json()?;
    let result = &resp["result"].as_array().unwrap();

    let instruments = result.par_iter()
        .filter(|item| {
            let period = item["settlement_period"].as_str().unwrap();
            period == "day" || period == "week"
        })
        .map(|item| {
            let name = item["instrument_name"].as_str().unwrap();
            name.to_string()
        })
        .filter(|name| name.contains(get_month(&curr_month)))
        .collect::<Vec<_>>();
    Ok(instruments)
}

fn get_data(url: &str) -> Result<(f64, f64), Box<dyn Error>> {
    let resp: Value = ureq::get(url).call()?.into_json()?;
    let volume = resp["result"][0]["volume"].as_f64().unwrap();
    let oi = resp["result"][0]["open_interest"].as_f64().unwrap();
    Ok((oi, volume))
}

pub fn get_instrument_info() -> Result<Vec<Instrument>, Box<dyn Error>> {
    let instruments = get_instruments_for_month()?;
    let info = instruments.par_iter()
        .map(|name| {
            let url = format!("https://www.deribit.com/api/v2/public/get_book_summary_by_instrument?instrument_name={}", name);
            let (oi, volume) = get_data(&url).unwrap();
            Instrument { name: name.to_owned(), oi, volume }
        }).collect::<Vec<_>>();
    Ok(info)
}

pub fn alert_event(old: &Instrument, new: &Instrument) -> bool {
    if new.volume - old.volume >= 250.0 {
        true
    }
    else {
        false
    }
}