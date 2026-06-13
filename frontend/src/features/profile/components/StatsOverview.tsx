import React from "react";
import { Stats } from "../types";
import { CheckCircle2, Coins, TrendingUp } from "lucide-react";

interface StatsOverviewProps {
  stats: Stats;
}

export const StatsOverview: React.FC<StatsOverviewProps> = ({ stats }) => {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <div className="rounded-xl border border-white/5 bg-slate-900/40 p-4 shadow-sm transition-all duration-300 hover:shadow-md hover:scale-105">
        <div className="flex items-center gap-3">
          <div className="rounded-lg bg-blue-500/10 p-2 text-blue-400">
            <CheckCircle2 className="h-5 w-5" />
          </div>
          <div>
            <p className="text-sm font-medium text-slate-500">
              Bounties Completed
            </p>
            <p className="text-xl font-bold text-slate-100">
              {stats.bountiesCompleted}
            </p>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-white/5 bg-slate-900/40 p-4 shadow-sm transition-all duration-300 hover:shadow-md hover:scale-105">
        <div className="flex items-center gap-3">
          <div className="rounded-lg bg-yellow-500/10 p-2 text-yellow-400">
            <Coins className="h-5 w-5" />
          </div>
          <div>
            <p className="text-sm font-medium text-slate-500">Total Earned</p>
            <p className="text-xl font-bold text-slate-100">
              {stats.totalEarned} XLM
            </p>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-white/5 bg-slate-900/40 p-4 shadow-sm transition-all duration-300 hover:shadow-md hover:scale-105">
        <div className="flex items-center gap-3">
          <div className="rounded-lg bg-emerald-500/10 p-2 text-emerald-400">
            <TrendingUp className="h-5 w-5" />
          </div>
          <div>
            <p className="text-sm font-medium text-slate-500">Success Rate</p>
            <p className="text-xl font-bold text-slate-100">
              {stats.successRate}%
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
