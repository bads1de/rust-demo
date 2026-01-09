use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

pub struct Bybit {
    client: reqwest::Client,
}

impl Bybit {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Bybit {
    fn id(&self) -> &str {
        "bybit"
    }

    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.bybit.com/v5/market/instruments-info")
            .query(&[("category", "spot")])
            .send()
            .await
            .context("Failed to get instruments info")?;

        let json: Value = res.json().await.context("Failed to parse JSON")?;

        let symbols = json["result"]["list"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid format"))?
            .iter()
            .filter(|s| {
                let is_usdt = s["quoteCoin"].as_str() == Some("USDT");
                let is_trading = s["status"].as_str() == Some("Trading");
                is_usdt && is_trading
            })
            .map(|s| s["symbol"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        let res = self.client
            .get("https://api.bybit.com/v5/market/kline")
            .query(&[
                ("category", "spot"),
                ("symbol", symbol),
                ("interval", interval),
                ("limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to send request to Bybit")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse JSON")?;
        
        // Bybitのレスポンス形式: { "retCode": 0, "result": { "list": [...] } }
        let rows = json["result"]["list"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: 'list' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        // Bybitのリストは新しい順（降順）で返ってくることが多いので、チャート表示のために昇順（古い順）に反転する必要があるか確認が必要。
        // 通常チャートライブラリは昇順を期待する。
        // APIドキュメントによると "Sort in reverse by startTime" とあるので、reverse()が必要。
        
        for row in rows {
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
        
        // 昇順（古い日付が先頭）に並べ替え
        klines.reverse();

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bybit_fetch_candles() {
        let bybit = Bybit::new();
        // Bybit V5 API: interval "1" = 1 minute
        let result = bybit.fetch_candles("BTCUSDT", "1").await;
        
        assert!(result.is_ok(), "API call should succeed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
        
        let first = &candles[0];
        assert!(first.close > 0.0);
        assert!(first.volume >= 0.0);
    }
}
