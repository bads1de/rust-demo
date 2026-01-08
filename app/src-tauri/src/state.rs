use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};

pub struct Config {
    pub period: usize,
    pub multiplier: f64,
}

pub struct AppState {
    pub klines: Mutex<HashMap<String, Vec<KlineData>>>,
    pub config: Mutex<Config>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            klines: Mutex::new(HashMap::new()),
            config: Mutex::new(Config {
                period: 20,
                multiplier: 2.0,
            }),
        }
    }

    /// 新しいローソク足データを追加・更新し、最新の指標を計算して返す
    pub fn update_and_calculate(&self, symbol: &str, new_kline: KlineData) -> Option<KlineWithIndicator> {
        let mut klines_map = self.klines.lock().unwrap();
        let klines = klines_map.entry(symbol.to_string()).or_insert_with(Vec::new);

        // データの更新ロジック
        if klines.is_empty() {
            klines.push(new_kline.clone());
        } else {
            let last_idx = klines.len() - 1;
            if klines[last_idx].time == new_kline.time {
                // 同じ時刻なら上書き (確定前の更新)
                klines[last_idx] = new_kline.clone();
            } else if klines[last_idx].time < new_kline.time {
                // 新しい時刻なら追加
                klines.push(new_kline.clone());
                // 履歴制限 (メモリ節約)
                if klines.len() > 2000 {
                    klines.remove(0);
                }
            } else {
                // 古いデータが来た場合は無視 (通常ありえない)
                return None; 
            }
        }

        // 設定の取得
        let (period, multiplier) = {
            let config = self.config.lock().unwrap();
            (config.period, config.multiplier)
        };

        // 指標計算 (全体に対して計算)
        // 最適化: 本来は直近部分だけ再計算すれば早いが、Rustなら全体計算でも十分高速
        let sma_vals = calculate_sma(klines, period);
        let (u_vals, l_vals) = calculate_bollinger_bands(klines, period, multiplier);
        let rsi_vals = calculate_rsi(klines, 14);
        let (macd_v, signal_v, hist_v) = calculate_macd(klines, 12, 26, 9);
        let (st_k, st_d) = calculate_stoch(klines, 14, 3, 3);
        
        // 最新の計算結果を抽出
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
