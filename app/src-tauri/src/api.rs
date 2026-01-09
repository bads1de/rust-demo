use tauri::State;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use crate::state::AppState;
use crate::exchanges::{Exchange, binance::Binance, bybit::Bybit, bitget::Bitget, okx::Okx};

/// 取引所インスタンスを生成するヘルパー
fn get_exchange_impl(name: &str) -> Box<dyn Exchange> {
    match name.to_lowercase().as_str() {
        "bybit" => Box::new(Bybit::new()),
        "bitget" => Box::new(Bitget::new()),
        "okx" => Box::new(Okx::new()),
        _ => Box::new(Binance::new()), // Default to Binance
    }
}

/// 全USDT銘柄を取得する (デフォルトはBinanceだが、将来的には引数で指定可能にすべき)
pub async fn get_all_symbols() -> Result<Vec<String>, String> {
    let exchange = Binance::new();
    exchange.get_symbols().await.map_err(|e| e.to_string())
}

/// 過去のローソク足データを取得する
#[tauri::command]
pub async fn fetch_candles(
    state: State<'_, AppState>,
    exchange_name: String,
    symbol: String,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    {
        let mut config = state.config.lock().map_err(|_| "Failed to lock config".to_string())?;
        config.period = period;
        config.multiplier = multiplier;
    }

    let exchange = get_exchange_impl(&exchange_name);
    
    // Bybit等のインターバル形式変換が必要ならここでやる
    // 今回は簡単のため "1m" (Binance/Bitget/OKX) と "1" (Bybit) の違いを吸収するロジックを入れる
    let interval = match exchange.id() {
        "bybit" => "1",
        _ => "1m",
    };

    let klines = exchange.fetch_candles(&symbol, interval).await
        .map_err(|e| e.to_string())?;

    // 取得したデータを状態にキャッシュ
    state.klines.insert(symbol.clone(), klines.clone());

    // テクニカル指標の計算
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