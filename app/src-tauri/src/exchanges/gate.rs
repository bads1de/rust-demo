use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// Gate.io 取引所の実装。
/// 
/// Gate.io API v4 (REST) を使用してデータを取得します。
pub struct Gate {
    client: reqwest::Client,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Gate {
    fn id(&self) -> &str {
        "gate"
    }

    /// Gate.io Currency Pairs APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.gateio.ws/api/v4/spot/currency_pairs")
            .send()
            .await
            .context("Failed to get currency pairs from Gate.io")?;

        let json: Value = res.json().await.context("Failed to parse Gate.io pairs JSON")?;

        // Gate: [ { "id": "BTC_USDT", "quote": "USDT", "trade_status": "tradable", ... }, ... ]
        let symbols = json.as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Gate.io format"))?
            .iter()
            .filter(|s| {
                let quote = s["quote"].as_str() == Some("USDT");
                let status = s["trade_status"].as_str() == Some("tradable");
                quote && status
            })
            .map(|s| s["id"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    /// Gate.io Candlesticks APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // Gate.io interval format: "10s", "1m", "5m", "30m", "1h", "4h", "8h", "1d", "7d"
        // api.rsからは "1m" が渡されるのでそのまま使用可能。
        let res = self.client
            .get("https://api.gateio.ws/api/v4/spot/candlesticks")
            .query(&[
                ("currency_pair", symbol),
                ("interval", interval),
                ("limit", "1000"),
            ])
            .send()
            .await
            .context("Failed to fetch candles from Gate.io")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("Gate.io API Error: {}", res.status()));
        }

        let rows: Vec<Vec<Value>> = res.json().await.context("Failed to parse Gate.io kline JSON")?;
        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // Gate.io order: [time, volume, close, high, low, open]
            // 他の取引所と異なり、Volumeが2番目、Openが最後に来る点に注意。
            if let (Some(t), Some(v), Some(c), Some(h), Some(l), Some(o)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                // timeは文字列の秒単位
                let time_sec = t.as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);

                klines.push(KlineData {
                    time: time_sec * 1000,
                    open: o.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    high: h.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    low: l.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    close: c.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    volume: v.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }

        // Gate.ioは通常昇順（古い順）でデータを返すため、反転不要
        // (APIドキュメントには明記がないが、標準的な挙動として確認済み)

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
    async fn test_gate_fetch_candles() {
        let exchange = Gate::new();
        // Gate.io uses "BTC_USDT" format
        let result = exchange.fetch_candles("BTC_USDT", "1m").await;
        assert!(result.is_ok(), "Gate.io API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
        assert!(candles[0].close > 0.0);
    }

    #[tokio::test]
    async fn test_gate_get_symbols() {
        let exchange = Gate::new();
        let result = exchange.get_symbols().await;
        assert!(result.is_ok());
        let symbols = result.unwrap();
        // Check for common pair
        assert!(symbols.contains(&"BTC_USDT".to_string()));
    }
}