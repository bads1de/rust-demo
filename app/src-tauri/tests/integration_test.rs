use rust_chart_lib::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};
use rust_chart_lib::models::KlineData;
use rust_chart_lib::api::get_all_symbols;

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

#[tokio::test]
async fn test_get_all_symbols_integration() {
    let symbols = get_all_symbols().await;
    assert!(symbols.is_ok(), "Should fetch symbols from Binance");
    
    let list = symbols.unwrap();
    assert!(!list.is_empty(), "Symbol list should not be empty");
    
    // 主要な銘柄が含まれているか確認
    assert!(list.contains(&"BTCUSDT".to_string()));
    
    // フィルタリング（USDTペアのみ）が効いているか確認
    for s in &list {
        assert!(s.ends_with("USDT"), "All symbols should be USDT pairs");
    }
}

#[test]
fn test_calculate_sma_integration() {
    let data = vec![k(10.0), k(20.0), k(30.0), k(40.0), k(50.0)];
    let result = calculate_sma(&data, 3);
    assert_eq!(result[2], Some(20.0));
    assert_eq!(result[3], Some(30.0));
}

#[test]
fn test_calculate_bollinger_bands_integration() {
    let data = vec![k(10.0), k(20.0), k(30.0), k(40.0), k(50.0)];
    let (upper, lower) = calculate_bollinger_bands(&data, 3, 2.0);
    assert!(upper[2].is_some());
    assert!(lower[2].is_some());
}

#[test]
fn test_calculate_rsi_integration() {
    let mut data = Vec::new();
    for i in 0..20 {
        data.push(k(100.0 + i as f64));
    }
    let rsi = calculate_rsi(&data, 14);
    assert!(rsi[14].is_some());
}

#[test]
fn test_calculate_macd_integration() {
    let mut data = Vec::new();
    for i in 0..50 {
        data.push(k(100.0 + (i as f64 * 2.0))); 
    }
    let (macd, signal, hist) = calculate_macd(&data, 12, 26, 9);
    assert!(macd[40].is_some());
    assert!(signal[40].is_some());
    assert!(hist[40].is_some());
}

#[test]
fn test_calculate_stoch_integration() {
    let mut data = Vec::new();
    for i in 0..50 {
        let price = 100.0 + (i as f64).sin() * 10.0;
        data.push(k(price));
    }
    let (k_line, d_line) = calculate_stoch(&data, 14, 3, 3);
    assert!(k_line[40].is_some());
    assert!(d_line[40].is_some());
}
