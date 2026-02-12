import { useEffect, useState } from "react";
import clsx from "clsx";
import { X, RefreshCw, TrendingUp, TrendingDown } from "lucide-react";
import type {
  KalshiOrder,
  PolymarketOrder,
  KalshiPosition,
  PolymarketPosition,
} from "@/types";
import { Button } from "@/components/common/Button";
import { Spinner } from "@/components/common/Spinner";
import { formatCurrency } from "@/utils/format";
import api from "@/api/client";
import { STATUS_COLORS } from "@/utils/constants";

export function OrderList() {
  const [activeTab, setActiveTab] = useState<"orders" | "positions">("orders");
  const [platform, setPlatform] = useState<"kalshi" | "polymarket">("kalshi");
  const [loading, setLoading] = useState(true);
  const [orders, setOrders] = useState<{
    kalshi: KalshiOrder[];
    polymarket: PolymarketOrder[];
  }>({ kalshi: [], polymarket: [] });
  const [positions, setPositions] = useState<{
    kalshi: KalshiPosition[];
    polymarket: PolymarketPosition[];
  }>({ kalshi: [], polymarket: [] });
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [kalshiOrders, polyOrders, kalshiPositions, polyPositions] =
        await Promise.all([
          api.orders.kalshi.list(),
          api.orders.polymarket.list(),
          api.positions.kalshi.list(),
          api.positions.polymarket.list(),
        ]);

      setOrders({
        kalshi: kalshiOrders.orders || [],
        polymarket: polyOrders.orders || [],
      });
      setPositions({
        kalshi: kalshiPositions.positions || [],
        polymarket: polyPositions.positions || [],
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch data");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleCancelOrder = async (orderId: string) => {
    try {
      if (platform === "kalshi") {
        await api.orders.kalshi.cancel(orderId);
      } else {
        await api.orders.polymarket.cancel(orderId);
      }
      fetchData();
    } catch (err) {
      console.error("Failed to cancel order:", err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Spinner />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-red-400">
        <p>{error}</p>
        <Button variant="ghost" size="sm" onClick={fetchData} className="mt-2">
          <RefreshCw className="w-4 h-4 mr-1" />
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-shrink-0 flex items-center gap-2 p-2 border-b border-gray-700">
        <div className="flex gap-1">
          <button
            onClick={() => setActiveTab("orders")}
            className={clsx(
              "px-2 py-1 text-xs rounded transition-colors",
              activeTab === "orders"
                ? "bg-blue-500/20 text-blue-400"
                : "text-gray-400 hover:text-gray-300",
            )}
          >
            Orders
          </button>
          <button
            onClick={() => setActiveTab("positions")}
            className={clsx(
              "px-2 py-1 text-xs rounded transition-colors",
              activeTab === "positions"
                ? "bg-blue-500/20 text-blue-400"
                : "text-gray-400 hover:text-gray-300",
            )}
          >
            Positions
          </button>
        </div>

        <div className="flex gap-1 ml-2">
          <button
            onClick={() => setPlatform("kalshi")}
            className={clsx(
              "px-2 py-1 text-xs rounded transition-colors",
              platform === "kalshi"
                ? "bg-blue-500/20 text-blue-400"
                : "text-gray-400 hover:text-gray-300",
            )}
          >
            Kalshi
          </button>
          <button
            onClick={() => setPlatform("polymarket")}
            className={clsx(
              "px-2 py-1 text-xs rounded transition-colors",
              platform === "polymarket"
                ? "bg-purple-500/20 text-purple-400"
                : "text-gray-400 hover:text-gray-300",
            )}
          >
            Polymarket
          </button>
        </div>

        <Button
          variant="ghost"
          size="sm"
          onClick={fetchData}
          className="ml-auto"
        >
          <RefreshCw className="w-3 h-3" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {activeTab === "orders" ? (
          platform === "kalshi" ? (
            <KalshiOrderTable
              orders={orders.kalshi}
              onCancel={handleCancelOrder}
            />
          ) : (
            <PolymarketOrderTable
              orders={orders.polymarket}
              onCancel={handleCancelOrder}
            />
          )
        ) : platform === "kalshi" ? (
          <KalshiPositionTable positions={positions.kalshi} />
        ) : (
          <PolymarketPositionTable positions={positions.polymarket} />
        )}
      </div>
    </div>
  );
}

function KalshiOrderTable({
  orders,
  onCancel,
}: {
  orders: KalshiOrder[];
  onCancel: (id: string) => void;
}) {
  if (orders.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 text-sm">No orders</div>
    );
  }

  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-gray-800">
        <tr className="text-gray-400 border-b border-gray-700">
          <th className="text-left p-2">Ticker</th>
          <th className="text-left p-2">Side</th>
          <th className="text-right p-2">Price</th>
          <th className="text-right p-2">Qty</th>
          <th className="text-left p-2">Status</th>
          <th className="p-2"></th>
        </tr>
      </thead>
      <tbody>
        {orders.map((order) => (
          <tr
            key={order.order_id}
            className="border-b border-gray-800 hover:bg-gray-800/50"
          >
            <td className="p-2 text-white font-mono">{order.ticker}</td>
            <td className="p-2">
              <span
                className={clsx(
                  order.side === "yes" ? "text-green-400" : "text-red-400",
                )}
              >
                {order.side.toUpperCase()}
              </span>
            </td>
            <td className="p-2 text-right font-mono">
              {order.yes_price?.toFixed(2) || order.no_price?.toFixed(2) || "-"}
            </td>
            <td className="p-2 text-right font-mono">
              {order.fill_count}/{order.initial_count}
            </td>
            <td className="p-2">
              <span
                className={clsx(
                  "px-1.5 py-0.5 rounded text-[10px]",
                  STATUS_COLORS[order.status],
                )}
              >
                {order.status}
              </span>
            </td>
            <td className="p-2">
              {order.status === "resting" && (
                <button
                  onClick={() => onCancel(order.order_id)}
                  className="text-red-400 hover:text-red-300"
                >
                  <X className="w-3 h-3" />
                </button>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function PolymarketOrderTable({
  orders,
  onCancel,
}: {
  orders: PolymarketOrder[];
  onCancel: (id: string) => void;
}) {
  if (orders.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 text-sm">No orders</div>
    );
  }

  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-gray-800">
        <tr className="text-gray-400 border-b border-gray-700">
          <th className="text-left p-2">Market</th>
          <th className="text-left p-2">Side</th>
          <th className="text-right p-2">Price</th>
          <th className="text-right p-2">Size</th>
          <th className="text-left p-2">Status</th>
          <th className="p-2"></th>
        </tr>
      </thead>
      <tbody>
        {orders.map((order) => (
          <tr
            key={order.id}
            className="border-b border-gray-800 hover:bg-gray-800/50"
          >
            <td className="p-2 text-white truncate max-w-[100px]">
              {order.market}
            </td>
            <td className="p-2">
              <span
                className={clsx(
                  order.side === "BUY" ? "text-green-400" : "text-red-400",
                )}
              >
                {order.side}
              </span>
            </td>
            <td className="p-2 text-right font-mono">{order.price}</td>
            <td className="p-2 text-right font-mono">
              {order.size_matched}/{order.original_size}
            </td>
            <td className="p-2">
              <span
                className={clsx(
                  "px-1.5 py-0.5 rounded text-[10px]",
                  STATUS_COLORS[
                    order.status.toLowerCase() as keyof typeof STATUS_COLORS
                  ] || "bg-gray-700 text-gray-400",
                )}
              >
                {order.status}
              </span>
            </td>
            <td className="p-2">
              {order.status === "LIVE" && (
                <button
                  onClick={() => onCancel(order.id)}
                  className="text-red-400 hover:text-red-300"
                >
                  <X className="w-3 h-3" />
                </button>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function KalshiPositionTable({ positions }: { positions: KalshiPosition[] }) {
  if (positions.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 text-sm">No positions</div>
    );
  }

  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-gray-800">
        <tr className="text-gray-400 border-b border-gray-700">
          <th className="text-left p-2">Ticker</th>
          <th className="text-right p-2">Position</th>
          <th className="text-right p-2">Exposure</th>
          <th className="text-right p-2">PnL</th>
        </tr>
      </thead>
      <tbody>
        {positions.map((pos) => (
          <tr
            key={pos.ticker}
            className="border-b border-gray-800 hover:bg-gray-800/50"
          >
            <td className="p-2 text-white font-mono">{pos.ticker}</td>
            <td className="p-2 text-right font-mono">{pos.position}</td>
            <td className="p-2 text-right font-mono">
              {formatCurrency(pos.market_exposure)}
            </td>
            <td className="p-2 text-right font-mono">
              {pos.realized_pnl !== undefined && (
                <span
                  className={
                    pos.realized_pnl >= 0 ? "text-green-400" : "text-red-400"
                  }
                >
                  {pos.realized_pnl >= 0 ? (
                    <TrendingUp className="w-3 h-3 inline mr-1" />
                  ) : (
                    <TrendingDown className="w-3 h-3 inline mr-1" />
                  )}
                  {formatCurrency(pos.realized_pnl)}
                </span>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function PolymarketPositionTable({
  positions,
}: {
  positions: PolymarketPosition[];
}) {
  if (positions.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 text-sm">No positions</div>
    );
  }

  return (
    <div className="divide-y divide-gray-800">
      {positions.map((pos, idx) => (
        <div key={pos.id || idx} className="p-2 hover:bg-gray-800/50">
          <div className="flex items-center justify-between">
            <span className="text-xs text-white truncate max-w-[150px]">
              {pos.title || pos.asset || "Unknown"}
            </span>
            <span className="text-xs font-mono text-gray-300">
              {pos.size || "0"}
            </span>
          </div>
          <div className="flex items-center justify-between mt-1 text-[10px] text-gray-500">
            <span>Avg: {pos.avgPrice || "-"}</span>
            <span>Value: {pos.value || "-"}</span>
            {pos.pnl && (
              <span
                className={
                  parseFloat(pos.pnl) >= 0 ? "text-green-400" : "text-red-400"
                }
              >
                PnL: {pos.pnl}
              </span>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
