use serde::{Deserialize, Deserializer, Serialize};

/// 文字列として来る数値を `f64` としてデシリアライズするためのカスタムヘルパー関数。
///
/// Binance API は数値を `"123.45"` のような文字列として返すため、
/// Rustの `f64` 型に直接マッピングできません。この関数を `#[serde(deserialize_with = ...)]` で指定することで、
/// 文字列 -> f64 の変換を自動化します。
pub fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

/// ローソク足データ (Kline/Candlestick data) を表す構造体。
///
/// Binance API のレスポンス形式に合わせています。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineData {
    /// タイムスタンプ (Unix time in milliseconds)
    #[serde(rename = "t")]
    pub time: i64,
    
    /// 始値 (Open price)。文字列からf64へ変換されます。
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

/// フロントエンドに送信するための、ローソク足データとテクニカル指標を統合した構造体。
///
/// `#[serde(flatten)]` アトリビュートにより、JSON化された際は `kline` の中身が
/// この構造体のトップレベルフィールドとして展開されます。
/// 例: `{ "symbol": "BTC", "time": 123, "open": 100, ..., "sma": 99, ... }`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineWithIndicator {
    /// 通貨ペア名 (例: "BTCUSDT")
    pub symbol: String,
    
    /// 元のローソク足データ (フラット展開される)
    #[serde(flatten)]
    pub kline: KlineData,
    
    /// 単純移動平均線 (Simple Moving Average)
    pub sma: Option<f64>,
    /// ボリンジャーバンド上部バンド (+2σ)
    pub upper_band: Option<f64>,
    /// ボリンジャーバンド下部バンド (-2σ)
    pub lower_band: Option<f64>,
    /// 相対力指数 (Relative Strength Index)
    pub rsi: Option<f64>,
    /// MACD (Moving Average Convergence Divergence) ライン
    pub macd: Option<f64>,
    /// MACD シグナルライン
    pub macd_signal: Option<f64>,
    /// MACD ヒストグラム
    pub macd_hist: Option<f64>,
    /// ストキャスティクス %K
    pub stoch_k: Option<f64>,
    /// ストキャスティクス %D
    pub stoch_d: Option<f64>,
}

/// WebSocket から受信する個別のメッセージ構造体。
///
/// Binanceのklineストリームのペイロードに対応します。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineMessage {
    /// シンボル名 (例: "BTCUSDT")
    #[serde(rename = "s")]
    pub symbol: String,
    /// Kline データの実体
    #[serde(rename = "k")]
    pub kline: KlineData,
}

/// 複数のストリームを同時に購読した際の WebSocket ラッパー構造体。
///
/// Combined Streams モードでは、個別のメッセージがこの形式でラップされて送られてきます。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CombinedStreamPayload {
    /// ストリーム名 (例: "btcusdt@kline_1m")
    pub stream: String,
    /// 実際のデータペイロード
    pub data: KlineMessage,
}