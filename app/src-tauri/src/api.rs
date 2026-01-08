use tauri::State;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use crate::state::AppState;

/// 指定された通貨ペアの過去データをBinance APIから取得し、
/// テクニカル指標を計算して返す非同期関数。
///
/// チャートの初期表示時にフロントエンドから呼び出されます。
///
/// # Arguments
/// * `state` - アプリケーション共有状態。取得したデータを保存するために使用。
/// * `symbol` - 通貨ペア (例: "BTCUSDT")。
/// * `period` - 指標の計算期間。
/// * `multiplier` - ボリンジャーバンドの倍率。
pub async fn fetch_candles(
    state: State<'_, AppState>,
    symbol: String,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    // 設定を更新 (全銘柄で共有される設定として扱う)
    // 複数のチャートが同時に初期化される場合、ロック競合が起きる可能性があるが、
    // ここでのロック時間は極めて短いため問題ない。
    {
        let mut config = state.config.lock().map_err(|_| "Failed to lock config".to_string())?;
        config.period = period;
        config.multiplier = multiplier;
    }

    // HTTPクライアント作成
    let client = reqwest::Client::new();
    
    // Binance API (Klines endpoint) へリクエスト
    // limit=1000 で過去1000分（約16時間）のデータを取得
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

    // レスポンスのパース (配列の配列として返る)
    let rows: Vec<Vec<serde_json::Value>> = res.json().await.map_err(|e| e.to_string())?;
    let mut klines = Vec::new();

    // データの変換 (String -> f64 など)
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

    // 取得したデータをメモリ上の状態(AppState)に保存
    // これにより、後続のWebSocket更新時に過去データを参照して指標計算が可能になる
    {
        let mut klines_map = state.klines.lock().map_err(|_| "Lock error".to_string())?;
        klines_map.insert(symbol.clone(), klines.clone());
    }

    // 全期間に対して指標を一括計算
    let sma_vals = calculate_sma(&klines, period);
    let (u_vals, l_vals) = calculate_bollinger_bands(&klines, period, multiplier);
    let rsi_vals = calculate_rsi(&klines, 14);
    let (macd, signal, hist) = calculate_macd(&klines, 12, 26, 9);
    let (stoch_k, stoch_d) = calculate_stoch(&klines, 14, 3, 3);

    // 計算結果を結合して返却用構造体を作成
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