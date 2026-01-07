# Rust + Tauri リアルタイムチャートアプリ 要件定義書

## 1. プロジェクト概要

**Project Name**: `Rust-Tauri-Chart` (仮)

RyzenPC のパワーを活かした Rust 製のバックエンドと、柔軟な Web フロントエンドを融合させたデスクトップアプリケーション。
「Tauri」を採用することで、Electron よりも圧倒的に軽量かつセキュアな環境を実現し、TradingView のような**スムーズで美しいリアルタイムチャート体験**を提供する。

## 2. 目標 (Goal)

- **Performance**: Rust による高速なデータ処理と WebSocket 通信。
- **Experience**: ネイティブアプリのようなレスポンスと、Web 技術によるモダンな UI。
- **Visual**: "Borderless & Dynamic" なデザイン（ダークモード、グラスモーフィズムなど）。

## 3. アーキテクチャ構成

Tauri の「Core Process (Rust)」と「Webview Process (Frontend)」の分離モデルを採用。

### A. Frontend (UI 層)

- **Framework**: React + TypeScript (Vite)
  - コンポーネント指向での開発効率を重視。
- **Styling**: **Tailwind CSS**
  - ユーティリティファーストなアプローチで、高速な UI 実装と一貫したデザインシステムを実現。- **Chart Library**: **TradingView Lightweight Charts**
  - Canvas ベースの描画で、大量のデータポイントでも高速に動作。
- **State Management**: React Context / Hooks
  - Rust からのイベントストリームを受け取り、コンポーネントへ分配。

### B. Backend (Core 層)

- **Language**: Rust
- **Framework**: Tauri (v2 推奨)
- **Async Runtime**: **Tokio**
  - 非同期 I/O のデファクトスタンダード。
- **Network**: **tokio-tungstenite** (WebSocket)
  - Binance API との常時接続を管理。
  - ※ 認証不要の `wss://stream.binance.com:9443` を使用予定。
- **Serialization**: **Serde**
  - JSON データの高速パース。

### C. データフロー

1. **Connect**: Rust 側で Binance WebSocket に接続。
2. **Stream**: 1 分足 (Kline/Candlestick) の更新情報を受信。
3. **Process**: 受信データを軽量な Struct に変換し、必要なデータのみを抽出。
4. **Emit**: Tauri の `Event System` を介して、フロントエンドへ Payload を送信。
5. **Render**: React がイベントを検知し、Lightweight Charts の `update` メソッドを叩く。

## 4. 機能要件 (MVP: Minimum Viable Product)

### 4.1 チャート機能

- **リアルタイム更新**: 最新の価格（Open, High, Low, Close）に合わせてローソク足がピョコピョコ動くアニメーション。
- **通貨ペア**: デフォルトで `BTCUSDT` を表示。
- **タイムフレーム**: 1 分足 (`1m`) を採用（動きがわかりやすいため）。

### 4.2 UI 機能

- **接続ステータス表示**: WebSocket の接続状態（接続中、切断、エラー）を右下などにインジケーター表示。
- **価格表示**: 最新価格を大きくヘッダーに表示し、前日比で色を変える（緑/赤）。
- **ウィンドウ制御**: カスタムタイトルバー（OS 標準のバーを消し、アプリ内に閉じるボタン等を統合）で没入感を出す。

## 5. 技術的制約・ルール

- **エラーハンドリング**: WebSocket 切断時は自動再接続を行うロジックを Rust 側に実装する。
- **型安全性**: TypeScript と Rust で型定義を可能な限り合わせる（`tauri-specta` などの導入も視野にいれるが、MVP では手動定義でスピーディに）。

## 6. 開発ステップ

1. **Setup**: Tauri プロジェクトの初期化 (`npm create tauri-app`)。
2. **Backend**: Rust で Binance WebSocket への接続テスト実装（ログ出力まで）。
3. **Bridge**: Rust からフロントエンドへのイベント発火 (`emit`) の実装。
4. **Frontend**: Lightweight Charts の導入と、固定データの描画確認。
5. **Integration**: ストリームデータをチャートに流し込み、リアルタイム化完成。
6. **Polish**: CSS によるデザイン調整（ダークモード、グリッド調整）。
