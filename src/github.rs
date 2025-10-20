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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchingError {
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Serde(#[from] serde_json::Error),
}

fn fetch_contributions(username: &str) -> Result<ApiData, FetchingError> {
    let url = format!(
        "https://github-contributions-api.jogruber.de/v4/{}",
        username
    );
    let body = get(url)?.text()?;

    let parsed: ApiData = serde_json::from_str(&body)?;

    Ok(parsed)
}

pub fn get_contributions(username: &str) -> Result<Vec<u8>, FetchingError> {
    let api_data = fetch_contributions(username)?;

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
    Ok(recent_counts.into_iter().map(|(_, level)| level).collect())
}

#[test]
pub fn fetch_github_contribution_data() {
    fetch_contributions("chamaloriz");
}
