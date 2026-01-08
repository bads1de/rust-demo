import { TradingChart } from "./components/TradingChart";

const SYMBOLS = [
  "BTCUSDT", "ETHUSDT", "XRPUSDT", "BNBUSDT", "SOLUSDT",
  "TRXUSDT", "DOGEUSDT", "ADAUSDT", "BCHUSDT", "LINKUSDT"
];

function App() {
  return (
    <div className="min-h-screen bg-slate-950 p-4">
      {/* Dashboard Header */}
      <header className="mb-6 flex flex-col md:flex-row md:items-center justify-between gap-4 bg-slate-900/40 p-6 rounded-2xl border border-slate-800 shadow-2xl backdrop-blur-md">
        <div>
          <h1 className="text-2xl font-black text-white tracking-tighter flex items-center gap-3">
            <span className="p-2 bg-blue-600 rounded-lg shadow-[0_0_20px_rgba(37,99,235,0.4)]">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>
            </span>
            RUST-CHART <span className="text-blue-500">PRO</span>
          </h1>
          <p className="text-slate-500 text-xs font-bold mt-1 uppercase tracking-widest">
            High-Performance Real-time Multi-Market Terminal
          </p>
        </div>
        
        <div className="flex items-center gap-6">
          <div className="text-right hidden sm:block">
            <div className="text-[10px] font-black text-slate-500 uppercase">Compute Engine</div>
            <div className="text-sm font-mono text-emerald-500 flex items-center gap-2 justify-end">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
              RUST TA-LIB ACTIVE
            </div>
          </div>
          <div className="h-10 w-px bg-slate-800"></div>
          <div className="text-right">
            <div className="text-[10px] font-black text-slate-500 uppercase">Active Streams</div>
            <div className="text-sm font-mono text-blue-400">10 PAIRS / 1m</div>
          </div>
        </div>
      </header>

      {/* Main Grid */}
      <main className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        {SYMBOLS.map((symbol) => (
          <div key={symbol} className="h-[500px]">
            <TradingChart symbol={symbol} />
          </div>
        ))}
      </main>

      {/* Footer Info */}
      <footer className="mt-8 text-center text-[10px] font-mono text-slate-600 uppercase tracking-[0.2em] pb-8">
        Powered by Tauri v2 + Rust + React • Data from Binance WebSocket API
      </footer>
    </div>
  );
}

export default App;