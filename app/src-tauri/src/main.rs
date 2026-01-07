// モジュールの宣言
mod api;
mod indicators;
mod models;
mod websocket;

// 必要な機能をインポート
use api::fetch_candles;
use websocket::start_websocket_listener;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 非同期タスクとして WebSocket 接続を開始
            tauri::async_runtime::spawn(start_websocket_listener(app_handle));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![fetch_candles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}