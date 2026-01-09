use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

pub struct Bitget {
    client: reqwest::Client,
}

impl Bitget {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Bitget {
    fn id(&self) -> &str {
        "bitget"
    }

    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.bitget.com/api/v2/spot/public/symbols")
            .send()
            .await
            .context("Failed to get symbols from Bitget")?;

        let json: Value = res.json().await.context("Failed to parse JSON")?;
        
        // Bitget V2: { code: "00000", msg: "...", data: [ ... ] }
        let symbols = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid format"))?
            .iter()
            .filter(|s| {
                let quote = s["quoteCoin"].as_str() == Some("USDT");
                let status = s["status"].as_str() == Some("online");
                quote && status
            })
            .map(|s| s["symbol"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // Bitget V2 uses "1min" instead of "1m"
        let granularity = match interval {
            "1m" => "1min",
            "5m" => "5min",
            "15m" => "15min",
            "30m" => "30min",
            "1h" => "1h",
            "4h" => "4h",
            "1d" => "1d",
            _ => interval,
        };

        let res = self.client
            .get("https://api.bitget.com/api/v2/spot/market/candles")
            .query(&[
                ("symbol", symbol),
                ("granularity", granularity),
                ("limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to fetch candles from Bitget")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse JSON")?;
        let rows = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: 'data' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        // Bitget returns [ts, o, h, l, c, v, ...]
        for row in rows {
            if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                // Bitget timestamps are strings in milliseconds
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

        // Bitget usually returns newest first, so reverse to old -> new
        klines.reverse();

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bitget_fetch_candles() {
        let bitget = Bitget::new();
        // Bitget V2 API: granularity "1min"
        let result = bitget.fetch_candles("BTCUSDT", "1min").await;
        
        assert!(result.is_ok());
        let candles = result.unwrap();
        assert!(!candles.is_empty());
        
        let first = &candles[0];
        assert!(first.close > 0.0);
    }

    #[tokio::test]
    async fn test_bitget_get_symbols() {
        let bitget = Bitget::new();
        let result = bitget.get_symbols().await;
        assert!(result.is_ok());
        let symbols = result.unwrap();
        assert!(symbols.contains(&"BTCUSDT".to_string()));
    }
}
