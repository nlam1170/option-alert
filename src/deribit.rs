use rayon::prelude::*;
use serde_json::Value;
use chrono::Datelike;
use std::{collections::HashMap, error::Error};
                                                                       
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
        (11, "NOV"),
        (12, "DEC"),
    ];

    let months_map: HashMap<u32, &str> = months.par_iter().copied().collect();
    months_map[num]
}

fn get_next_month(curr_month: &u32) -> u32 {
    if *curr_month == 12 { 1 }
    else { curr_month + 1 }
}

fn get_instrument_names() -> Result<Vec<String>, Box<dyn Error>> {
    let curr_month = chrono::Local::now().month();
    let next_month = get_next_month(&curr_month);
    let url = "https://www.deribit.com/api/v2/public/get_instruments?currency=BTC&expired=false&kind=option";
    let resp: Value = ureq::get(url).call()?.into_json()?;
    let result = &resp["result"].as_array().unwrap();

    let instruments = result.par_iter()
        .map(|item| {
            let name = item["instrument_name"].as_str().unwrap();
            name.to_string()
        })
        .filter(|name| name.contains(get_month(&curr_month)) || name.contains(get_month(&next_month)))
        .collect::<Vec<_>>();
    Ok(instruments)
}

fn get_data(name: &str) -> Result<Data, Box<dyn Error>> {
    let url = format!("https://www.deribit.com/api/v2/public/get_book_summary_by_instrument?instrument_name={}", name);
    let resp: Value = ureq::get(&url).call()?.into_json()?;
    let oi = resp["result"][0]["open_interest"].as_f64().unwrap();
    let volume = resp["result"][0]["volume"].as_f64().unwrap();
    Ok(Data{ oi, volume })
}

pub fn check_alert_event(old: &Data, new: &Data) -> bool {
    if new.volume - old.volume >= 250.0 {
        true
    }
    else { false }
}


#[derive(Debug, Clone, Copy)]
pub struct Data {
    pub oi: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct Instruments(pub HashMap<String, Data>);

impl Instruments {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let names = get_instrument_names()?;
        let map = names.par_iter()
            .map(|name| {
                let data = get_data(name).unwrap();
                (name.clone(), data)
            })
            .collect::<HashMap<_,_>>();
        
        Ok( Self(map) )
    }
}