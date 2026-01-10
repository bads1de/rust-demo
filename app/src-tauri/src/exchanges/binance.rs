use async_trait::async_trait;
use crate::models::KlineData;
use super::Exchange;
use anyhow::{Result, Context};
use serde_json::Value;

/// Binance取引所機能の実装構造体。
///
/// Binance API v3 (REST) および WebSocket API を使用して、
/// 市場データの取得やリアルタイム更新を行います。
pub struct Binance {
    /// REST APIリクエスト送信用のHTTPクライアント
    client: reqwest::Client,
}

impl Binance {
    /// 新しいBinanceクライアントインスタンスを生成します。
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

use crate::models::CombinedStreamPayload;



#[async_trait]
impl Exchange for Binance {

    /// 取引所の識別IDを返します。
    fn id(&self) -> &str {
        "binance"
    }

    /// 取引可能なシンボル一覧を取得します。
    ///
    /// `GET /api/v3/exchangeInfo` を呼び出し、以下の条件に一致するシンボルのみを返します：
    /// - 決済通貨（quoteAsset）が "USDT" であること
    /// - ステータスが "TRADING"（取引可能）であること
    async fn get_symbols(&self) -> Result<Vec<String>> {
        let res = self.client
            .get("https://api.binance.com/api/v3/exchangeInfo")
            .send()
            .await
            .context("Failed to get exchange info from Binance")?;

        let json: Value = res.json().await.context("Failed to parse Binance exchange info JSON")?;
        
        let symbols = json["symbols"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid Binance exchange info format"))?
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

    /// 指定されたシンボルと時間足のローソク足（Klines）データを取得します。
    ///
    /// `GET /api/v3/klines` を使用します。
    /// データ取得数は最大1000件に固定しています。
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
            return Err(anyhow::anyhow!("Binance API Error: {}", res.status()));
        }

        let rows: Vec<Vec<Value>> = res.json().await.context("Failed to parse Binance kline JSON")?;
        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            // 配列のインデックスに基づいてデータをパース:
            // 0: Open time, 1: Open, 2: High, 3: Low, 4: Close, 5: Volume
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

    /// WebSocket接続先URLを返します。
    ///
    /// BinanceのCombined Streamsエンドポイントを使用します。
    fn websocket_url(&self) -> &str {
        "wss://stream.binance.com:9443/stream"
    }

    /// WebSocket購読リクエストのペイロードを作成します。
    ///
    /// 指定された全シンボルに対して、1分足（kline_1m）の更新を購読するためのJSONメッセージを生成します。
    fn websocket_subscription_payload(&self, symbols: &[String]) -> Result<String> {
        let params: Vec<String> = symbols.iter()
            .map(|s| format!("{}@kline_1m", s.to_lowercase()))
            .collect();
        
        let payload = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": params,
            "id": 1
        });
        
        Ok(payload.to_string())
    }

    /// 受信したWebSocketメッセージをパースし、シンボルとKlineデータに変換します。
    ///
    /// 期待されるフォーマット（Combined Stream）:
    /// `{"stream":"<symbol>@kline_1m", "data": {"e":"kline", "s":"<symbol>", "k":{...}}}`
    ///
    /// Klineデータが含まれないメッセージ（購読確認やPingなど）は無視され、Ok(None)が返されます。
    fn parse_websocket_message(&self, msg: &str) -> Result<Option<(String, KlineData)>> {
        // "k" (Klineデータ) が含まれていないメッセージは無視（制御メッセージ等）
        if !msg.contains(r#""k""#) {
            return Ok(None);
        }

        let parsed: CombinedStreamPayload = serde_json::from_str(msg)?;
        Ok(Some((parsed.data.symbol, parsed.data.kline)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_binance_fetch_candles() {
        let binance = Binance::new();
        let result = binance.fetch_candles("BTCUSDT", "1m").await;
        assert!(result.is_ok(), "Binance API call failed");
        let candles = result.unwrap();
        assert!(!candles.is_empty(), "Should return candle data");
    }
}