use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// Bitget取引所の実装。
/// 
/// Bitget V2 API (REST) を使用してデータを取得します。
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

    /// Bitget V2 Spot Symbols APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.bitget.com/api/v2/spot/public/symbols")
            .send()
            .await
            .context("Failed to get symbols from Bitget")?;

        let json: Value = res.json().await.context("Failed to parse Bitget symbols JSON")?;
        
        let symbols = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Bitget format"))?
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

    /// Bitget V2 Kline APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // Bitget V2では "1m" ではなく "1min" 形式を期待するため変換
        let granularity = match interval {
            "1m" => "1min",
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
            return Err(anyhow::anyhow!("Bitget API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse Bitget kline JSON")?;
        let rows = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Bitget response format: 'data' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // Bitget V2 [ts, o, h, l, c, v, ...]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bitget_fetch_candles() {
        let bitget = Bitget::new();
        let result = bitget.fetch_candles("BTCUSDT", "1min").await;
        assert!(result.is_ok(), "Bitget API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
    }
}