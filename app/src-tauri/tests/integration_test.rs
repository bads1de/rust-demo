use rust_chart_lib::indicators::{calculate_sma, calculate_bollinger_bands};
use rust_chart_lib::models::KlineData;

// ヘルパー: テスト用のダミーデータを作成
fn create_dummy_kline(close: f64) -> KlineData {
    KlineData {
        time: 0,
        open: 0.0,
        high: 0.0,
        low: 0.0,
        close,
        volume: 0.0,
    }
}

#[test]
fn test_calculate_sma_integration() {
    let data = vec![
        create_dummy_kline(10.0),
        create_dummy_kline(20.0),
        create_dummy_kline(30.0),
        create_dummy_kline(40.0),
        create_dummy_kline(50.0),
    ];

    // 期間3のSMAを計算
    let result = calculate_sma(&data, 3);

    // インデックス0, 1 は計算できないので None
    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    
    // インデックス2: (10 + 20 + 30) / 3 = 20.0
    assert_eq!(result[2], Some(20.0));
    
    // インデックス3: (20 + 30 + 40) / 3 = 30.0
    assert_eq!(result[3], Some(30.0));
    
    // インデックス4: (30 + 40 + 50) / 3 = 40.0
    assert_eq!(result[4], Some(40.0));
}

#[test]
fn test_calculate_sma_insufficient_data() {
    let data = vec![create_dummy_kline(10.0)];
    // データが期間より短い場合
    let result = calculate_sma(&data, 3);
    assert_eq!(result[0], None);
}

#[test]
fn test_calculate_bollinger_bands_integration() {
    let data = vec![
        create_dummy_kline(10.0),
        create_dummy_kline(20.0),
        create_dummy_kline(30.0),
        create_dummy_kline(40.0),
        create_dummy_kline(50.0),
    ];

    // 期間3、倍率2.0で計算
    let (upper, lower) = calculate_bollinger_bands(&data, 3, 2.0);

    // インデックス2の検証 (値: 10, 20, 30)
    // 平均(SMA) = 20.0
    // 分散 = ((10-20)^2 + (20-20)^2 + (30-20)^2) / 3 = (100 + 0 + 100) / 3 = 200/3 ≈ 66.66...
    // 標準偏差 = sqrt(66.66...) ≈ 8.165
    // Upper = 20 + (8.165 * 2) ≈ 36.33
    // Lower = 20 - (8.165 * 2) ≈ 3.67

    assert!(upper[2].is_some());
    assert!(lower[2].is_some());
    
    let u = upper[2].unwrap();
    let l = lower[2].unwrap();
    
    // 計算誤差を考慮して範囲でチェック
    assert!((u - 36.33).abs() < 0.1);
    assert!((l - 3.67).abs() < 0.1);

    // 上部バンド > SMA > 下部バンド の関係が成り立っているか
    assert!(u > 20.0);
    assert!(l < 20.0);
}
