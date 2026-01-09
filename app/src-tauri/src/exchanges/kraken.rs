use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// Kraken取引所の実装。
/// 
/// Kraken API (REST) を使用してデータを取得します。
/// KrakenのAPIはレスポンス形式が動的（ペア名がキーになる）であるため、特殊なパース処理を行っています。
pub struct Kraken {
    client: reqwest::Client,
}

impl Kraken {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for Kraken {
    fn id(&self) -> &str {
        "kraken"
    }

    /// Kraken AssetPairs APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.kraken.com/0/public/AssetPairs")
            .send()
            .await
            .context("Failed to get asset pairs from Kraken")?;

        let json: Value = res.json().await.context("Failed to parse Kraken pairs JSON")?;
        
        // Kraken returns a map: { result: { "XXBTZUSD": { ... }, ... } }
        let result = json["result"].as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid Kraken format"))?;

        let mut symbols = Vec::new();
        for (_key, info) in result {
            // "wsname" (例: "XBT/USDT") があればそれを使用する
            // 内部名の "XXBTZUSD" などはユーザーには分かりにくいため。
            if let Some(wsname) = info["wsname"].as_str() {
                if wsname.ends_with("/USDT") {
                    // クエリに使用可能な "altname" (例: "XBTUSDT") をリストに追加する
                    if let Some(altname) = info["altname"].as_str() {
                         symbols.push(altname.to_string());
                    }
                }
            }
        }
        
        Ok(symbols)
    }

    /// Kraken OHLC APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // Kraken interval parameter (minutes integer)
        // api.rsで "1" に変換されて渡ってくることを期待
        let interval_val = match interval {
            "1m" => "1",
            _ => interval,
        };

        let res = self.client
            .get("https://api.kraken.com/0/public/OHLC")
            .query(&[
                ("pair", symbol),
                ("interval", interval_val),
            ])
            .send()
            .await
            .context("Failed to fetch candles from Kraken")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("Kraken API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse Kraken OHLC JSON")?;
        
        // result: { "PAIRNAME": [ ... ], "last": ... }
        // ペア名は正規化されたり変わったりするため、"last" 以外のキーで配列であるものを探す
        let result = json["result"].as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid Kraken response: 'result' not found"))?;

        let mut rows = None;
        for (key, val) in result {
            if key != "last" && val.is_array() {
                rows = val.as_array();
                break;
            }
        }
        
        let rows = rows.ok_or_else(|| anyhow::anyhow!("No candle data found in Kraken response"))?;
        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // Kraken: [time, open, high, low, close, vwap, volume, count]
            // 値は数値または文字列（APIバージョンによる）の可能性があるため柔軟にパース
            if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(6)
            ) {
                // Time is seconds
                let time = if t.is_i64() { t.as_i64().unwrap() } else { t.as_f64().unwrap() as i64 } * 1000;
                
                let parse_f64 = |val: &Value| -> f64 {
                    if let Some(s) = val.as_str() {
                        s.parse().unwrap_or(0.0)
                    } else if let Some(n) = val.as_f64() {
                        n
                    } else {
                        0.0
                    }
                };

                klines.push(KlineData {
                    time,
                    open: parse_f64(o),
                    high: parse_f64(h),
                    low: parse_f64(l),
                    close: parse_f64(c),
                    volume: parse_f64(v),
                });
            }
        }

        // Krakenは通常昇順（古い順）でデータを返すため、反転は不要

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
    async fn test_kraken_fetch_candles() {
        let exchange = Kraken::new();
        // "XBTUSDT" is the altname for Bitcoin/Tether
        let result = exchange.fetch_candles("XBTUSDT", "1").await;
        assert!(result.is_ok(), "Kraken API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
    }

    #[tokio::test]
    async fn test_kraken_get_symbols() {
        let exchange = Kraken::new();
        let result = exchange.get_symbols().await;
        assert!(result.is_ok());
        let symbols = result.unwrap();
        // 確認: USDTペアが含まれているか
        assert!(symbols.iter().any(|s| s.contains("USDT")));
    }
}