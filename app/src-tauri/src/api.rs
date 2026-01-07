use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::calculate_sma;

/// 過去のローソク足データを Binance API から取得し、指標を計算して返す
///
/// # Returns
/// - `Ok(Vec<KlineWithIndicator>)`: 取得に成功した場合、ローソク足データと指標のリストを返します。
/// - `Err(String)`: HTTPリクエストやパースに失敗した場合、エラーメッセージを返します。
#[tauri::command]
pub async fn fetch_candles() -> Result<Vec<KlineWithIndicator>, String> {
    // HTTPクライアントの生成
    let client = reqwest::Client::new();
    
    // Binance API へリクエスト送信 (BTCUSDT, 1分足, 1000件)
    let res = client
        .get("https://api.binance.com/api/v3/klines")
        .query(&[
            ("symbol", "BTCUSDT"),
            ("interval", "1m"),
            ("limit", "1000"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // レスポンスステータスの確認
    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    // JSON レスポンスをパース (配列の配列として返ってくるため)
    let rows: Vec<Vec<serde_json::Value>> = res.json().await.map_err(|e| e.to_string())?;
    let mut klines = Vec::new();

    // 各行を KlineData 構造体に変換
    for row in rows {
        if let (Some(time_val), Some(open_val), Some(high_val), Some(low_val), Some(close_val), Some(volume_val)) = (
            row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
        ) {
             let time = time_val.as_i64().unwrap_or(0);
             // parse::<f64>() で数値に変換。失敗したら 0.0 とする
             let open = open_val.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
             let high = high_val.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
             let low = low_val.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
             let close = close_val.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
             let volume = volume_val.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

             klines.push(KlineData {
                 time, open, high, low, close, volume
             });
        }
    }

    // SMA (20期間) を計算
    let sma_values = calculate_sma(&klines, 20);

    // データを結合
    let combined = klines.into_iter().zip(sma_values.into_iter())
        .map(|(kline, sma)| KlineWithIndicator { kline, sma })
        .collect();

    Ok(combined)
}
