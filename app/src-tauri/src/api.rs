use tauri::State;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use crate::state::AppState;
use crate::exchanges::{Exchange, binance::Binance, bybit::Bybit, bitget::Bitget, okx::Okx, kucoin::KuCoin, kraken::Kraken, gate::Gate, mexc::Mexc};

/// 指定された名前の取引所インスタンス（トレイトオブジェクト）を生成します。
/// 
/// 実行時に動的に型を決定するため `Box<dyn Exchange>` を返します。
pub fn get_exchange_impl(name: &str) -> Box<dyn Exchange> {
    match name.to_lowercase().as_str() {
        "bybit" => Box::new(Bybit::new()),
        "bitget" => Box::new(Bitget::new()),
        "okx" => Box::new(Okx::new()),
        "kucoin" => Box::new(KuCoin::new()),
        "kraken" => Box::new(Kraken::new()),
        "gate" => Box::new(Gate::new()),
        "mexc" => Box::new(Mexc::new()),
        _ => Box::new(Binance::new()), // デフォルトはBinance
    }
}

/// 全USDT銘柄を取得します。
/// 現時点ではデフォルトの取引所（Binance）の銘柄リストを返します。
pub async fn get_all_symbols() -> Result<Vec<String>, String> {
    let exchange = Binance::new();
    exchange.get_symbols().await.map_err(|e| e.to_string())
}

/// フロントエンドからのリクエストを受け、指定された取引所からローソク足データを取得し、
/// テクニカル指標を計算して返します。
/// 
/// # Arguments
/// * `exchange_name` - 取引所名 ("binance", "bybit", etc.)
/// * `symbol` - 通貨ペア名
/// * `period` - 移動平均線などの計算期間
/// * `multiplier` - ボリンジャーバンドの倍率
#[tauri::command]
pub async fn fetch_candles(
    state: State<'_, AppState>,
    exchange_name: String,
    symbol: String,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    // 1. 計算設定の更新
    {
        let mut config = state.config.lock().map_err(|_| "Failed to lock config".to_string())?;
        config.period = period;
        config.multiplier = multiplier;
    }

    // 2. 取引所インスタンスの取得
    let exchange = get_exchange_impl(&exchange_name);
    
    // 3. 取引所ごとの時間足形式の調整
    // 例: Binanceは "1m", Bybitは "1" を期待する
    let interval = match exchange.id() {
        "bybit" => "1",
        "kraken" => "1",
        _ => "1m", // Binance, Bitget, OKX, KuCoin, Gate, Mexc
    };

    // 4. 生データの取得
    let klines = exchange.fetch_candles(&symbol, interval).await
        .map_err(|e| e.to_string())?;

    // 5. 状態管理（AppState）への保存
    // リアルタイム更新の基準データとしてメモリ上に保持します
    state.klines.insert(symbol.clone(), klines.clone());

    // 6. テクニカル指標の計算（taクレートと自作 indicators.rs を使用）
    let sma_vals = calculate_sma(&klines, period);
    let (u_vals, l_vals) = calculate_bollinger_bands(&klines, period, multiplier);
    let rsi_vals = calculate_rsi(&klines, 14);
    let (macd, signal, hist) = calculate_macd(&klines, 12, 26, 9);
    let (stoch_k, stoch_d) = calculate_stoch(&klines, 14, 3, 3);

    // 7. データの統合と返却
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

/// スキャナープリセット一覧を取得します。
#[tauri::command]
pub fn get_scanner_presets() -> Vec<crate::scanner::ScannerPreset> {
    crate::scanner::get_default_presets()
}

/// 指定されたプリセットで全銘柄をスキャンします。
///
/// # Arguments
/// * `preset_id` - プリセットID (例: "rsi_overbought")
/// * `exchange_name` - 取引所名 (例: "binance")
/// * `interval` - 時間足 (例: "1h")
#[tauri::command]
pub async fn run_scanner(
    preset_id: String,
    exchange_name: String,
    interval: String,
) -> Result<Vec<crate::scanner::ScanResult>, String> {
    use crate::scanner::{get_default_presets, ScanResult};
    
    // 1. プリセットを取得
    let presets = get_default_presets();
    let preset = presets.iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| format!("Preset '{}' not found", preset_id))?;
    
    // 2. 取引所インスタンスの取得
    let exchange = get_exchange_impl(&exchange_name);
    
    // 3. 取引所ごとの時間足形式の調整
    let interval_str = match (exchange.id(), interval.as_str()) {
        ("bybit", "1h") => "60",
        ("bybit", "4h") => "240",
        ("bybit", "1d") => "D",
        ("kraken", "1h") => "60",
        ("kraken", "4h") => "240",
        ("kraken", "1d") => "1440",
        (_, i) => i,
    };
    
    // 4. 全シンボルを取得
    let symbols = exchange.get_symbols().await.map_err(|e| e.to_string())?;
    
    // 5. 並列でデータ取得＆フィルタリング（最大100銘柄に制限）
    let limited_symbols: Vec<_> = symbols.into_iter().take(100).collect();
    let mut results = Vec::new();
    
    for symbol in limited_symbols {
        // データ取得
        let klines = match exchange.fetch_candles(&symbol, interval_str).await {
            Ok(k) if k.len() >= 30 => k,
            _ => continue, // データ不足はスキップ
        };
        
        // テクニカル指標の計算
        let sma_vals = calculate_sma(&klines, 20);
        let (upper_vals, lower_vals) = calculate_bollinger_bands(&klines, 20, 2.0);
        let rsi_vals = calculate_rsi(&klines, 14);
        let (macd_vals, signal_vals, _) = calculate_macd(&klines, 12, 26, 9);
        let (stoch_k_vals, stoch_d_vals) = calculate_stoch(&klines, 14, 3, 3);
        
        // 最新足のインデックス
        let last = klines.len() - 1;
        let prev = if last > 0 { last - 1 } else { 0 };
        
        // ScanResultを構築
        let scan_result = ScanResult {
            exchange: exchange_name.clone(),
            symbol: symbol.clone(),
            price: klines[last].close,
            rsi: rsi_vals[last],
            stoch_k: stoch_k_vals[last],
            stoch_d: stoch_d_vals[last],
            macd: macd_vals[last],
            macd_signal: signal_vals[last],
            sma: sma_vals[last],
            upper_band: upper_vals[last],
            lower_band: lower_vals[last],
            prev_macd: macd_vals[prev],
            prev_macd_signal: signal_vals[prev],
        };
        
        // フィルタ条件をチェック
        if scan_result.matches_preset(preset) {
            results.push(scan_result);
        }
    }
    
    Ok(results)
}
