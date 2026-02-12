import { useEffect, useState } from "react";
import {
  Wallet,
  RefreshCw,
  AlertCircle,
  TrendingUp,
  TrendingDown,
} from "lucide-react";
import type { AccountBalance } from "@/types";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from "@/components/common/Card";
import { Button } from "@/components/common/Button";
import { Spinner } from "@/components/common/Spinner";
import { formatCurrency } from "@/utils/format";
import api from "@/api/client";
import clsx from "clsx";

export function BalanceCard() {
  const [balance, setBalance] = useState<AccountBalance | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.accounts.balance();
      setBalance(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch balance");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBalance();
    const interval = setInterval(fetchBalance, 30000);
    return () => clearInterval(interval);
  }, []);

  if (loading && !balance) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center h-24">
          <Spinner />
        </CardContent>
      </Card>
    );
  }

  if (error && !balance) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center h-24 text-red-400">
          <AlertCircle className="w-5 h-5 mb-1" />
          <span className="text-xs">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            onClick={fetchBalance}
            className="mt-2"
          >
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  const kalshi = balance?.kalshi;
  const polymarket = balance?.polymarket;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle className="flex items-center gap-2">
          <Wallet className="w-4 h-4" />
          Account Balances
        </CardTitle>
        <Button
          variant="ghost"
          size="sm"
          onClick={fetchBalance}
          disabled={loading}
        >
          <RefreshCw className={clsx("w-3 h-3", loading && "animate-spin")} />
        </Button>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-blue-400">Kalshi</span>
            {kalshi?.available ? (
              <span className="text-[10px] text-green-400 bg-green-500/20 px-1.5 py-0.5 rounded">
                Connected
              </span>
            ) : (
              <span className="text-[10px] text-red-400 bg-red-500/20 px-1.5 py-0.5 rounded">
                Disconnected
              </span>
            )}
          </div>
          {kalshi?.available ? (
            <div className="space-y-1">
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Balance</span>
                <span className="text-white font-mono">
                  {formatCurrency(kalshi.balance || 0)}
                </span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Portfolio</span>
                <span className="text-white font-mono">
                  {formatCurrency(kalshi.portfolio_value || 0)}
                </span>
              </div>
              {kalshi.pnl && (
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">PnL</span>
                  <span
                    className={clsx(
                      "font-mono",
                      parseFloat(kalshi.pnl) >= 0
                        ? "text-green-400"
                        : "text-red-400",
                    )}
                  >
                    {parseFloat(kalshi.pnl) >= 0 ? (
                      <TrendingUp className="w-3 h-3 inline mr-1" />
                    ) : (
                      <TrendingDown className="w-3 h-3 inline mr-1" />
                    )}
                    {kalshi.pnl}
                  </span>
                </div>
              )}
            </div>
          ) : (
            <div className="text-xs text-gray-500">
              {kalshi?.error || "Not available"}
            </div>
          )}
        </div>

        <div className="bg-purple-500/10 border border-purple-500/20 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-purple-400">
              Polymarket
            </span>
            {polymarket?.available ? (
              <span className="text-[10px] text-green-400 bg-green-500/20 px-1.5 py-0.5 rounded">
                Connected
              </span>
            ) : (
              <span className="text-[10px] text-red-400 bg-red-500/20 px-1.5 py-0.5 rounded">
                Disconnected
              </span>
            )}
          </div>
          {polymarket?.available ? (
            <div className="space-y-1">
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Balance</span>
                <span className="text-white font-mono">
                  {formatCurrency(polymarket.balance || 0)}
                </span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-gray-400">Portfolio</span>
                <span className="text-white font-mono">
                  {formatCurrency(polymarket.portfolio_value || 0)}
                </span>
              </div>
              {polymarket.pnl && (
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">PnL</span>
                  <span
                    className={clsx(
                      "font-mono",
                      parseFloat(polymarket.pnl) >= 0
                        ? "text-green-400"
                        : "text-red-400",
                    )}
                  >
                    {parseFloat(polymarket.pnl) >= 0 ? (
                      <TrendingUp className="w-3 h-3 inline mr-1" />
                    ) : (
                      <TrendingDown className="w-3 h-3 inline mr-1" />
                    )}
                    {polymarket.pnl}
                  </span>
                </div>
              )}
            </div>
          ) : (
            <div className="text-xs text-gray-500">
              {polymarket?.error || "Not available"}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
