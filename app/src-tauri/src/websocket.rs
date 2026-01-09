use tauri::{AppHandle, Emitter, Manager};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use std::sync::Arc;
use crate::state::AppState;
use crate::exchanges::Exchange;
use crate::api::get_exchange_impl;

/// 指定された取引所の全銘柄監視を開始するメインタスク
pub async fn start_websocket_listener(app_handle: AppHandle, exchange_name: String) {
    let exchange: Arc<Box<dyn Exchange>> = Arc::new(get_exchange_impl(&exchange_name));

    // 1. 全銘柄リストを取得
    let all_symbols = match exchange.get_symbols().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to fetch symbols for {}: {}", exchange_name, e);
            return;
        }
    };

    println!("Starting {} monitoring for {} symbols...", exchange_name, all_symbols.len());

    // 2. 取引所ごとに制限があるためチャンク分割（とりあえず200）
    let chunk_size = 200;
    let chunks = all_symbols.chunks(chunk_size);

    for chunk in chunks {
        let chunk_vec = chunk.to_vec();
        let app_clone = app_handle.clone();
        let exchange_clone = exchange.clone();
        
        // チャンクごとに独立したリスナースレッドを起動
        tauri::async_runtime::spawn(async move {
            listen_to_stream_chunk(app_clone, exchange_clone, chunk_vec).await;
        });
    }
}

/// 特定の銘柄グループを監視する個別のリスナータスク
async fn listen_to_stream_chunk(app_handle: AppHandle, exchange: Arc<Box<dyn Exchange>>, symbols: Vec<String>) {
    let url_str = exchange.websocket_url();
    let url = Url::parse(url_str).expect("Invalid WebSocket URL");
    
    loop {
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("[{}] Connected for {} symbols", exchange.id(), symbols.len());
                
                // 接続直後に購読メッセージを送信
                if let Ok(payload) = exchange.websocket_subscription_payload(&symbols) {
                    if !payload.is_empty() {
                        if let Err(e) = ws_stream.send(Message::Text(payload.into())).await {
                            eprintln!("[{}] Subscription failed: {}", exchange.id(), e);
                            continue;
                        }
                    }
                }

                while let Some(msg) = ws_stream.next().await {
                    let Ok(msg) = msg else { continue };
                    
                    match msg {
                        Message::Text(text) => {
                            // メッセージ解析（Exchangeトレイトに委譲）
                            match exchange.parse_websocket_message(&text) {
                                Ok(Some((symbol, new_kline))) => {
                                    let state = app_handle.state::<AppState>();
                                    // DashMap版の高速更新
                                    if let Some(update_data) = state.update_and_calculate(&symbol, new_kline) {
                                        let _ = app_handle.emit("kline-update", update_data);
                                    }
                                }
                                Ok(None) => {} // Heartbeat or ignore
                                Err(e) => {
                                    // eprintln!("Parse error: {}", e);
                                }
                            }
                        },
                        Message::Ping(data) => {
                            let _ = ws_stream.send(Message::Pong(data)).await;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("[{}] Connection error: {}. Retrying...", exchange.id(), e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}