import { TradingChart } from "./components/TradingChart";
import {
  Activity,
  LayoutDashboard,
  BarChart3,
  Settings,
  Bell,
} from "lucide-react";

function App() {
  return (
    <div className="min-h-screen bg-[#020617] text-slate-300 flex select-none">
      {/* Custom Title Bar for Dragging */}
      <div
        data-tauri-drag-region
        className="fixed top-0 left-0 right-0 h-8 z-50 flex justify-end items-center px-4 bg-transparent"
      >
        <div className="flex gap-2">
          <div className="w-3 h-3 rounded-full bg-slate-700 hover:bg-amber-500 transition-colors cursor-pointer"></div>
          <div className="w-3 h-3 rounded-full bg-slate-700 hover:bg-emerald-500 transition-colors cursor-pointer"></div>
          <div
            className="w-3 h-3 rounded-full bg-slate-700 hover:bg-rose-500 transition-colors cursor-pointer"
            onClick={() => window.close()}
          ></div>
        </div>
      </div>

      {/* Sidebar */}
      <aside className="w-16 border-r border-slate-800 flex flex-col items-center py-6 gap-8 bg-[#020617]/80">
        <div className="w-10 h-10 bg-indigo-600 rounded-xl flex items-center justify-center text-white font-bold shadow-lg shadow-indigo-500/20">
          R
        </div>
        <nav className="flex flex-col gap-6">
          <button className="p-2 text-indigo-400 hover:bg-slate-800 rounded-lg transition-colors">
            <LayoutDashboard className="w-6 h-6" />
          </button>
          <button className="p-2 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors">
            <BarChart3 className="w-6 h-6" />
          </button>
          <button className="p-2 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors">
            <Activity className="w-6 h-6" />
          </button>
        </nav>
        <div className="mt-auto flex flex-col gap-6">
          <button className="p-2 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors">
            <Bell className="w-6 h-6" />
          </button>
          <button className="p-2 text-slate-500 hover:bg-slate-800 rounded-lg transition-colors">
            <Settings className="w-6 h-6" />
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 overflow-y-auto">
        <header className="flex justify-between items-center mb-10">
          <div>
            <h1 className="text-3xl font-bold text-white tracking-tight">
              Rust Trading Dashboard
            </h1>
            <p className="text-slate-500 text-sm mt-1">
              Real-time market analytics powered by Tauri
            </p>
          </div>
          <div className="flex gap-4">
            <div className="px-4 py-2 bg-slate-800/50 rounded-lg border border-slate-700 flex items-center gap-3">
              <span className="w-2 h-2 rounded-full bg-indigo-500"></span>
              <span className="text-sm font-medium">Network: Stable</span>
            </div>
            <button className="px-6 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-all shadow-lg shadow-indigo-500/20">
              Refresh Data
            </button>
          </div>
        </header>

        {/* Chart Section */}
        <section className="grid grid-cols-1 gap-8">
          <TradingChart />

          <div className="grid grid-cols-3 gap-6">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className="p-6 bg-slate-900/40 border border-slate-800 rounded-2xl backdrop-blur-sm"
              >
                <div className="text-slate-500 text-xs mb-2 uppercase tracking-wider font-bold">
                  Metric {i}
                </div>
                <div className="text-2xl font-bold text-white">0.0000</div>
                <div className="text-emerald-500 text-xs mt-1">+0.00%</div>
              </div>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}

export default App;
