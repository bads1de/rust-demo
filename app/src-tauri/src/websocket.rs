use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::models::{CombinedStreamPayload, KlineWithIndicator};
use crate::state::AppState;
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};

/// WebSocket接続を管理し、データを受信してフロントエンドに送信するタスク
pub async fn start_websocket_listener(app_handle: AppHandle) {
    // 購読するシンボルリスト (小文字)
    let symbols = vec!["btcusdt", "ethusdt", "xrpusdt", "bnbusdt", "solusdt", "trxusdt", "dogeusdt", "adausdt", "bchusdt", "linkusdt"];
    
    // ストリームURLの構築 (例: .../stream?streams=btcusdt@kline_1m/ethusdt@kline_1m/...)
    let streams = symbols.iter()
        .map(|s| format!("{}@kline_1m", s))
        .collect::<Vec<_>>()
        .join("/");
    
    let url_str = format!("wss://stream.binance.com:9443/stream?streams={}", streams);
    let url = Url::parse(&url_str).unwrap();
    
    loop {
        println!("Connecting to Binance Combined Streams...");
        
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to Binance WebSocket");
                
                while let Some(msg) = ws_stream.next().await {
                    let Ok(msg) = msg else { continue };
                    if !msg.is_text() { continue; }
                    let Ok(text) = msg.to_text() else { continue };

                    // 複合ストリームのパース
                    if let Ok(payload) = serde_json::from_str::<CombinedStreamPayload>(text) {
                        let symbol = payload.data.symbol.clone();
                        let new_kline = payload.data.kline;
                        let state = app_handle.state::<AppState>();
                        
                        // 共有データをロックして更新
                        let indicators = {
                            let mut klines_map = state.klines.lock().unwrap();
                            // シンボルに対応する Vec を取得または作成
                            let klines = klines_map.entry(symbol.clone()).or_insert_with(Vec::new);
                            
                            if klines.is_empty() {
                                None
                            } else {
                                let last_idx = klines.len() - 1;
                                if klines[last_idx].time == new_kline.time {
                                    klines[last_idx] = new_kline.clone();
                                } else if klines[last_idx].time < new_kline.time {
                                    klines.push(new_kline.clone());
                                    if klines.len() > 2000 { klines.remove(0); }
                                }

                                let config = state.config.lock().unwrap();
                                let p = config.period;
                                let m = config.multiplier;

                                // 指標計算
                                let sma_vals = calculate_sma(klines, p);
                                let (u_vals, l_vals) = calculate_bollinger_bands(klines, p, m);
                                let rsi_vals = calculate_rsi(klines, 14);
                                let (macd_v, signal_v, hist_v) = calculate_macd(klines, 12, 26, 9);
                                let (st_k, st_d) = calculate_stoch(klines, 14, 3, 3);
                                
                                let i = klines.len() - 1;
                                Some((sma_vals[i], u_vals[i], l_vals[i], rsi_vals[i], macd_v[i], signal_v[i], hist_v[i], st_k[i], st_d[i]))
                            }
                        };

                        if let Some((sma, upper_band, lower_band, rsi, macd, macd_signal, macd_hist, stoch_k, stoch_d)) = indicators {
                            let update_data = KlineWithIndicator {
                                symbol: symbol.clone(),
                                kline: new_kline,
                                sma, upper_band, lower_band, rsi, macd, macd_signal, macd_hist, stoch_k, stoch_d
                            };
                            let _ = app_handle.emit("kline-update", update_data);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {}. Retrying in 5 seconds...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}
