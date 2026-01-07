use serde::{Deserialize, Deserializer, Serialize};

/// 文字列として来る数値を f64 としてデシリアライズするためのヘルパー関数
pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

/// ローソク足データ (Kline/Candlestick data) を表す構造体
/// Binance API から取得される個々のデータポイントに対応します。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineData {
    /// タイムスタンプ (Unix time in milliseconds)
    #[serde(rename = "t")]
    pub time: i64,
    /// 始値 (Open price)
    #[serde(rename = "o", deserialize_with = "deserialize_f64_from_string")]
    pub open: f64,
    /// 高値 (High price)
    #[serde(rename = "h", deserialize_with = "deserialize_f64_from_string")]
    pub high: f64,
    /// 安値 (Low price)
    #[serde(rename = "l", deserialize_with = "deserialize_f64_from_string")]
    pub low: f64,
    /// 終値 (Close price)
    #[serde(rename = "c", deserialize_with = "deserialize_f64_from_string")]
    pub close: f64,
    /// 出来高 (Volume)
    #[serde(rename = "v", deserialize_with = "deserialize_f64_from_string")]
    pub volume: f64,
}

/// ローソク足データとテクニカル指標を合わせた構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineWithIndicator {
    /// 元のローソク足データをフラットに展開
    #[serde(flatten)]
    pub kline: KlineData,
    /// 移動平均線 (Simple Moving Average)
    pub sma: Option<f64>,
}

/// WebSocket から受信するメッセージ構造体
/// リアルタイムの Kline データ更新を含みます。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineMessage {
    /// Kline データの実体
    #[serde(rename = "k")]
    pub kline: KlineData,
}
