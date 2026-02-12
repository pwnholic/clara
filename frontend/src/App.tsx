import { useState, useMemo } from "react";
import { Layout } from "@/components/Layout";
import { OpportunityList } from "@/components/Dashboard";
import { OrderList, OrderForm } from "@/components/Orders";
import { HistoryTable } from "@/components/History";
import { BalanceCard } from "@/components/Accounts";
import { useWebSocket } from "@/hooks/useWebSocket";
import type { MatchedMarketData } from "@/types";
import {
  formatPriceCents,
  formatCurrency,
  formatPercent,
  formatDateTime,
} from "@/utils/format";
import clsx from "clsx";

function App() {
  const wsUrl = useMemo(() => {
    const devPorts = ["5173", "5174", "5175"];
    const isDev = devPorts.includes(window.location.port);
    return isDev
      ? "ws://localhost:8000/ws"
      : `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${window.location.host}/ws`;
  }, []);

  const { matchedMarkets, isConnected, stats, lastUpdateTime, metrics } =
    useWebSocket(wsUrl);

  const [selectedMarket, setSelectedMarket] =
    useState<MatchedMarketData | null>(null);
  const [activeTab, setActiveTab] = useState<"detail" | "orders" | "history">(
    "detail",
  );

  const totalProfit = matchedMarkets
    .filter((m) => m.has_opportunity)
    .reduce((sum, m) => sum + m.expected_profit, 0);

  return (
    <Layout
      isConnected={isConnected}
      stats={stats}
      totalProfit={totalProfit}
      lastUpdateTime={lastUpdateTime}
    >
      <div className="h-full flex gap-2 p-2">
        <div className="flex-1 flex flex-col min-w-0 gap-2">
          <div className="flex-1 min-h-0">
            <OpportunityList
              markets={matchedMarkets}
              onSelectMarket={setSelectedMarket}
              selectedMarketId={
                selectedMarket
                  ? `${selectedMarket.kalshi_market_id}_${selectedMarket.polymarket_market_id}`
                  : undefined
              }
            />
          </div>
          <div className="h-64 flex-shrink-0">
            <OrderList />
          </div>
        </div>

        <aside className="w-96 flex-shrink-0 flex flex-col gap-2">
          <div className="flex items-center gap-1 text-xs border-b border-gray-700 pb-2">
            {(["detail", "orders", "history"] as const).map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={clsx(
                  "px-3 py-1.5 rounded transition-colors capitalize",
                  activeTab === tab
                    ? "bg-blue-500/20 text-blue-400"
                    : "text-gray-400 hover:text-gray-300",
                )}
              >
                {tab}
              </button>
            ))}
          </div>

          <div className="flex-1 min-h-0 overflow-hidden">
            {activeTab === "detail" ? (
              selectedMarket ? (
                <MarketDetail market={selectedMarket} />
              ) : (
                <div className="flex flex-col items-center justify-center h-full text-gray-500">
                  <p className="text-sm">Select a market to view details</p>
                </div>
              )
            ) : activeTab === "orders" ? (
              <div className="space-y-4">
                <BalanceCard />
                {selectedMarket && <OrderForm market={selectedMarket} />}
              </div>
            ) : (
              <HistoryTable />
            )}
          </div>

          {metrics && (
            <div className="flex-shrink-0 bg-gray-800 rounded-lg p-2 border border-gray-700">
              <div className="text-[10px] text-gray-400 uppercase font-medium mb-1">
                API Latency
              </div>
              <div className="flex items-center gap-4 text-xs">
                <span className="text-blue-400">
                  Kalshi: {metrics.api_latency.kalshi_ms?.toFixed(0) || "-"}ms
                </span>
                <span className="text-purple-400">
                  Polymarket:{" "}
                  {metrics.api_latency.polymarket_ms?.toFixed(0) || "-"}ms
                </span>
              </div>
            </div>
          )}
        </aside>
      </div>
    </Layout>
  );
}

