use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::models::CombinedStreamPayload;
use crate::state::AppState;
use crate::api::get_all_symbols;

/// 全銘柄の監視を開始するメインタスク
pub async fn start_websocket_listener(app_handle: AppHandle) {
    // 1. まず全銘柄リストを取得
    let all_symbols = match get_all_symbols().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to fetch symbols: {}", e);
            // 取得に失敗した場合は従来の10銘柄でフォールバック
            vec!["BTCUSDT", "ETHUSDT", "XRPUSDT", "BNBUSDT", "SOLUSDT", "TRXUSDT", "DOGEUSDT", "ADAUSDT", "BCHUSDT", "LINKUSDT"]
                .iter().map(|s| s.to_string()).collect()
        }
    };

    println!("Starting monitoring for {} symbols...", all_symbols.len());

    // 2. Binanceの上限（1024ストリーム）に合わせて、銘柄をチャンクに分割
    // 今回は安全のため、1つの接続あたり200銘柄程度に分ける
    let chunks = all_symbols.chunks(200);

    for chunk in chunks {
        let chunk_vec: Vec<String> = chunk.iter().map(|s| s.to_lowercase()).collect();
        let app_clone = app_handle.clone();
        
        // チャンクごとに独立したリスナースレッドを起動
        tauri::async_runtime::spawn(async move {
            listen_to_stream_chunk(app_clone, chunk_vec).await;
        });
    }
}

/// 特定の銘柄グループを監視する個別のリスナータスク
async fn listen_to_stream_chunk(app_handle: AppHandle, symbols: Vec<String>) {
    let streams = symbols.iter()
        .map(|s| format!("{}@kline_1m", s))
        .collect::<Vec<_>>()
        .join("/");
    
    let url_str = format!("wss://stream.binance.com:9443/stream?streams={}", streams);
    let url = Url::parse(&url_str).unwrap();
    
    loop {
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("Sub-stream connected for {} symbols", symbols.len());
                
                while let Some(msg) = ws_stream.next().await {
                    let Ok(msg) = msg else { continue };
                    if !msg.is_text() { continue; }
                    let Ok(text) = msg.to_text() else { continue };

                    if let Ok(payload) = serde_json::from_str::<CombinedStreamPayload>(text) {
                        let symbol = payload.data.symbol;
                        let new_kline = payload.data.kline;
                        let state = app_handle.state::<AppState>();
                        
                        // DashMap版の高速更新
                        if let Some(update_data) = state.update_and_calculate(&symbol, new_kline) {
                            // ※注意: 全銘柄の全メッセージをフロントに送ると通信がパンクするため、
                            // 本来はここでフィルタリング（RSI < 30など）を行う。
                            // 今回はデモとして全て送信する。
                            let _ = app_handle.emit("kline-update", update_data);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Sub-stream error: {}. Retrying...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}