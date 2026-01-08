use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::{KlineData, KlineWithIndicator};
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};

/// アプリケーションの設定情報を保持する構造体
///
/// 現在の計算に使用する期間や倍率などを保持します。
pub struct Config {
    /// 移動平均線などの計算期間 (デフォルト: 20)
    pub period: usize,
    /// ボリンジャーバンドの標準偏差倍率 (デフォルト: 2.0)
    pub multiplier: f64,
}

/// アプリケーション全体で共有する状態 (State)
///
/// Tauriの `manage` 機能によってメモリ上に保持され、複数のスレッド（WebSocket受信スレッド、メインスレッド）から
/// 安全にアクセスできるように `Mutex` で保護されています。
pub struct AppState {
    /// シンボルごとのローソク足データを保持する HashMap。
    /// キー: シンボル名 (例: "BTCUSDT")
    /// 値: ローソク足データのリスト (時系列順)
    pub klines: Mutex<HashMap<String, Vec<KlineData>>>,
    
    /// 現在の計算設定
    pub config: Mutex<Config>,
}

impl AppState {
    /// 新しい AppState を初期化します。
    pub fn new() -> Self {
        Self {
            klines: Mutex::new(HashMap::new()),
            config: Mutex::new(Config {
                period: 20,
                multiplier: 2.0,
            }),
        }
    }

    /// 新しいローソク足データを受け取り、状態を更新し、最新のテクニカル指標を計算して返します。
    ///
    /// このメソッドはスレッドセーフであり、データのロック取得、更新、計算、ロック解除を一括で行います。
    ///
    /// # Arguments
    /// * `symbol` - 通貨ペアのシンボル名（例: "btcusdt"）。
    /// * `new_kline` - WebSocketなどで受信した最新のローソク足データ。
    ///
    /// # Returns
    /// * `Option<KlineWithIndicator>` - 更新後の最新データと指標を含む構造体。
    ///   データが古い場合など、更新が不要な場合は `None` を返します。
    ///
    /// # Logic
    /// 1. `klines` のロックを取得します。
    /// 2. 該当シンボルのデータリストを取得（なければ作成）します。
    /// 3. データのタイムスタンプを確認し、追加するか上書きするか判断します。
    ///    - 同じ時刻: 確定前の更新データなので上書き。
    ///    - 新しい時刻: 新しい足として追加（履歴が2000件を超えたら古いものを削除）。
    /// 4. 設定（`period`, `multiplier`）を取得します。
    /// 5. 全データに対してテクニカル指標を再計算します。
    /// 6. 最新の1件だけを `KlineWithIndicator` に詰めて返します。
    pub fn update_and_calculate(&self, symbol: &str, new_kline: KlineData) -> Option<KlineWithIndicator> {
        // Mutexロックを取得。失敗した場合はパニックさせる（通常は起きない）
        let mut klines_map = self.klines.lock().unwrap();
        
        // HashMapからシンボルに対応するベクタを取得、なければ新規作成
        let klines = klines_map.entry(symbol.to_string()).or_insert_with(Vec::new);

        // --- データの更新ロジック ---
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
                // 履歴制限 (メモリ節約と計算負荷軽減のため2000件に制限)
                if klines.len() > 2000 {
                    klines.remove(0);
                }
            } else {
                // 受信したデータが保持している最新データより古い場合（遅延などで順序が逆転した場合）は無視
                return None; 
            }
        }

        // --- 設定の取得 ---
        // klinesのロックを持ったままconfigのロックを取っても、デッドロックの順序さえ守れば安全だが、
        // ここでは念のためスコープを分けてロック時間を最小限にする
        let (period, multiplier) = {
            let config = self.config.lock().unwrap();
            (config.period, config.multiplier)
        };

        // --- 指標計算 ---
        // Rustの計算速度が非常に高速なため、データ更新のたびに全件再計算してもパフォーマンスに影響はない。
        // (10銘柄 x 2000件程度ならマイクロ秒オーダーで完了する)
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