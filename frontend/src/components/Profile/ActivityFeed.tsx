"use client";

import {
  CheckCircle2,
  PlusCircle,
  Trophy,
  Users,
  MessageSquare,
  Shield,
  Zap,
} from "lucide-react";

type ActivityType = "join" | "complete" | "reward" | "guild" | "comment" | "badge";

interface Activity {
  id: string;
  type: ActivityType;
  content: string;
  timestamp: string;
  details?: string;
}

const mockActivities: Activity[] = [
  {
    id: "1",
    type: "complete",
    content: "Bounty completed: Smart Contract Audit",
    timestamp: "2 hours ago",
    details: "Earned 250 XP and 50 XLM",
  },
  {
    id: "2",
    type: "join",
    content: "Joined the Galactic Vanguard Guild",
    timestamp: "5 hours ago",
    details: "Assigned to the 'Sentry' rank",
  },
  {
    id: "3",
    type: "badge",
    content: "Unlocked 'Early Voyager' Badge",
    timestamp: "Yesterday",
  },
  {
    id: "4",
    type: "comment",
    content: "Commented on 'Stellar Governance' proposal",
    timestamp: "2 days ago",
  },
  {
    id: "5",
    type: "reward",
    content: "Claimed Monthly Participation Reward",
    timestamp: "3 days ago",
    details: "10 XLM deposited to wallet",
  },
];

const getIcon = (type: ActivityType) => {
  switch (type) {
    case "join":
      return <PlusCircle className="h-4 w-4 text-emerald-400" />;
    case "complete":
      return <CheckCircle2 className="h-4 w-4 text-blue-400" />;
    case "reward":
      return <Zap className="h-4 w-4 text-yellow-400" />;
    case "guild":
      return <Users className="h-4 w-4 text-purple-400" />;
    case "comment":
      return <MessageSquare className="h-4 w-4 text-slate-400" />;
    case "badge":
      return <Trophy className="h-4 w-4 text-amber-400" />;
    default:
      return <Shield className="h-4 w-4 text-slate-400" />;
  }
};

const getIconBg = (type: ActivityType) => {
  switch (type) {
    case "join": return "bg-emerald-500/10 border-emerald-500/20";
    case "complete": return "bg-blue-500/10 border-blue-500/20";
    case "reward": return "bg-yellow-500/10 border-yellow-500/20";
    case "guild": return "bg-purple-500/10 border-purple-500/20";
    case "comment": return "bg-slate-500/10 border-slate-500/20";
    case "badge": return "bg-amber-500/10 border-amber-500/20";
    default: return "bg-slate-500/10 border-slate-500/20";
  }
};

export default function ActivityFeed() {
  return (
    <div className="rounded-2xl border border-white/5 bg-slate-900/40 p-6 shadow-xl backdrop-blur-sm">
      <div className="mb-6 flex items-center justify-between">
        <h2 className="text-xl font-bold text-white">Activity Feed</h2>
        <span className="rounded-full bg-slate-800/50 px-3 py-1 text-xs font-medium text-slate-400">
          Recent
        </span>
      </div>

      <div className="relative space-y-8 pl-1">
        {/* Vertical Timeline Line */}
        <div className="absolute left-[19px] top-2 bottom-4 w-[2px] bg-gradient-to-b from-slate-700/50 via-slate-700/20 to-transparent"></div>

        {mockActivities.map((activity) => (
          <div key={activity.id} className="relative flex gap-6">
            {/* Circular Icon Wrapper */}
            <div className={`relative z-10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border shadow-lg backdrop-blur-md ${getIconBg(activity.type)} transition-transform hover:scale-110`}>
              {getIcon(activity.type)}
            </div>

            <div className="flex flex-col pt-1">
              <div className="flex flex-col gap-1 sm:flex-row sm:items-baseline sm:gap-3">
                <h3 className="text-sm font-semibold text-slate-100">
                  {activity.content}
                </h3>
                <span className="text-[10px] font-medium uppercase tracking-wider text-slate-500">
                  {activity.timestamp}
                </span>
              </div>

              {activity.details && (
                <p className="mt-1 text-xs text-slate-400 leading-relaxed">
                  {activity.details}
                </p>
              )}
            </div>
          </div>
        ))}
      </div>

      <button className="mt-8 w-full rounded-xl border border-white/5 bg-white/5 py-2.5 text-sm font-medium text-slate-300 transition-colors hover:bg-white/10">
        View All Activity
      </button>
    </div>
  );
}
