import { useState, useEffect } from "react";
import { History, RefreshCw } from "lucide-react";
import type { HistoryEntry } from "@/types";
import { Button } from "@/components/common/Button";
import { Spinner } from "@/components/common/Spinner";
import {
  formatCurrency,
  formatPercent,
  formatRelativeTime,
} from "@/utils/format";
import api from "@/api/client";
import { STATUS_COLORS } from "@/utils/constants";
import clsx from "clsx";

interface HistoryTableProps {
  limit?: number;
}

export function HistoryTable({ limit = 50 }: HistoryTableProps) {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [total, setTotal] = useState(0);

  const fetchHistory = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.history.list(limit);
      setEntries(result.entries || []);
      setTotal(result.total || 0);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch history");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchHistory();
  }, [limit]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-32">
        <Spinner />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-32 text-red-400">
        <p className="text-sm">{error}</p>
        <Button
          variant="ghost"
          size="sm"
          onClick={fetchHistory}
          className="mt-2"
        >
          <RefreshCw className="w-4 h-4 mr-1" />
          Retry
        </Button>
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-32 text-gray-500">
        <History className="w-8 h-8 mb-2" />
        <p className="text-sm">No trade history</p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-shrink-0 flex items-center justify-between p-2 border-b border-gray-700">
        <span className="text-xs text-gray-400">{total} trades</span>
        <Button variant="ghost" size="sm" onClick={fetchHistory}>
          <RefreshCw className="w-3 h-3" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-gray-800">
            <tr className="text-gray-400 border-b border-gray-700">
              <th className="text-left p-2">Time</th>
              <th className="text-left p-2">Event</th>
              <th className="text-left p-2">Side</th>
              <th className="text-right p-2">Profit</th>
              <th className="text-left p-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((entry) => (
              <tr
                key={entry.id}
                className="border-b border-gray-800 hover:bg-gray-800/50"
              >
                <td className="p-2 text-gray-400">
                  {formatRelativeTime(entry.timestamp)}
                </td>
                <td className="p-2">
                  <div className="text-white truncate max-w-[120px]">
                    {entry.event_name}
                  </div>
                  {entry.team_name && (
                    <div className="text-yellow-400 text-[10px]">
                      {entry.team_name}
                    </div>
                  )}
                </td>
                <td className="p-2">
                  <div className="flex items-center gap-1">
                    <span className="text-blue-400">
                      K:{entry.kalshi_side.toUpperCase()}
                    </span>
                    <span className="text-gray-500">/</span>
                    <span className="text-purple-400">
                      P:{entry.poly_side.toUpperCase()}
                    </span>
                  </div>
                </td>
                <td className="p-2 text-right">
                  <div
                    className={clsx(
                      "font-mono",
                      entry.profit_margin >= 0
                        ? "text-green-400"
                        : "text-red-400",
                    )}
                  >
                    {formatPercent(entry.profit_margin)}
                  </div>
                  {entry.realized_profit !== undefined && (
                    <div className="text-[10px] text-gray-500">
                      {formatCurrency(entry.realized_profit)}
                    </div>
                  )}
                </td>
                <td className="p-2">
                  <span
                    className={clsx(
                      "px-1.5 py-0.5 rounded text-[10px]",
                      STATUS_COLORS[entry.status],
                    )}
                  >
                    {entry.status}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
