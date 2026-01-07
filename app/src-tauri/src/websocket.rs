use tauri::{AppHandle, Emitter};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::models::KlineMessage;

/// WebSocket接続を管理し、データを受信してフロントエンドに送信するタスク
pub async fn start_websocket_listener(app_handle: AppHandle) {
    let url = Url::parse("wss://stream.binance.com:9443/ws/btcusdt@kline_1m").unwrap();
    
    // 接続が切れた場合に再接続するための無限ループ
    loop {
        println!("Connecting to Binance WebSocket...");
        
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to Binance WebSocket");
                
                // メッセージ受信ループ
                while let Some(msg) = ws_stream.next().await {
                    // エラーメッセージはスキップ
                    let Ok(msg) = msg else { continue };
                    
                    // テキストメッセージ以外はスキップ
                    if !msg.is_text() { continue; }
                    
                    // テキストの取り出し
                    let Ok(text) = msg.to_text() else { continue };

                    // JSONパース
                    match serde_json::from_str::<KlineMessage>(text) {
                        Ok(parsed) => {
                            // フロントエンドにイベントを発行
                            let _ = app_handle.emit("kline-update", parsed.kline);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {}. Retrying in 5 seconds...", e);
            }
        }
        
        // 再接続前の待機
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
