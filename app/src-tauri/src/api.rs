use tauri::State;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use crate::state::AppState;

/// 過去のローソク足データを Binance API から取得し、指標を計算して返す
///
/// # Returns
/// - `Ok(Vec<KlineWithIndicator>)`: 取得に成功した場合、ローソク足データと指標のリストを返します。
/// - `Err(String)`: HTTPリクエストやパースに失敗した場合、エラーメッセージを返します。
#[tauri::command]
pub async fn fetch_candles(
    state: State<'_, AppState>,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    // 設定を更新
    {
        let mut config = state.config.lock().map_err(|_| "Failed to lock config".to_string())?;
        config.period = period;
        config.multiplier = multiplier;
    }

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

    // 共有ステートを更新 (Mutexロックを取得)
    {
        let mut state_klines = state.klines.lock().map_err(|_| "Failed to lock state".to_string())?;
        *state_klines = klines.clone();
    }

    // 各指標を計算 (Rustの並行処理を使えばさらに高速化可能だが、ここでは順次実行)
    let sma_values = calculate_sma(&klines, period);
    let (upper_band, lower_band) = calculate_bollinger_bands(&klines, period, multiplier);
    let rsi_values = calculate_rsi(&klines, 14);
    let (macd, signal, hist) = calculate_macd(&klines, 12, 26, 9);
    let (stoch_k, stoch_d) = calculate_stoch(&klines, 14, 3, 3);

    // データを結合
    let combined = klines.into_iter().enumerate()
        .map(|(i, kline)| KlineWithIndicator {
            kline,
            sma: sma_values[i],
            upper_band: upper_band[i],
            lower_band: lower_band[i],
            rsi: rsi_values[i],
            macd: macd[i],
            macd_signal: signal[i],
            macd_hist: hist[i],
            stoch_k: stoch_k[i],
            stoch_d: stoch_d[i],
        })
        .collect();

    Ok(combined)
}