function MarketDetail({ market }: { market: MatchedMarketData }) {
  return (
    <div className="h-full overflow-y-auto p-2 space-y-3">
      <div className="bg-gray-800 rounded-lg p-3 border border-gray-700">
        <h3 className="text-sm font-semibold text-white mb-1">
          {market.event_name}
        </h3>
        {market.team_name && (
          <p className="text-xs text-yellow-400">{market.team_name}</p>
        )}
        {market.end_time && (
          <p className="text-xs text-gray-500 mt-1">
            {formatDateTime(market.end_time)}
          </p>
        )}
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-3">
          <div className="text-xs text-blue-400 font-medium mb-2">Kalshi</div>
          <div className="space-y-1 text-xs">
            <div className="flex justify-between">
              <span className="text-green-400">Yes</span>
              <span className="font-mono">
                {formatPriceCents(market.kalshi_yes_price)}¢
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-red-400">No</span>
              <span className="font-mono">
                {formatPriceCents(market.kalshi_no_price)}¢
              </span>
            </div>
          </div>
          <div className="mt-2 text-[10px]">
            <span
              className={clsx(
                "px-1.5 py-0.5 rounded",
                market.kalshi_ready
                  ? "bg-blue-500/20 text-blue-400"
                  : "bg-gray-700 text-gray-500",
              )}
            >
              {market.kalshi_ready ? "Ready" : "Not Ready"}
            </span>
          </div>
        </div>

        <div className="bg-purple-500/10 border border-purple-500/20 rounded-lg p-3">
          <div className="text-xs text-purple-400 font-medium mb-2">
            Polymarket
          </div>
          <div className="space-y-1 text-xs">
            <div className="flex justify-between">
              <span className="text-green-400">Yes</span>
              <span className="font-mono">
                {formatPriceCents(market.poly_yes_price)}¢
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-red-400">No</span>
              <span className="font-mono">
                {formatPriceCents(market.poly_no_price)}¢
              </span>
            </div>
          </div>
          <div className="mt-2 text-[10px]">
            <span
              className={clsx(
                "px-1.5 py-0.5 rounded",
                market.poly_ready
                  ? "bg-purple-500/20 text-purple-400"
                  : "bg-gray-700 text-gray-500",
              )}
            >
              {market.poly_ready ? "Ready" : "Not Ready"}
            </span>
          </div>
        </div>
      </div>

      <div
        className={clsx(
          "rounded-lg p-3 border",
          market.has_opportunity
            ? "bg-green-500/10 border-green-500/20"
            : "bg-gray-800 border-gray-700",
        )}
      >
        <div className="text-xs text-gray-400 mb-1">Arbitrage Opportunity</div>
        {market.has_opportunity ? (
          <>
            <div className="text-lg font-bold text-green-400">
              {formatPercent(market.profit_margin)}
            </div>
            <div className="text-xs text-green-400/70 mt-1">
              Net Profit: {formatCurrency(market.expected_profit)}
            </div>
            {market.arbitrage_type && (
              <div className="text-xs text-gray-400 mt-2">
                Strategy: {market.arbitrage_type}
              </div>
            )}
          </>
        ) : (
          <div className="text-sm text-gray-500">No opportunity</div>
        )}
      </div>

      {market.has_opportunity && (
        <div className="bg-gray-800 rounded-lg p-3 border border-gray-700">
          <div className="text-xs text-gray-400 mb-2">Trade Details</div>
          <div className="space-y-1 text-xs">
            {market.kalshi_contracts !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-500">Kalshi Contracts</span>
                <span className="text-blue-400 font-mono">
                  {Math.round(market.kalshi_contracts)}
                </span>
              </div>
            )}
            {market.kalshi_fee !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-500">Kalshi Fee</span>
                <span className="text-orange-400 font-mono">
                  -{formatCurrency(market.kalshi_fee)}
                </span>
              </div>
            )}
            {market.gross_profit !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-500">Gross Profit</span>
                <span className="font-mono">
                  {formatCurrency(market.gross_profit)}
                </span>
              </div>
            )}
            <div className="flex justify-between pt-1 border-t border-gray-700">
              <span className="text-gray-400 font-medium">Net Profit</span>
              <span className="text-green-400 font-mono font-medium">
                {formatCurrency(market.expected_profit)}
              </span>
            </div>
          </div>
        </div>
      )}

      <div className="pt-2 border-t border-gray-700">
        <OrderForm market={market} />
      </div>
    </div>
  );
}

export default App;
