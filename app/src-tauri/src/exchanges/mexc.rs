use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// MEXC取引所の実装。
/// 
/// MEXC API v3 (Binance互換) を使用してデータを取得します。
pub struct Mexc {
    client: reqwest::Client,
}

impl Mexc {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Mexc {
    fn id(&self) -> &str {
        "mexc"
    }

    /// MEXC Exchange Info APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.mexc.com/api/v3/exchangeInfo")
            .send()
            .await
            .context("Failed to get exchange info from MEXC")?;

        let json: Value = res.json().await.context("Failed to parse MEXC exchange info JSON")?;
        
        let symbols = json["symbols"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid MEXC exchange info format"))?
            .iter()
            .filter(|s| {
                let is_usdt = s["quoteAsset"].as_str() == Some("USDT");
                // MEXC status は "ENABLED" や "TRADING"、あるいは "1" などの場合があるため柔軟に判定
                let status = s["status"].as_str().unwrap_or("");
                let is_enabled = status == "ENABLED" || status == "TRADING" || status == "1"; 
                is_usdt && is_enabled
            })
            .map(|s| s["symbol"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    /// MEXC Kline APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // MEXC V3はBinance互換のため、パラメータ形式は同じ ("1m", "5m" 等)
        let res = self.client
            .get("https://api.mexc.com/api/v3/klines")
            .query(&[
                ("symbol", symbol),
                ("interval", interval),
                ("limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to send request to MEXC")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("MEXC API Error: {}", res.status()));
        }

        let rows: Vec<Vec<Value>> = res.json().await.context("Failed to parse MEXC kline JSON")?;
        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // MEXC (Binance compatible): [t, o, h, l, c, v, ...]
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

    fn websocket_url(&self) -> &str { unimplemented!() }
    fn websocket_subscription_payload(&self, _symbols: &[String]) -> Result<String> { unimplemented!() }
    fn parse_websocket_message(&self, _msg: &str) -> Result<Option<(String, KlineData)>> { unimplemented!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mexc_fetch_candles() {
        let exchange = Mexc::new();
        let result = exchange.fetch_candles("BTCUSDT", "1m").await;
        assert!(result.is_ok(), "MEXC API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
    }

    #[tokio::test]
    async fn test_mexc_get_symbols() {
        let exchange = Mexc::new();
        let result = exchange.get_symbols().await;
        assert!(result.is_ok());
        let symbols = result.unwrap();
        assert!(symbols.contains(&"BTCUSDT".to_string()));
    }
}