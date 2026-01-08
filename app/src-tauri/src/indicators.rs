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

