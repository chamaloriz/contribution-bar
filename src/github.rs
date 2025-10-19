// API Source : https://github.com/grubersjoe/github-contributions-api
// Results are cached hourly

use chrono::{Duration, NaiveDate, Utc};
use reqwest::blocking::get;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Contribution {
    date: NaiveDate,
    level: u8,
}

#[derive(Debug, Deserialize)]
struct ApiData {
    contributions: Vec<Contribution>,
}

fn fetch_contributions(username: &str) -> ApiData {
    let url = format!(
        "https://github-contributions-api.jogruber.de/v4/{}",
        username
    );
    let body = get(url)
        .expect("issue while doing the request")
        .text()
        .expect("issue while getting the text");

    let parsed: ApiData = serde_json::from_str(&body).expect("Failed to parse JSON");

    parsed
}

pub fn get_contributions(username: &str) -> Vec<u8> {
    let api_data = fetch_contributions(username);
    let today = Utc::now().naive_utc().date();
    let end_date = today - Duration::days(6);

    let mut recent_counts: Vec<(NaiveDate, u8)> = api_data
        .contributions
        .iter()
        .filter(|c| c.date >= end_date && c.date <= today)
        .map(|c| (c.date, c.level))
        .collect();

    recent_counts.sort_by_key(|(date, _)| *date);
    recent_counts.reverse();
    recent_counts.into_iter().map(|(_, level)| level).collect()
}
