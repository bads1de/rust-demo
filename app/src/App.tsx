import { TradingChart } from "./components/TradingChart";
import { TitleBar } from "./components/TitleBar";

const SYMBOLS = [
  "BTCUSDT", "ETHUSDT", "XRPUSDT", "BNBUSDT", "SOLUSDT",
  "TRXUSDT", "DOGEUSDT", "ADAUSDT", "BCHUSDT", "LINKUSDT"
];

function App() {
  return (
    <div className="min-h-screen bg-[#0B0E14] text-gray-300 font-sans selection:bg-indigo-500/30">
      <TitleBar />
      
      {/* Main Content Area - Added top padding for TitleBar */}
      <div className="pt-14 px-6 pb-8 max-w-[1920px] mx-auto">
        
        {/* Dashboard Header */}
        <header className="mb-8 flex flex-col md:flex-row md:items-end justify-between gap-6">
          <div>
            <h1 className="text-3xl font-bold text-white tracking-tight flex items-center gap-3">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-indigo-500"></span>
              </span>
              Market Overview
            </h1>
            <p className="text-sm text-gray-500 mt-1 font-medium">
              Real-time Analysis • <span className="text-indigo-400">Rust Compute Engine</span>
            </p>
          </div>
          
          <div className="flex items-center gap-6 text-xs font-mono text-gray-500 bg-[#111620] px-4 py-2 rounded-lg border border-white/5">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
              <span>WS: CONNECTED</span>
            </div>
            <div className="w-px h-3 bg-gray-700"></div>
            <div className="flex items-center gap-2">
              <span>LATENCY: &lt;1ms</span>
            </div>
          </div>
        </header>

        {/* Main Grid */}
        <main className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">
          {SYMBOLS.map((symbol) => (
            <div key={symbol} className="h-[420px]">
              <TradingChart symbol={symbol} />
            </div>
          ))}
        </main>

        {/* Footer Info */}
        <footer className="mt-12 text-center">
          <p className="text-[10px] font-medium text-gray-600 uppercase tracking-widest hover:text-indigo-500 transition-colors cursor-default">
            Powered by Tauri v2 • Rust • React
          </p>
        </footer>
      </div>
    </div>
  );
}

export default App;
