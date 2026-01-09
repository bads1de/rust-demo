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

  // 取引所変更時のハンドラー
  const handleExchangeChange = (exchange: string) => {
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
  };

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
      <header className="mt-9 h-18 flex items-center justify-between px-6 bg-[#111620] border-b border-white/5 shrink-0 gap-4">
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
        <div className="flex-1 flex justify-center min-w-0 mx-4">
          <div className="flex items-center bg-[#0B0E14] rounded-lg p-1 border border-white/5 space-x-1 overflow-x-auto max-w-full custom-scrollbar">
            <ExchangeBtn
              name="BINANCE"
              active={selectedExchange === "binance"}
              onClick={() => handleExchangeChange("binance")}
              activeColor="bg-[#F0B90B] text-black"
            />
            <ExchangeBtn
              name="BYBIT"
              active={selectedExchange === "bybit"}
              onClick={() => handleExchangeChange("bybit")}
              activeColor="bg-[#171A1E] text-[#F7A600] border border-[#F7A600]/20"
            />
            <ExchangeBtn
              name="BITGET"
              active={selectedExchange === "bitget"}
              onClick={() => handleExchangeChange("bitget")}
              activeColor="bg-[#00F0FF] text-black"
            />
            <ExchangeBtn
              name="OKX"
              active={selectedExchange === "okx"}
              onClick={() => handleExchangeChange("okx")}
              activeColor="bg-white text-black"
            />
            <ExchangeBtn
              name="KUCOIN"
              active={selectedExchange === "kucoin"}
              onClick={() => handleExchangeChange("kucoin")}
              activeColor="bg-[#00D095] text-white"
            />
            <ExchangeBtn
              name="KRAKEN"
              active={selectedExchange === "kraken"}
              onClick={() => handleExchangeChange("kraken")}
              activeColor="bg-[#5841D8] text-white"
            />
            <ExchangeBtn
              name="GATE"
              active={selectedExchange === "gate"}
              onClick={() => handleExchangeChange("gate")}
              activeColor="bg-[#D32F2F] text-white"
            />
            <ExchangeBtn
              name="MEXC"
              active={selectedExchange === "mexc"}
              onClick={() => handleExchangeChange("mexc")}
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



const ExchangeBtn = ({

  name,

  active,

  onClick,

  activeColor,

}: {

  name: string;

  active: boolean;

  onClick: () => void;

  activeColor: string;

}) => (

  <button

    onClick={onClick}

    className={`px-3 py-1.5 text-[10px] font-bold rounded-md transition-all whitespace-nowrap ${

      active

        ? activeColor + " shadow-lg scale-105"

        : "text-gray-500 hover:text-gray-300 hover:bg-white/5"

    }`}

  >

    {name}

  </button>

);
