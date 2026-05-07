// API Source: https://docs.gitea.com/api/
// GET {server_url}/api/v1/users/{username}/heatmap
// Returns: [{"timestamp": <unix_seconds>, "contributions": <int>}, ...]
// Multiple entries per day are possible; aggregate by UTC date.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct HeatmapEntry {
    timestamp: i64,
    contributions: u32,
}

#[derive(Error, Debug)]
pub enum FetchingError {
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid timestamp in response: {0}")]
    Timestamp(i64),
}

fn fetch_heatmap(
    server_url: &str,
    username: &str,
    token: Option<&str>,
) -> Result<Vec<HeatmapEntry>, FetchingError> {
    let url = format!(
        "{}/api/v1/users/{}/heatmap",
        server_url.trim_end_matches('/'),
        username
    );
    let mut req = Client::new().get(url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("token {}", t));
    }
    let body = req.send()?.text()?;
    let parsed: Vec<HeatmapEntry> = serde_json::from_str(&body)?;
    Ok(parsed)
}

pub fn get_contributions(
    server_url: &str,
    username: &str,
    token: Option<&str>,
) -> Result<Vec<u8>, FetchingError> {
    let entries = fetch_heatmap(server_url, username, token)?;

    let mut by_date: HashMap<NaiveDate, u32> = HashMap::new();
    for e in entries {
        let dt = DateTime::<Utc>::from_timestamp(e.timestamp, 0)
            .ok_or(FetchingError::Timestamp(e.timestamp))?;
        *by_date.entry(dt.date_naive()).or_insert(0) += e.contributions;
    }

    let today = Utc::now().date_naive();
    let counts: Vec<u32> = (0..7)
        .map(|i| {
            by_date
                .get(&(today - Duration::days(i)))
                .copied()
                .unwrap_or(0)
        })
        .collect();

    let max = *counts.iter().max().unwrap_or(&0);
    let levels: Vec<u8> = counts
        .iter()
        .map(|&c| {
            if c == 0 || max == 0 {
                0
            } else {
                (c as u64 * 5).div_ceil(max as u64).clamp(1, 5) as u8
            }
        })
        .collect();

    Ok(levels)
}

#[test]
pub fn fetch_gitea_contribution_data() {
    let _ = fetch_heatmap("https://gitea.selfhosted.be", "chamaloriz", None);
}
