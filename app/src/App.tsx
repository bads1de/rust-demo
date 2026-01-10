import { useEffect, useState, useMemo, useCallback, memo } from "react";
import { TradingChart } from "./components/TradingChart";
import { TitleBar } from "./components/TitleBar";
import { ScannerPanel } from "./components/ScannerPanel";
import { listen } from "@tauri-apps/api/event";
import { Search, ArrowUpDown, Zap, Scan, List } from "lucide-react";

// Types
interface KlineWithIndicator {
  symbol: string;
  c: number;
  o: number;
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
  const [sidebarTab, setSidebarTab] = useState<"list" | "scanner">("list");

  // 取引所変更時のハンドラー
  const handleExchangeChange = useCallback((exchange: string) => {
    setSelectedExchange(exchange);

    // 取引所に応じたデフォルトシンボルを設定
    if (exchange === "okx" || exchange === "kucoin") {
      setSelectedSymbol("BTC-USDT");
    } else if (exchange === "kraken") {
      setSelectedSymbol("XBTUSDT");
    } else if (exchange === "gate") {
      setSelectedSymbol("BTC_USDT");
    } else {
      setSelectedSymbol("BTCUSDT"); // Binance, Bybit, Mexc, Bitget
    }
  }, []);

  // WebSocketイベントの監視 (全銘柄) - バッチ更新で負荷軽減
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let buffer: Record<string, KlineWithIndicator> = {};
    let timeoutId: number | null = null;
    let isMounted = true;

    const startListen = async () => {
      unlisten = await listen<KlineWithIndicator>("kline-update", (event) => {
        if (!isMounted) return; // Skip if unmounted
        const data = event.payload;
        buffer[data.symbol] = data;

        if (timeoutId === null) {
          timeoutId = window.setTimeout(() => {
            if (!isMounted) return; // Double check before setState
            setMarketData((prev) => ({
              ...prev,
              ...buffer,
            }));
            buffer = {};
            timeoutId = null;
          }, 100); // 100msごとにバッチ更新
        }
      });
    };

    startListen();
    return () => {
      isMounted = false;
      if (unlisten) unlisten();
      if (timeoutId !== null) window.clearTimeout(timeoutId);
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
        valA = ((a.c - a.o) / a.o) * 100;
        valB = ((b.c - b.o) / b.o) * 100;
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
      <header className="mt-9 h-18 flex items-center justify-between px-6 bg-[#111620] border-b border-white/5 shrink-0 gap-4 relative z-10">
        {/* Left: Title & Stats */}
        <div className="flex flex-col justify-center shrink-0">
          <h1 className="text-xl font-black text-white tracking-tighter flex items-center gap-2">
            <Zap size={20} className="text-indigo-500 fill-current" />
            MARKET SCANNER
          </h1>
          <div className="flex items-center gap-2 mt-1">
            <span className="text-[10px] font-bold text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded flex items-center gap-1.5 border border-emerald-400/20">
              <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></div>
              {Object.keys(marketData).length} LIVE PAIRS
            </span>
            <span className="text-[10px] font-bold text-gray-500 bg-white/5 px-2 py-0.5 rounded border border-white/5">
              8 EXCHANGES
            </span>
          </div>
        </div>

        {/* Center: Exchange Selector */}
        <div className="flex-1 flex justify-center min-w-0 mx-4 relative">
          <div className="flex items-center bg-[#0B0E14] rounded-lg p-1 border border-white/5 space-x-1 overflow-x-auto max-w-full custom-scrollbar relative z-20">
            <ExchangeBtn
              name="BINANCE"
              active={selectedExchange === "binance"}
              onClick={handleExchangeChange}
              id="binance"
              activeColor="bg-[#F0B90B] text-black"
            />
            <ExchangeBtn
              name="BYBIT"
              active={selectedExchange === "bybit"}
              onClick={handleExchangeChange}
              id="bybit"
              activeColor="bg-[#171A1E] text-[#F7A600] border border-[#F7A600]/20"
            />
            <ExchangeBtn
              name="BITGET"
              active={selectedExchange === "bitget"}
              onClick={handleExchangeChange}
              id="bitget"
              activeColor="bg-[#00F0FF] text-black"
            />
            <ExchangeBtn
              name="OKX"
              active={selectedExchange === "okx"}
              onClick={handleExchangeChange}
              id="okx"
              activeColor="bg-white text-black"
            />
            <ExchangeBtn
              name="KUCOIN"
              active={selectedExchange === "kucoin"}
              onClick={handleExchangeChange}
              id="kucoin"
              activeColor="bg-[#00D095] text-white"
            />
            <ExchangeBtn
              name="KRAKEN"
              active={selectedExchange === "kraken"}
              onClick={handleExchangeChange}
              id="kraken"
              activeColor="bg-[#5841D8] text-white"
            />
            <ExchangeBtn
              name="GATE"
              active={selectedExchange === "gate"}
              onClick={handleExchangeChange}
              id="gate"
              activeColor="bg-[#D32F2F] text-white"
            />
            <ExchangeBtn
              name="MEXC"
              active={selectedExchange === "mexc"}
              onClick={handleExchangeChange}
              id="mexc"
              activeColor="bg-[#2E7BCF] text-white"
            />
          </div>
        </div>

        {/* Right: Search */}
        <div className="flex items-center gap-4 shrink-0">
          <div className="relative group">
            <Search
              className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-indigo-500 transition-colors"
              size={14}
            />
            <input
              type="text"
              placeholder="Search pairs..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="bg-[#0B0E14] border border-white/5 rounded-full pl-9 pr-4 py-1.5 text-xs focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/50 transition-all w-48 focus:w-64"
            />
          </div>
        </div>
      </header>

      {/* Main Content (Split View) */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Sidebar */}
        <aside className="w-80 border-right border-white/5 bg-[#111620]/50 flex flex-col shrink-0 overflow-hidden">
          {/* Tab Selector */}
          <div className="flex border-b border-white/5 bg-[#151b26]">
            <button
              onClick={() => setSidebarTab("list")}
              className={`flex-1 py-2 text-[10px] font-bold uppercase flex items-center justify-center gap-1.5 transition-all ${
                sidebarTab === "list"
                  ? "text-white border-b-2 border-indigo-500"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              <List size={12} />
              LIVE
            </button>
            <button
              onClick={() => setSidebarTab("scanner")}
              className={`flex-1 py-2 text-[10px] font-bold uppercase flex items-center justify-center gap-1.5 transition-all ${
                sidebarTab === "scanner"
                  ? "text-white border-b-2 border-indigo-500"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              <Scan size={12} />
              SCANNER
            </button>
          </div>

          {sidebarTab === "scanner" ? (
            <ScannerPanel
              exchange={selectedExchange}
              onSymbolSelect={setSelectedSymbol}
            />
          ) : (
            <>
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
                  const change = ((data.c - data.o) / data.o) * 100;
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
                      <div
                        className={`text-right font-mono font-bold ${rsiColor}`}
                      >
                        {data.rsi ? data.rsi.toFixed(1) : "--"}
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}
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

const ExchangeBtn = memo(
  ({
    name,
    active,
    onClick,
    activeColor,
    id,
  }: {
    name: string;
    active: boolean;
    onClick: (id: string) => void;
    activeColor: string;
    id: string;
  }) => (
    <button
      onClick={() => onClick(id)}
      type="button"
      className={`px-3 py-1.5 text-[10px] font-bold rounded-md transition-all whitespace-nowrap cursor-pointer relative z-30 ${
        active
          ? activeColor + " shadow-lg scale-105"
          : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
      }`}
    >
      {name}
    </button>
  )
);

export default App;
