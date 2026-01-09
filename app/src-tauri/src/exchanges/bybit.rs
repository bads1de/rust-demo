use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// Bybit取引所の実装。
/// 
/// Bybit V5 API (REST) を使用してデータを取得します。
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

    /// Bybit V5 Instruments Info APIを使用して取引可能なUSDTペアを取得します。
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.bybit.com/v5/market/instruments-info")
            .query(&[("category", "spot")])
            .send()
            .await
            .context("Failed to get instruments from Bybit")?;

        let json: Value = res.json().await.context("Failed to parse Bybit instruments JSON")?;

        let symbols = json["result"]["list"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Bybit instruments format"))?
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

    /// Bybit V5 Kline APIを使用してデータを取得します。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>> {
        // ... (existing code omitted) ...
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
            .context("Failed to fetch candles from Bybit")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("Bybit API Error: {}", res.status()));
        }

        let json: Value = res.json().await.context("Failed to parse Bybit kline JSON")?;
        
        let rows = json["result"]["list"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Bybit response format: 'list' not found"))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
            ) {
                klines.push(KlineData {
                    // Bybitのタイムスタンプは文字列型（ミリ秒）
                    time: t.as_str().unwrap_or("0").parse().unwrap_or(0),
                    open: o.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    high: h.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    low: l.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    close: c.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    volume: v.as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }
        
        // BybitのAPIは新しい順(降順)でデータを返すため、チャート表示(昇順)のために反転させる
        klines.reverse();

        Ok(klines)
    }

    fn websocket_url(&self) -> &str {
        "wss://stream.bybit.com/v5/public/spot"
    }

    fn websocket_subscription_payload(&self, symbols: &[String]) -> Result<String> {
        let args: Vec<String> = symbols.iter()
            .map(|s| format!("kline.1.{}", s))
            .collect();
        
        let payload = serde_json::json!({
            "op": "subscribe",
            "args": args
        });
        
        Ok(payload.to_string())
    }

    fn parse_websocket_message(&self, msg: &str) -> Result<Option<(String, KlineData)>> {
        // Ping/Pong等は無視 (Bybitはop: pingを送る必要があるが、今回は受信解析のみ)
        // データメッセージ: {"topic": "kline.1.BTCUSDT", "data": [...]}
        let json: Value = serde_json::from_str(msg)?;

        if let Some(topic) = json["topic"].as_str() {
            if topic.starts_with("kline.1.") {
                let symbol = topic.replace("kline.1.", "");
                
                if let Some(data_list) = json["data"].as_array() {
                    if let Some(data) = data_list.first() {
                        // Bybit WS fields: start, open, high, low, close, volume, turnover
                        let t = data["start"].as_i64().unwrap_or(0); // WS returns timestamp as number (long)
                        let o = data["open"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let h = data["high"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let l = data["low"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let c = data["close"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let v = data["volume"].as_str().unwrap_or("0").parse().unwrap_or(0.0);

                        return Ok(Some((symbol, KlineData {
                            time: t,
                            open: o,
                            high: h,
                            low: l,
                            close: c,
                            volume: v,
                        })));
                    }
                }
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bybit_fetch_candles() {
        let bybit = Bybit::new();
        // Bybit V5 API: interval "1" は 1分足を指す
        let result = bybit.fetch_candles("BTCUSDT", "1").await;
        assert!(result.is_ok(), "Bybit API call failed");
                let candles = result.unwrap();
                assert!(!candles.is_empty(), "Should return candle data");
            }
        
            #[test]
            fn test_bybit_parse_websocket() {
                let bybit = Bybit::new();
                let msg = r#"{"topic":"kline.1.BTCUSDT","data":[{"start":1672531200000,"end":1672531260000,"interval":"1","open":"16500.5","close":"16501.0","high":"16502.0","low":"16500.0","volume":"10.5","turnover":"173260.5","confirm":false,"timestamp":1672531201000}],"ts":1672531201000,"type":"snapshot"}"#;
                
                let result = bybit.parse_websocket_message(msg).unwrap();
                assert!(result.is_some());
                let (symbol, kline) = result.unwrap();
                assert_eq!(symbol, "BTCUSDT");
                assert_eq!(kline.close, 16501.0);
            }
        }
        