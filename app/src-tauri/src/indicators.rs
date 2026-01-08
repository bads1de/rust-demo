use crate::models::KlineData;
use ta::indicators::{SimpleMovingAverage, BollingerBands, RelativeStrengthIndex, MovingAverageConvergenceDivergence, FastStochastic};
use ta::{Next, DataItem};

/// 指定された期間の移動平均線 (SMA) を計算する
pub fn calculate_sma(data: &[KlineData], period: usize) -> Vec<Option<f64>> {
    let mut sma = SimpleMovingAverage::new(period).unwrap();
    let mut result = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let value = sma.next(item.close);
        if i < period - 1 {
            result.push(None);
        } else {
            result.push(Some(value));
        }
    }
    result
}

/// 指定された期間のボリンジャーバンドを計算する
pub fn calculate_bollinger_bands(data: &[KlineData], period: usize, multiplier: f64) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut bb = BollingerBands::new(period, multiplier).unwrap();
    let mut upper = Vec::with_capacity(data.len());
    let mut lower = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let output = bb.next(item.close);
        if i < period - 1 {
            upper.push(None);
            lower.push(None);
        } else {
            upper.push(Some(output.upper));
            lower.push(Some(output.lower));
        }
    }
    (upper, lower)
}

/// RSI (Relative Strength Index) を計算する
pub fn calculate_rsi(data: &[KlineData], period: usize) -> Vec<Option<f64>> {
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut result = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let value = rsi.next(item.close);
        if i < period {
            result.push(None);
        } else {
            result.push(Some(value));
        }
    }
    result
}

/// MACD を計算する
pub fn calculate_macd(data: &[KlineData], fast_period: usize, slow_period: usize, signal_period: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut macd = MovingAverageConvergenceDivergence::new(fast_period, slow_period, signal_period).unwrap();
    let mut macd_line = Vec::with_capacity(data.len());
    let mut signal_line = Vec::with_capacity(data.len());
    let mut hist_line = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let output = macd.next(item.close);
        
        if i < slow_period - 1 {
            macd_line.push(None);
            signal_line.push(None);
            hist_line.push(None);
        } else {
            macd_line.push(Some(output.macd));
            signal_line.push(Some(output.signal));
            hist_line.push(Some(output.histogram));
        }
    }
    (macd_line, signal_line, hist_line)
}

/// ストキャスティクス (Slow Stochastic) を計算する
/// 
/// 定義:
/// Fast %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// Slow %K = SMA(Fast %K, smooth_k)
/// Slow %D = SMA(Slow %K, smooth_d)
pub fn calculate_stoch(data: &[KlineData], period: usize, smooth_k: usize, smooth_d: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    // Fast Stochastic (Raw %K)
    let mut stoch = FastStochastic::new(period).unwrap();
    // Smoothers
    let mut sma_k = SimpleMovingAverage::new(smooth_k).unwrap();
    let mut sma_d = SimpleMovingAverage::new(smooth_d).unwrap();

    let mut k_line = Vec::with_capacity(data.len());
    let mut d_line = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let data_item = DataItem::builder()
            .high(item.high)
            .low(item.low)
            .close(item.close)
            .open(item.open)
            .volume(item.volume)
            .build()
            .unwrap();

        // 1. Fast %K を計算 (戻り値は0-100)
        let fast_k = stoch.next(&data_item);
        
        // 2. Slow %K を計算 (Fast %K を smooth_k で平滑化)
        let slow_k = sma_k.next(fast_k);
        
        // 3. Slow %D を計算 (Slow %K を smooth_d で平滑化)
        let slow_d = sma_d.next(slow_k);

        if i < period {
            k_line.push(None);
            d_line.push(None);
        } else {
            k_line.push(Some(slow_k));
            d_line.push(Some(slow_d));
        }
    }
    (k_line, d_line)
}
