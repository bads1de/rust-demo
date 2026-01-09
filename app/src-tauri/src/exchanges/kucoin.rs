use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// KuCoin取引所の実装。
/// 
/// KuCoin API v1 (REST) を使用してデータを取得します。
pub struct KuCoin {
    client: reqwest::Client,
}

impl KuCoin {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for KuCoin {
    fn id(&self) -> &str {
        "kucoin"
    }

    /// KuCoin Symbols APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.kucoin.com/api/v1/symbols")
            .send()
            .await
            .context("Failed to get symbols from KuCoin")?;

        let json: Value = res.json().await.context("Failed to parse KuCoin symbols JSON")?;
        
        let symbols = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid KuCoin format"))?
            .iter()
            .filter(|s| {
                let quote = s["quoteCurrency"].as_str() == Some("USDT");
                let trading = s["enableTrading"].as_bool() == Some(true);
                quote && trading
            })
            .map(|s| s["symbol"].as_str().unwrap_or("").to_string())
            .collect();

        Ok(symbols)
    }

    /// KuCoin Market Candles APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // KuCoin API type: 1min, 3min, 15min, 30min, 1hour, ...
        // api.rsなどから "1m" が渡された場合 "1min" に変換する
        let type_val = match interval {
            "1m" => "1min",
            _ => interval,
        };

        let res = self.client
            .get("https://api.kucoin.com/api/v1/market/candles")
            .query(&[
                ("symbol", symbol),
                ("type", type_val),
            ])
            .send()
            .await
            .context("Failed to fetch candles from KuCoin")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("KuCoin API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse KuCoin kline JSON")?;
        
        let rows = json["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid KuCoin response format: 'data' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // KuCoin order: [time, open, close, high, low, volume, turnover]
            // 注意: Binance等と違い open の次が close
            if let (Some(t), Some(o), Some(c), Some(h), Some(l), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                // timeは秒単位の文字列で返ってくる
                let time_sec = t.as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);

                klines.push(KlineData {
                    time: time_sec * 1000, // ミリ秒に変換
                    open: o.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    high: h.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    low: l.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    close: c.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    volume: v.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }

        // KuCoinは新しいデータ(降順)で返してくるため、チャート用に昇順に反転
        klines.reverse();

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kucoin_fetch_candles() {
        let exchange = KuCoin::new();
        // KuCoin symbol example: "BTC-USDT"
        let result = exchange.fetch_candles("BTC-USDT", "1min").await;
        assert!(result.is_ok(), "KuCoin API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
        // データチェック
        assert!(candles[0].close > 0.0);
    }

    #[tokio::test]
    async fn test_kucoin_get_symbols() {
        let exchange = KuCoin::new();
        let result = exchange.get_symbols().await;
        assert!(result.is_ok());
        let symbols = result.unwrap();
        assert!(symbols.contains(&"BTC-USDT".to_string()));
    }
}