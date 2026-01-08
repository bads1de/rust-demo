use rust_chart_lib::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use rust_chart_lib::models::KlineData;

// ヘルパー: テスト用のダミーデータを作成
fn create_dummy_kline(close: f64, high: f64, low: f64) -> KlineData {
    KlineData {
        time: 0,
        open: close, // Open = Close に設定して矛盾を防ぐ
        high, 
        low, 
        close,
        volume: 0.0,
    }
}

// 簡易ヘルパー
fn k(close: f64) -> KlineData {
    create_dummy_kline(close, close + 5.0, close - 5.0)
}

#[test]
fn test_calculate_sma_integration() {
    let data = vec![k(10.0), k(20.0), k(30.0), k(40.0), k(50.0)];
    let result = calculate_sma(&data, 3);
    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    assert_eq!(result[2], Some(20.0));
    assert_eq!(result[3], Some(30.0));
    assert_eq!(result[4], Some(40.0));
}

#[test]
fn test_calculate_bollinger_bands_integration() {
    let data = vec![k(10.0), k(20.0), k(30.0), k(40.0), k(50.0)];
    let (upper, lower) = calculate_bollinger_bands(&data, 3, 2.0);
    assert!(upper[2].is_some());
    assert!(lower[2].is_some());
    // 誤差許容
    let u = upper[2].unwrap();
    let l = lower[2].unwrap();
    assert!(u > 20.0);
    assert!(l < 20.0);
}

#[test]
fn test_calculate_rsi_integration() {
    // 15個のデータを作成 (RSI 14は15個目で値が出る)
    let mut data = Vec::new();
    for i in 0..20 {
        data.push(k(100.0 + i as f64)); // 上昇トレンド
    }
    let rsi = calculate_rsi(&data, 14);
    assert!(rsi[13].is_none()); // 14個目 (index 13) はまだNone
    assert!(rsi[14].is_some()); 
    // 上昇し続けているのでRSIは高いはず
    assert!(rsi[14].unwrap() > 50.0);
}

#[test]
fn test_calculate_macd_integration() {
    let mut data = Vec::new();
    for i in 0..50 {
        data.push(k(100.0 + (i as f64 * 2.0))); 
    }
    // MACD (12, 26, 9)
    let (macd, signal, hist) = calculate_macd(&data, 12, 26, 9);
    
    // Stub実装なので今はNoneが返ってくる -> テストは失敗するはず (Red)
    assert!(macd[40].is_some(), "MACD should be calculated");
    assert!(signal[40].is_some(), "Signal should be calculated");
    assert!(hist[40].is_some(), "Histogram should be calculated");
}

#[test]
fn test_calculate_stoch_integration() {
    let mut data = Vec::new();
    for i in 0..50 {
        // アップダウントレンドを作る
        let price = 100.0 + (i as f64).sin() * 10.0;
        data.push(k(price));
    }
    // Stoch (14, 1, 3)
    let (k_line, d_line) = calculate_stoch(&data, 14, 1, 3);
    
    // Stub実装なので失敗するはず (Red)
    assert!(k_line[40].is_some(), "Stoch %K should be calculated");
    assert!(d_line[40].is_some(), "Stoch %D should be calculated");
}