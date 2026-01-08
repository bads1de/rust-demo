use tauri::State;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use crate::state::AppState;
use serde_json::Value;

/// Binanceから取引可能な全USDT銘柄を取得する
pub async fn get_all_symbols() -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.binance.com/api/v3/exchangeInfo")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: Value = res.json().await.map_err(|e| e.to_string())?;
    
    let symbols_raw = json["symbols"]
        .as_array()
        .ok_or("Invalid exchangeInfo format")?;

    // デバッグ用: 最初の5件の構造を表示
    /*
    for (i, s) in symbols_raw.iter().enumerate().take(5) {
        println!("Symbol {}: status={}, quoteAsset={}, permissions={:?}", 
            s["symbol"], s["status"], s["quoteAsset"], s["permissions"]);
    }
    */

    let symbols = symbols_raw.iter()
        .filter(|s| {
            // かなり緩い条件で試す
            let is_usdt = s["quoteAsset"].as_str() == Some("USDT");
            let is_trading = s["status"].as_str() == Some("TRADING");
            is_usdt && is_trading
        })
        .map(|s| s["symbol"].as_str().unwrap().to_string())
        .collect();

    Ok(symbols)
}

/// 過去のローソク足データを取得する
#[tauri::command]
pub async fn fetch_candles(
    state: State<'_, AppState>,
    symbol: String,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    {
        let mut config = state.config.lock().map_err(|_| "Failed to lock config".to_string())?;
        config.period = period;
        config.multiplier = multiplier;
    }

    let client = reqwest::Client::new();
    let res = client
        .get("https://api.binance.com/api/v3/klines")
        .query(&[
            ("symbol", symbol.as_str()),
            ("interval", "1m"),
            ("limit", "1000"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API Error for {}: {}", symbol, res.status()));
    }

    let rows: Vec<Vec<serde_json::Value>> = res.json().await.map_err(|e| e.to_string())?;
    let mut klines = Vec::new();

    for row in rows {
        if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)) {
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

    state.klines.insert(symbol.clone(), klines.clone());

    let sma_vals = calculate_sma(&klines, period);
    let (u_vals, l_vals) = calculate_bollinger_bands(&klines, period, multiplier);
    let rsi_vals = calculate_rsi(&klines, 14);
    let (macd, signal, hist) = calculate_macd(&klines, 12, 26, 9);
    let (stoch_k, stoch_d) = calculate_stoch(&klines, 14, 3, 3);

    let combined = klines.into_iter().enumerate()
        .map(|(i, kline)| KlineWithIndicator {
            symbol: symbol.clone(),
            kline,
            sma: sma_vals[i],
            upper_band: u_vals[i],
            lower_band: l_vals[i],
            rsi: rsi_vals[i],
            macd: macd[i],
            macd_signal: signal[i],
            macd_hist: hist[i],
            stoch_k: stoch_k[i],
            stoch_d: stoch_d[i],
        })
        .collect();

    Ok(combined)
}
