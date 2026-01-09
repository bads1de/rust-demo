use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// OKX取引所の実装。
/// 
/// OKX API v5 (REST) を使用してデータを取得します。
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

    /// OKX V5 Instruments APIを使用して取引可能なSPOT/USDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://www.okx.com/api/v5/public/instruments")
            .query(&[("instType", "SPOT")])
            .send()
            .await
            .context("Failed to get instruments from OKX")?;

        let json: Value = res.json().await.context("Failed to parse OKX instruments JSON")?;

        let symbols = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid OKX format"))?
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

    /// OKX V5 Kline APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        let res = self.client
            .get("https://www.okx.com/api/v5/market/candles")
            .query(&[
                ("instId", symbol),
                ("bar", interval),
                ("limit", "300"), 
            ])
            .send()
            .await
            .context("Failed to fetch candles from OKX")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("OKX API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse OKX kline JSON")?;
        let rows = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid OKX response format: 'data' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // OKX: [ts, o, h, l, c, vol, ...]
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

        // 降順で返るため昇順に反転
        klines.reverse();

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
    async fn test_okx_fetch_candles() {
        let okx = Okx::new();
        // OKXのシンボル形式は "BTC-USDT"
        let result = okx.fetch_candles("BTC-USDT", "1m").await;
        assert!(result.is_ok(), "OKX API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
    }
}