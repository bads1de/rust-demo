use crate::models::KlineData;

/// 指定された期間の移動平均線 (SMA) を計算する
/// 
/// # Arguments
/// - `data`: 計算対象のローソク足リスト
/// - `period`: 期間 (例: 20)
pub fn calculate_sma(data: &[KlineData], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    
    // windows(period) は、配列を period 分の塊でスライドしながら取得するイテレータ
    // 例: [1,2,3,4,5].windows(3) -> [1,2,3], [2,3,4], [3,4,5]
    for (i, window) in data.windows(period).enumerate() {
        // window 内の終値 (close) を合計
        let sum: f64 = window.iter().map(|d| d.close).sum();
        // 指定期間の最後のインデックスに平均値をセット
        result[i + period - 1] = Some(sum / period as f64);
    }
    result
}

/// 指定された期間のボリンジャーバンドを計算する
/// 
/// # Arguments
/// - `data`: 計算対象のローソク足リスト
/// - `period`: 期間 (例: 20)
/// - `multiplier`: 標準偏差の倍率 (例: 2.0)
/// 
/// # Returns
/// (Upper Band, Lower Band) のタプル
pub fn calculate_bollinger_bands(data: &[KlineData], period: usize, multiplier: f64) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut upper = vec![None; data.len()];
    let mut lower = vec![None; data.len()];
    
    // まずSMAを計算
    let sma = calculate_sma(data, period);
    
    // windowsでスライドしながら標準偏差を計算
    for (i, window) in data.windows(period).enumerate() {
        // 対応するSMAの値がなければスキップ
        let Some(mean) = sma[i + period - 1] else { continue };
        
        // 分散 (Variance) の計算: Σ(x - mean)^2 / n
        let variance: f64 = window.iter()
            .map(|d| (d.close - mean).powi(2))
            .sum::<f64>() / period as f64;
            
        let std_dev = variance.sqrt();
        
        upper[i + period - 1] = Some(mean + (std_dev * multiplier));
        lower[i + period - 1] = Some(mean - (std_dev * multiplier));
    }
    
    (upper, lower)
}

