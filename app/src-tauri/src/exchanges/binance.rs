use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

pub struct Binance {
    client: reqwest::Client,
}

impl Binance {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Binance {
    fn id(&self) -> &str {
        "binance"
    }

    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.binance.com/api/v3/exchangeInfo")
            .send()
            .await
            .context("Failed to get exchange info")?;

        let json: Value = res.json().await.context("Failed to parse JSON")?;
        
        let symbols = json["symbols"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid format"))?
            .iter()
            .filter(|s| {
                let is_usdt = s["quoteAsset"].as_str() == Some("USDT");
                let is_trading = s["status"].as_str() == Some("TRADING");
                is_usdt && is_trading
            })
            .map(|s| s["symbol"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        let res = self.client
            .get("https://api.binance.com/api/v3/klines")
            .query(&[
                ("symbol", symbol),
                ("interval", interval),
                ("limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to send request to Binance")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("API Error: {}", res.status()));
        }

        let rows: Vec<Vec<Value>> = res.json().await.context("Failed to parse JSON")?;
        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                klines.push(KlineData {
                    time: t.as_i64().unwrap_or(0),
                    open: o.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    high: h.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    low: l.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    close: c.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    volume: v.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_binance_fetch_candles() {
        // Arrange
        let binance = Binance::new();
        let symbol = "BTCUSDT";
        let interval = "1m";

        // Act
        // 実装がないため、ここで panic! が発生しテストは失敗するはずです
        let result = binance.fetch_candles(symbol, interval).await;

        // Assert
        assert!(result.is_ok(), "API call should succeed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
        
        // データの整合性チェック
        let first_candle = &candles[0];
        assert!(first_candle.close > 0.0);
        assert!(first_candle.volume >= 0.0);
    }
}
