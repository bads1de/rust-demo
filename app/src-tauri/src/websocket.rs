use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::models::CombinedStreamPayload;
use crate::state::AppState;

/// WebSocket接続を管理し、リアルタイムデータを受信してフロントエンドに送信する非同期タスク。
///
/// Binanceの "Combined Streams" 機能を使用し、単一のWebSocket接続で
/// 複数の通貨ペアのデータを同時に受信します。
pub async fn start_websocket_listener(app_handle: AppHandle) {
    // 監視対象のシンボルリスト (Binance API仕様に合わせて小文字)
    let symbols = vec!["btcusdt", "ethusdt", "xrpusdt", "bnbusdt", "solusdt", "trxusdt", "dogeusdt", "adausdt", "bchusdt", "linkusdt"];
    
    // ストリームURLの構築
    // 形式: wss://stream.binance.com:9443/stream?streams=<symbol1>@kline_1m/<symbol2>@kline_1m/...
    let streams = symbols.iter()
        .map(|s| format!("{}@kline_1m", s))
        .collect::<Vec<_>>()
        .join("/");
    
    let url_str = format!("wss://stream.binance.com:9443/stream?streams={}", streams);
    let url = Url::parse(&url_str).unwrap();
    
    // 接続が切れた場合の自動再接続ループ
    loop {
        println!("Connecting to Binance Combined Streams...");
        
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to Binance WebSocket");
                
                // メッセージ受信ループ
                while let Some(msg) = ws_stream.next().await {
                    let Ok(msg) = msg else { continue };
                    if !msg.is_text() { continue; }
                    let Ok(text) = msg.to_text() else { continue };

                    // JSONメッセージをパース
                    if let Ok(payload) = serde_json::from_str::<CombinedStreamPayload>(text) {
                        let symbol = payload.data.symbol;
                        let new_kline = payload.data.kline;
                        
                        // アプリケーションの状態(AppState)を取得
                        let state = app_handle.state::<AppState>();
                        
                        // データの更新と指標の再計算を AppState に委譲
                        // (ロック管理や計算ロジックは state.rs に集約されている)
                        if let Some(update_data) = state.update_and_calculate(&symbol, new_kline) {
                            // 計算結果を含む最新データをフロントエンドへイベント発行
                            let _ = app_handle.emit("kline-update", update_data);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {}. Retrying in 5 seconds...", e);
                // 再接続前の待機 (サーバー負荷軽減のため必須)
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}
