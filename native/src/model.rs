//! Data structs shared between parse / store / sources layers.
//! Pure serde derives — no fetch / IO logic lives here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsRow {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub source: String,
    pub url: String,
    pub ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRow {
    pub id: String,
    pub code: String,
    pub name: String,
    pub market: String,
    pub price: f64,
    pub pct: f64,
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub time: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRow {
    pub id: String,
    pub cat: String,
    pub row: NewsRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: String,
    pub kind: String,
    pub payload: serde_json::Value,
}
