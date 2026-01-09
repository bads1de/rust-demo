use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

pub struct Okx {
    client: reqwest::Client,
}

impl Okx {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Okx {
    fn id(&self) -> &str {
        "okx"
    }

    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://www.okx.com/api/v5/public/instruments")
            .query(&[("instType", "SPOT")])
            .send()
            .await
            .context("Failed to get instruments from OKX")?;

        let json: Value = res.json().await.context("Failed to parse JSON")?;

        let symbols = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid format"))?
            .iter()
            .filter(|s| {
                let quote = s["quoteCcy"].as_str() == Some("USDT");
                let state = s["state"].as_str() == Some("live");
                quote && state
            })
            .map(|s| s["instId"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // OKX interval mapping if needed, but "1m" works for 'bar'
        let res = self.client
            .get("https://www.okx.com/api/v5/market/candles")
            .query(&[
                ("instId", symbol),
                ("bar", interval),
                ("limit", "300"), // OKX default max is 100, can go up to 300 for candles
            ])
            .send()
            .await
            .context("Failed to fetch candles from OKX")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse JSON")?;
        let rows = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: 'data' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // OKX: [ts, o, h, l, c, vol, volCcy, volCcyQuote, confirm]
            if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                klines.push(KlineData {
                    time: t.as_str().unwrap_or("0").parse().unwrap_or(0),
                    open: o.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    high: h.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    low: l.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    close: c.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    volume: v.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }

        // OKX returns newest first
        klines.reverse();

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_okx_fetch_candles() {
        let okx = Okx::new();
        // OKX uses hyphenated symbols e.g., "BTC-USDT"
        let result = okx.fetch_candles("BTC-USDT", "1m").await;
        
        assert!(result.is_ok());
        let candles = result.unwrap();
        assert!(!candles.is_empty());
    }

    #[tokio::test]
    async fn test_okx_get_symbols() {
        let okx = Okx::new();
        let result = okx.get_symbols().await;
        assert!(result.is_ok());
        // OKX returns internal format, but our get_symbols should normalize or return as is?
        // Usually UI expects "BTCUSDT", but OKX needs "BTC-USDT".
        // For simplicity in this app, we might return "BTC-USDT" and handle display in UI,
        // or convert. Let's assume we return raw ID for now.
        let symbols = result.unwrap();
        assert!(symbols.contains(&"BTC-USDT".to_string()));
    }
}
