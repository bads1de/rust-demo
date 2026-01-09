import { useEffect, useState, useMemo } from "react";
import { TradingChart } from "./components/TradingChart";
import { TitleBar } from "./components/TitleBar";
import { listen } from "@tauri-apps/api/event";
import { Search, ArrowUpDown, Zap } from "lucide-react";

// Types
interface KlineWithIndicator {
  symbol: string;
  kline: { c: number; o: number };
  rsi: number | null;
  macd: number | null;
  macd_hist: number | null;
}

function App() {
  const [marketData, setMarketData] = useState<
    Record<string, KlineWithIndicator>
  >({});
  const [selectedExchange, setSelectedExchange] = useState("binance");
  const [selectedSymbol, setSelectedSymbol] = useState("BTCUSDT");
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<"symbol" | "rsi" | "change">("symbol");
  const [sortOrder, setSortByOrder] = useState<"asc" | "desc">("asc");

  // 取引所が変更されたら、デフォルトのシンボル形式に合わせる
  useEffect(() => {
    if (selectedExchange === "okx") {
      setSelectedSymbol("BTC-USDT");
    } else {
      setSelectedSymbol("BTCUSDT");
    }
  }, [selectedExchange]);

  // WebSocketイベントの監視 (全銘柄)
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const startListen = async () => {
      unlisten = await listen<KlineWithIndicator>("kline-update", (event) => {
        const data = event.payload;
        // 大量のデータが来るため、パフォーマンスのために状態更新を少し工夫
        setMarketData((prev) => ({
          ...prev,
          [data.symbol]: data,
        }));
      });
    };

    startListen();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // フィルタリングとソート
  const sortedSymbols = useMemo(() => {
    const list = Object.values(marketData).filter((d) =>
      d.symbol.toLowerCase().includes(search.toLowerCase())
    );

    return list.sort((a, b) => {
      let valA: string | number | null;
      let valB: string | number | null;

      if (sortBy === "change") {
        valA = ((a.kline.c - a.kline.o) / a.kline.o) * 100;
        valB = ((b.kline.c - b.kline.o) / b.kline.o) * 100;
      } else {
        valA = a[sortBy as "symbol" | "rsi"];
        valB = b[sortBy as "symbol" | "rsi"];
      }

      if (valA === null) return 1;
      if (valB === null) return -1;

      const order = sortOrder === "asc" ? 1 : -1;
      return valA > valB ? order : -order;
    });
  }, [marketData, search, sortBy, sortOrder]);

  const toggleSort = (key: typeof sortBy) => {
    if (sortBy === key) {
      setSortByOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortBy(key);
      setSortByOrder("desc");
    }
  };

  return (
    <div className="min-h-screen bg-[#0B0E14] text-gray-300 font-sans flex flex-col overflow-hidden">
      <TitleBar />

      {/* Header */}
      <header className="mt-9 h-16 flex items-center justify-between px-6 bg-[#111620] border-b border-white/5 shrink-0">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-black text-white tracking-tighter flex items-center gap-2">
            <Zap size={20} className="text-indigo-500 fill-current" />
            MARKET SCANNER
          </h1>
          <div className="h-6 w-px bg-white/10"></div>
          <p className="text-xs font-mono text-gray-500">
            MONITORING {Object.keys(marketData).length} SYMBOLS
          </p>
          
          <div className="ml-4 flex items-center bg-[#0B0E14] rounded-lg p-1 border border-white/5">
            <button
              onClick={() => setSelectedExchange("binance")}
              className={`px-3 py-1 text-[10px] font-bold rounded transition-all ${
                selectedExchange === "binance"
                  ? "bg-indigo-500 text-white"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              BINANCE
            </button>
            <button
              onClick={() => setSelectedExchange("bybit")}
              className={`px-3 py-1 text-[10px] font-bold rounded transition-all ${
                selectedExchange === "bybit"
                  ? "bg-orange-500 text-white"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              BYBIT
            </button>
            <button
              onClick={() => setSelectedExchange("bitget")}
              className={`px-3 py-1 text-[10px] font-bold rounded transition-all ${
                selectedExchange === "bitget"
                  ? "bg-cyan-500 text-white"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              BITGET
            </button>
            <button
              onClick={() => setSelectedExchange("okx")}
              className={`px-3 py-1 text-[10px] font-bold rounded transition-all ${
                selectedExchange === "okx"
                  ? "bg-white text-black"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              OKX
            </button>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <div className="relative">
            <Search
              className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500"
              size={14}
            />
            <input
              type="text"
              placeholder="Search pairs..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="bg-[#0B0E14] border border-white/5 rounded-full pl-9 pr-4 py-1.5 text-xs focus:outline-none focus:border-indigo-500 transition-all w-64"
            />
          </div>
        </div>
      </header>

      {/* Main Content (Split View) */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Sidebar: Screener List */}
        <aside className="w-80 border-right border-white/5 bg-[#111620]/50 flex flex-col shrink-0 overflow-hidden">
          <div className="grid grid-cols-3 text-[10px] font-bold text-gray-500 uppercase p-3 bg-[#151b26] border-b border-white/5">
            <button
              onClick={() => toggleSort("symbol")}
              className="flex items-center gap-1 hover:text-white"
            >
              PAIR <ArrowUpDown size={10} />
            </button>
            <button
              onClick={() => toggleSort("change")}
              className="flex items-center gap-1 hover:text-white justify-end"
            >
              CHG% <ArrowUpDown size={10} />
            </button>
            <button
              onClick={() => toggleSort("rsi")}
              className="flex items-center gap-1 hover:text-white justify-end"
            >
              RSI <ArrowUpDown size={10} />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {sortedSymbols.map((data) => {
              const change =
                ((data.kline.c - data.kline.o) / data.kline.o) * 100;
              const isSelected = selectedSymbol === data.symbol;
              const rsiColor =
                data.rsi && data.rsi < 30
                  ? "text-emerald-400"
                  : data.rsi && data.rsi > 70
                  ? "text-red-400"
                  : "text-gray-400";

              return (
                <div
                  key={data.symbol}
                  onClick={() => setSelectedSymbol(data.symbol)}
                  className={`grid grid-cols-3 p-3 text-xs border-b border-white/2 cursor-pointer transition-all hover:bg-white/3 ${
                    isSelected
                      ? "bg-indigo-500/10 border-l-2 border-l-indigo-500"
                      : ""
                  }`}
                >
                  <div className="font-bold text-gray-200">
                    {data.symbol.replace("USDT", "")}
                  </div>
                  <div
                    className={`text-right font-mono ${
                      change >= 0 ? "text-emerald-500" : "text-red-500"
                    }`}
                  >
                    {change >= 0 ? "+" : ""}
                    {change.toFixed(2)}%
                  </div>
                  <div className={`text-right font-mono font-bold ${rsiColor}`}>
                    {data.rsi ? data.rsi.toFixed(1) : "--"}
                  </div>
                </div>
              );
            })}
          </div>
        </aside>

        {/* Right Content: Detailed Chart */}
        <main className="flex-1 p-6 bg-[#0B0E14] overflow-y-auto">
          <div className="h-full max-w-6xl mx-auto">
            <TradingChart 
              symbol={selectedSymbol} 
              exchange={selectedExchange} 
              key={`${selectedExchange}-${selectedSymbol}`} 
            />
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
