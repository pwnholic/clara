import { ReactNode } from "react";
import clsx from "clsx";
import { Wifi, WifiOff, Activity, TrendingUp, Clock } from "lucide-react";
import { formatRelativeTime } from "@/utils/format";

interface LayoutProps {
  children: ReactNode;
  isConnected: boolean;
  stats: {
    kalshiCount: number;
    polymarketCount: number;
    matchedCount: number;
    opportunitiesCount: number;
  };
  totalProfit: number;
  lastUpdateTime: Date | null;
}

export function Layout({
  children,
  isConnected,
  stats,
  totalProfit,
  lastUpdateTime,
}: LayoutProps) {
  return (
    <div className="h-screen flex flex-col bg-gray-900 text-gray-100 overflow-hidden">
      <header className="flex-shrink-0 bg-gray-800 border-b border-gray-700 px-4 py-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h1 className="text-lg font-bold text-white flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-blue-500" />
              Polkas
            </h1>

            <div className="flex items-center gap-4 text-sm">
              <div className="flex items-center gap-1">
                {isConnected ? (
                  <Wifi className="w-4 h-4 text-green-500" />
                ) : (
                  <WifiOff className="w-4 h-4 text-red-500" />
                )}
                <span
                  className={clsx(
                    isConnected ? "text-green-400" : "text-red-400",
                  )}
                >
                  {isConnected ? "Connected" : "Disconnected"}
                </span>
              </div>

              <div className="flex items-center gap-1 text-gray-400">
                <Activity className="w-4 h-4" />
                <span className="text-blue-400">{stats.kalshiCount}</span>
                <span>/</span>
                <span className="text-purple-400">{stats.polymarketCount}</span>
                <span className="text-gray-500 ml-1">markets</span>
              </div>

              <div className="flex items-center gap-1">
                <span className="text-gray-500">Matched:</span>
                <span className="text-white font-medium">
                  {stats.matchedCount}
                </span>
              </div>

              <div className="flex items-center gap-1">
                <span className="text-gray-500">Opportunities:</span>
                <span className="text-green-400 font-medium">
                  {stats.opportunitiesCount}
                </span>
              </div>

              <div className="flex items-center gap-1">
                <span className="text-gray-500">Profit:</span>
                <span
                  className={clsx(
                    "font-bold",
                    totalProfit >= 0 ? "text-green-400" : "text-red-400",
                  )}
                >
                  ${totalProfit.toFixed(2)}
                </span>
              </div>

              {lastUpdateTime && (
                <div className="flex items-center gap-1 text-gray-500 text-xs">
                  <Clock className="w-3 h-3" />
                  <span>
                    {formatRelativeTime(lastUpdateTime.toISOString())}
                  </span>
                </div>
              )}
            </div>
          </div>
        </div>
      </header>

      <main className="flex-1 overflow-hidden">{children}</main>
    </div>
  );
}
