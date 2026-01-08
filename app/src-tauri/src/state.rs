use dashmap::DashMap;
use std::sync::Mutex;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};

/// アプリケーションの設定情報
pub struct Config {
    pub period: usize,
    pub multiplier: f64,
}

/// アプリケーション全体で共有する状態 (State)
/// DashMapを使用することで、数千銘柄の並列更新でもロックの競合を最小限に抑えます。
pub struct AppState {
    /// シンボルごとのローソク足データを保持する高速なスレッドセーフMap
    pub klines: DashMap<String, Vec<KlineData>>,
    /// 計算設定（設定変更は稀なのでこちらはMutexで十分）
    pub config: Mutex<Config>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            klines: DashMap::new(),
            config: Mutex::new(Config {
                period: 20,
                multiplier: 2.0,
            }),
        }
    }

    /// データを更新し、計算結果を返す（DashMap版）
    pub fn update_and_calculate(&self, symbol: &str, new_kline: KlineData) -> Option<KlineWithIndicator> {
        // DashMapのエントリを取得。ロックは自動的に最小範囲で行われる。
        let mut entry = self.klines.entry(symbol.to_string()).or_insert_with(Vec::new);
        let klines = entry.value_mut();

        // データの更新ロジック (既存と同様)
        if klines.is_empty() {
            klines.push(new_kline.clone());
        } else {
            let last_idx = klines.len() - 1;
            if klines[last_idx].time == new_kline.time {
                klines[last_idx] = new_kline.clone();
            } else if klines[last_idx].time < new_kline.time {
                klines.push(new_kline.clone());
                if klines.len() > 2000 { klines.remove(0); }
            } else {
                return None; 
            }
        }

        let (period, multiplier) = {
            let config = self.config.lock().unwrap();
            (config.period, config.multiplier)
        };

        // 計算ロジック
        let sma_vals = calculate_sma(klines, period);
        let (u_vals, l_vals) = calculate_bollinger_bands(klines, period, multiplier);
        let rsi_vals = calculate_rsi(klines, 14);
        let (macd_v, signal_v, hist_v) = calculate_macd(klines, 12, 26, 9);
        let (st_k, st_d) = calculate_stoch(klines, 14, 3, 3);
        
        let i = klines.len() - 1;
        
        Some(KlineWithIndicator {
            symbol: symbol.to_string(),
            kline: new_kline,
            sma: sma_vals[i],
            upper_band: u_vals[i],
            lower_band: l_vals[i],
            rsi: rsi_vals[i],
            macd: macd_v[i],
            macd_signal: signal_v[i],
            macd_hist: hist_v[i],
            stoch_k: st_k[i],
            stoch_d: st_d[i],
        })
    }
}
