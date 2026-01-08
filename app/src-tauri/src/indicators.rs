use crate::models::KlineData;
use ta::indicators::{SimpleMovingAverage, BollingerBands, RelativeStrengthIndex, MovingAverageConvergenceDivergence, FastStochastic};
use ta::{Next, DataItem};

/// 指定された期間の単純移動平均線 (SMA) を計算します。
///
/// # Arguments
/// - `data`: 計算対象のローソク足リスト
/// - `period`: 期間 (例: 20)
pub fn calculate_sma(data: &[KlineData], period: usize) -> Vec<Option<f64>> {
    let mut sma = SimpleMovingAverage::new(period).unwrap();
    let mut result = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let value = sma.next(item.close);
        // ライブラリはデータ不足時でも値を返そうとする場合があるため、
        // 期間に満たない最初のデータは明示的に None とする（チャート表示の都合上）
        if i < period - 1 {
            result.push(None);
        } else {
            result.push(Some(value));
        }
    }
    result
}

/// 指定された期間と倍率でボリンジャーバンドを計算します。
///
/// # Returns
/// (Upper Band, Lower Band) のタプル
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

/// RSI (Relative Strength Index) を計算します。
pub fn calculate_rsi(data: &[KlineData], period: usize) -> Vec<Option<f64>> {
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut result = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let value = rsi.next(item.close);
        // RSIは最初の `period` 個のデータでは計算できない
        if i < period {
            result.push(None);
        } else {
            result.push(Some(value));
        }
    }
    result
}

/// MACD (Moving Average Convergence Divergence) を計算します。
/// 
/// # Returns
/// (MACD Line, Signal Line, Histogram) のタプル
pub fn calculate_macd(data: &[KlineData], fast_period: usize, slow_period: usize, signal_period: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut macd = MovingAverageConvergenceDivergence::new(fast_period, slow_period, signal_period).unwrap();
    let mut macd_line = Vec::with_capacity(data.len());
    let mut signal_line = Vec::with_capacity(data.len());
    let mut hist_line = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        let output = macd.next(item.close);
        
        // 長期EMAが計算できるまではMACDも安定しないため None とする
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

/// ストキャスティクス (Slow Stochastic) を計算します。
/// 
/// `ta` クレートには `SlowStochastic` 構造体もありますが、
/// 計算の柔軟性を確保するため、ここでは `FastStochastic` (Raw %K) を元に
/// 手動で平滑化を行うことで Slow %K, Slow %D を算出しています。
/// 
/// 定義:
/// 1. Fast %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// 2. Slow %K = SMA(Fast %K, smooth_k)
/// 3. Slow %D = SMA(Slow %K, smooth_d)
pub fn calculate_stoch(data: &[KlineData], period: usize, smooth_k: usize, smooth_d: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    // Fast Stochastic 計算機
    let mut stoch = FastStochastic::new(period).unwrap();
    // 平滑化用のSMA計算機
    let mut sma_k = SimpleMovingAverage::new(smooth_k).unwrap();
    let mut sma_d = SimpleMovingAverage::new(smooth_d).unwrap();

    let mut k_line = Vec::with_capacity(data.len());
    let mut d_line = Vec::with_capacity(data.len());

    for (i, item) in data.iter().enumerate() {
        // ストキャスティクスは高値・安値・終値すべてを使用する
        let data_item = DataItem::builder()
            .high(item.high)
            .low(item.low)
            .close(item.close)
            .open(item.open)
            .volume(item.volume)
            .build()
            .unwrap();

        // 1. Fast %K
        let fast_k = stoch.next(&data_item);
        
        // 2. Slow %K
        let slow_k = sma_k.next(fast_k);
        
        // 3. Slow %D
        let slow_d = sma_d.next(slow_k);

        // データ不足時は None
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