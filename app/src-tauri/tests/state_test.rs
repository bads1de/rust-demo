use rust_chart_lib::state::AppState;
use rust_chart_lib::models::KlineData;

// ヘルパー
fn create_kline(time: i64, close: f64) -> KlineData {
    KlineData {
        time,
        open: close,
        high: close + 10.0,
        low: close - 10.0,
        close,
        volume: 100.0,
    }
}

#[test]
fn test_state_update_logic() {
    let state = AppState::new();
    let symbol = "BTCUSDT";

    // 1. 新規追加
    let k1 = create_kline(1000, 50000.0);
    let res1 = state.update_and_calculate(symbol, k1.clone());
    assert!(res1.is_some());
    assert_eq!(res1.unwrap().kline.close, 50000.0);

    {
        let map = state.klines.lock().unwrap();
        assert_eq!(map.get(symbol).unwrap().len(), 1);
    }

    // 2. 同じ時刻の更新 (価格変動)
    let k1_update = create_kline(1000, 50100.0);
    let res2 = state.update_and_calculate(symbol, k1_update.clone());
    assert!(res2.is_some());
    assert_eq!(res2.unwrap().kline.close, 50100.0);

    {
        let map = state.klines.lock().unwrap();
        let list = map.get(symbol).unwrap();
        assert_eq!(list.len(), 1); // 増えていないこと
        assert_eq!(list[0].close, 50100.0); // 更新されていること
    }

    // 3. 次の時刻の追加
    let k2 = create_kline(2000, 50200.0);
    let res3 = state.update_and_calculate(symbol, k2.clone());
    assert!(res3.is_some());

    {
        let map = state.klines.lock().unwrap();
        assert_eq!(map.get(symbol).unwrap().len(), 2);
    }
}

#[test]
fn test_state_history_limit() {
    let state = AppState::new();
    let symbol = "ETHUSDT";

    // 2005個のデータを追加
    for i in 0..2005 {
        let k = create_kline(i * 1000, 100.0 + i as f64);
        state.update_and_calculate(symbol, k);
    }

    let map = state.klines.lock().unwrap();
    let list = map.get(symbol).unwrap();
    
    // 2000個に制限されているか
    assert_eq!(list.len(), 2000);
    
    // 最初のデータ (index 0) は time=5000 (i=5) であるはず (0,1,2,3,4 は削除された)
    assert_eq!(list[0].time, 5000);
    // 最後のデータ
    assert_eq!(list.last().unwrap().time, 2004000);
}
