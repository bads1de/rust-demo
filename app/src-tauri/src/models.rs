use serde::{Deserialize, Deserializer, Serialize};

/// 文字列として来る数値を f64 としてデシリアライズするためのヘルパー関数
pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

/// ローソク足データ (Kline/Candlestick data)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineData {
    #[serde(rename = "t")]
    pub time: i64,
    #[serde(rename = "o", deserialize_with = "deserialize_f64_from_string")]
    pub open: f64,
    #[serde(rename = "h", deserialize_with = "deserialize_f64_from_string")]
    pub high: f64,
    #[serde(rename = "l", deserialize_with = "deserialize_f64_from_string")]
    pub low: f64,
    #[serde(rename = "c", deserialize_with = "deserialize_f64_from_string")]
    pub close: f64,
    #[serde(rename = "v", deserialize_with = "deserialize_f64_from_string")]
    pub volume: f64,
}

/// ローソク足データとテクニカル指標を合わせた構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineWithIndicator {
    /// 通貨ペア名 (例: "BTCUSDT")
    pub symbol: String,
    /// 元のローソク足データ
    #[serde(flatten)]
    pub kline: KlineData,
    pub sma: Option<f64>,
    pub upper_band: Option<f64>,
    pub lower_band: Option<f64>,
    pub rsi: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_hist: Option<f64>,
    pub stoch_k: Option<f64>,
    pub stoch_d: Option<f64>,
}

/// WebSocket から受信する個別のメッセージ
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineMessage {
    /// シンボル名
    #[serde(rename = "s")]
    pub symbol: String,
    /// Kline データの実体
    #[serde(rename = "k")]
    pub kline: KlineData,
}

/// 複数のストリームを同時に購読した際の WebSocket ラッパー
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CombinedStreamPayload {
    pub stream: String,
    pub data: KlineMessage,
}
