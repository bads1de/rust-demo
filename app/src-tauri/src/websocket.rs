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
                        let symbol = payload.data.symbol;
                        let new_kline = payload.data.kline;
                        let state = app_handle.state::<AppState>();
                        
                        // AppStateのメソッドに委譲
                        if let Some(update_data) = state.update_and_calculate(&symbol, new_kline) {
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
