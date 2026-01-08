# Rust-Chart Pro 🦀📈

Rust の高い計算性能と Tauri v2 を活用した、リアルタイム暗号通貨監視ダッシュボードです。  
10 種類の主要通貨ペアに対して、5 種類以上のテクニカル指標を爆速でリアルタイム計算し、同期されたマルチチャートで表示します。

## ✨ 主な機能

- **マルチ銘柄同時監視**: 主要 10 銘柄（BTC, ETH, SOL 等）のデータを並列で取得・描画。
- **高精度テクニカル計算 (Rust)**:
  - SMA (単純移動平均線)
  - Bollinger Bands (ボリンジャーバンド)
  - RSI (相対力指数)
  - MACD (移動平均収束拡散手法)
  - Slow Stochastic (ストキャスティクス)
- **リアルタイム同期ダッシュボード**:
  - 全ての銘柄・インジケーターが時間軸で同期。
  - 各インジケーターの表示/非表示を瞬時に切り替え可能。
  - 2000 件の履歴データをメモリ上に保持し、ミリ秒単位で再計算。
- **モダン・ネイティブ UI**:
  - カスタムタイトルバーによるスリムなデザイン。
  - ウィンドウの最大化・最小化・ドラッグ移動に対応。
  - ダークテーマに最適化されたスタイリッシュな外観。

## 🛠 技術スタック

### Backend (Rust / Tauri v2)

- **Rust**: 並行処理と高速演算を担当。
- **ta-lib (Rust)**: 信頼性の高いテクニカル指標計算。
- **Tokio / WebSocket**: Binance API との低遅延ストリーミング。
- **Serde**: 高速な JSON シリアライズ/デシリアライズ。

### Frontend (React / TypeScript)

- **Lightweight Charts**: TradingView 提供の高性能 Canvas チャート。
- **Tailwind CSS**: 柔軟で美しいスタイリング。
- **Lucide React**: 美しいアイコンセット。

## 🚀 始め方

### 必須条件

- Rust (最新の安定版)
- Node.js (v18 以上)
- Windows / macOS / Linux

### セットアップ

```bash
# 依存関係のインストール
cd app
npm install

# 開発モードで起動
npm run tauri dev
```

## 🧪 テスト

バックエンドの計算ロジックと状態管理は、TDD（テスト駆動開発）に基づいて徹底的に検証されています。

```bash
cd app/src-tauri
cargo test
```

---

Powered by Rust 🦀 and passion for performance.
