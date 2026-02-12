import { useState } from "react";
import clsx from "clsx";
import { Send, AlertTriangle, Loader2 } from "lucide-react";
import type {
  MatchedMarketData,
  OrderRequest,
  ArbitrageExecuteRequest,
} from "@/types";
import { Button } from "@/components/common/Button";
import { Input } from "@/components/common/Input";
import { Select } from "@/components/common/Input";
import {
  formatPriceCents,
  formatCurrency,
  formatPercent,
} from "@/utils/format";
import api from "@/api/client";

interface OrderFormProps {
  market: MatchedMarketData;
  onSuccess?: () => void;
}

export function OrderForm({ market, onSuccess }: OrderFormProps) {
  const [side, setSide] = useState<"yes" | "no">("yes");
  const [amount, setAmount] = useState("100");
  const [contracts, setContracts] = useState("10");
  const [orderType, setOrderType] = useState<"single" | "arbitrage">(
    "arbitrage",
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const kalshiPrice =
    side === "yes" ? market.kalshi_yes_price : market.kalshi_no_price;
  const polyPrice =
    side === "yes" ? market.poly_yes_price : market.poly_no_price;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      if (orderType === "arbitrage") {
        const arbitrageRequest: ArbitrageExecuteRequest = {
          kalshi_ticker: market.kalshi_market_id,
          kalshi_side: side,
          kalshi_bet: parseFloat(amount),
          kalshi_price: kalshiPrice,
          poly_token_id: market.poly_token_id || "",
          poly_side: side === "yes" ? "buy" : "sell",
          poly_amount: parseFloat(amount),
        };

        const result = await api.arbitrage.execute(arbitrageRequest);

        if (result.success) {
          setSuccess("Arbitrage executed successfully!");
          onSuccess?.();
        } else {
          setError(result.error || "Arbitrage execution failed");
        }
      } else {
        const orderRequest: OrderRequest = {
          ticker: market.kalshi_market_id,
          side,
          action: "buy",
          count: parseInt(contracts),
        };

        const result = await api.orders.kalshi.create(orderRequest);

        if (result.success) {
          setSuccess("Order placed successfully!");
          onSuccess?.();
        } else {
          setError(result.error || "Order failed");
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  const estimatedProfit = market.has_opportunity
    ? (parseFloat(amount) * market.profit_margin) / 100
    : 0;

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div className="bg-gray-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-3">
          <span className="text-xs font-medium text-gray-400">Order Type</span>
          <div className="flex gap-1">
            <button
              type="button"
              onClick={() => setOrderType("arbitrage")}
              className={clsx(
                "px-2 py-1 text-xs rounded transition-colors",
                orderType === "arbitrage"
                  ? "bg-green-500/20 text-green-400 border border-green-500/30"
                  : "bg-gray-700 text-gray-400",
              )}
            >
              Arbitrage
            </button>
            <button
              type="button"
              onClick={() => setOrderType("single")}
              className={clsx(
                "px-2 py-1 text-xs rounded transition-colors",
                orderType === "single"
                  ? "bg-blue-500/20 text-blue-400 border border-blue-500/30"
                  : "bg-gray-700 text-gray-400",
              )}
            >
              Single
            </button>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Select
            label="Side"
            value={side}
            onChange={(e) => setSide(e.target.value as "yes" | "no")}
            options={[
              { value: "yes", label: "Yes" },
              { value: "no", label: "No" },
            ]}
          />

          {orderType === "arbitrage" ? (
            <Input
              label="Amount (USD)"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              min="1"
              step="1"
            />
          ) : (
            <Input
              label="Contracts"
              type="number"
              value={contracts}
              onChange={(e) => setContracts(e.target.value)}
              min="1"
              step="1"
            />
          )}
        </div>

        <div className="mt-3 grid grid-cols-2 gap-2 text-xs">
          <div className="bg-gray-900 rounded p-2">
            <div className="text-gray-500">Kalshi {side.toUpperCase()}</div>
            <div className="text-blue-400 font-mono">
              {formatPriceCents(kalshiPrice)}¢
            </div>
          </div>
          <div className="bg-gray-900 rounded p-2">
            <div className="text-gray-500">Poly {side.toUpperCase()}</div>
            <div className="text-purple-400 font-mono">
              {formatPriceCents(polyPrice)}¢
            </div>
          </div>
        </div>

        {market.has_opportunity && orderType === "arbitrage" && (
          <div className="mt-3 bg-green-500/10 border border-green-500/20 rounded p-2">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-400">Est. Profit Margin</span>
              <span className="text-green-400 font-medium">
                {formatPercent(market.profit_margin)}
              </span>
            </div>
            <div className="flex items-center justify-between text-xs mt-1">
              <span className="text-gray-400">Est. Profit</span>
              <span className="text-green-400 font-bold">
                {formatCurrency(estimatedProfit)}
              </span>
            </div>
          </div>
        )}
      </div>

      {error && (
        <div className="flex items-center gap-2 text-red-400 text-xs bg-red-500/10 rounded p-2">
          <AlertTriangle className="w-4 h-4 flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {success && (
        <div className="flex items-center gap-2 text-green-400 text-xs bg-green-500/10 rounded p-2">
          <span>{success}</span>
        </div>
      )}

      <Button
        type="submit"
        className="w-full"
        loading={loading}
        disabled={!market.both_ready}
      >
        {loading ? (
          <>
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            Processing...
          </>
        ) : (
          <>
            <Send className="w-4 h-4 mr-2" />
            {orderType === "arbitrage" ? "Execute Arbitrage" : "Place Order"}
          </>
        )}
      </Button>

      {!market.both_ready && (
        <div className="text-xs text-yellow-400 text-center">
          Market not ready for trading
        </div>
      )}
    </form>
  );
}
