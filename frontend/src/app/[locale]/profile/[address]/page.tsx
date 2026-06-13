import React from "react";
import {
  mockUser,
  mockStats,
  mockAchievements,
} from "@/features/profile/mockData";
import { ReputationCard } from "@/features/profile/components/ReputationCard";
import { StatsOverview } from "@/features/profile/components/StatsOverview";
import { AchievementGrid } from "@/features/profile/components/AchievementGrid";
import ActivityFeed from "@/components/Profile/ActivityFeed";

// social additions
import { ProfileSocialSection } from "@/components/Profile/ProfileSocialSection";
import { mockFeed } from "@/features/social/mockData";
import { Settings, Share2 } from "lucide-react";
import Image from "next/image";
import Link from "next/link";

export default async function ProfilePage({
  params,
}: {
  params: Promise<{ address: string }>;
}) {
  await params;
  return (


    <main className="min-h-screen bg-slate-950 text-white pb-20 pt-10 px-4 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-5xl space-y-8">

        {/* Header Section */}
        <div className="relative rounded-2xl bg-slate-900/40 p-6 shadow-md border border-slate-800/50 sm:p-8 transition-shadow duration-300 hover:shadow-xl">
          <div className="absolute right-6 top-6 flex gap-3">
            <button className="rounded-full bg-slate-800/50 p-2 text-slate-400 hover:bg-slate-800 transition-colors">
              <Share2 className="h-5 w-5" />
            </button>
            <Link href="/profile/settings">
              <button className="rounded-full bg-slate-800/50 p-2 text-slate-400 hover:bg-slate-800 transition-colors">
                <Settings className="h-5 w-5" />
              </button>
            </Link>
          </div>

          <div className="flex flex-col items-center gap-6 sm:flex-row sm:items-start">
            <div className="relative h-24 w-24 shrink-0 overflow-hidden rounded-full border-4 border-slate-800 shadow-lg sm:h-32 sm:w-32">
              <Image
                src={mockUser.avatarUrl}
                alt={mockUser.displayName}
                width={128}
                height={128}
                className="h-full w-full object-cover"
              />
            </div>

            <div className="flex-1 text-center sm:text-left sm:pt-2">
              <h1 className="text-3xl font-bold text-white">
                {mockUser.displayName}
              </h1>
              <div className="mt-1 flex flex-wrap items-center justify-center gap-3 sm:justify-start">
                <span className="font-mono text-sm text-slate-500 bg-slate-800/50 px-2 py-1 rounded">
                  {mockUser.address}
                </span>
                <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium border
                  ${mockUser.tier === 'Gold' ? 'bg-yellow-100 text-yellow-800 border-yellow-200' : 'bg-slate-800/50 text-gray-800 border-gray-200'}`}>
                  {mockUser.tier} Member
                </span>
              </div>
              <p className="mt-4 max-w-2xl text-slate-400">
                {mockUser.bio}
              </p>
            </div>
          </div>
        </div>

        {/* Reputation & Stats */}
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <ReputationCard user={mockUser} />
          </div>
          <div className="lg:col-span-1">
            {/* Stats are integrated into a grid in the component, but we can wrap it or just place it */}
            {/* The StatsOverview is a grid of 3. It might look squashed in 1 col.
                 Let's place it below Rep Card full width, OR keep it here. 
                 The StatsOverview component is responsive grid-cols-1 sm:grid-cols-3.
                 If placed in lg:col-span-1 (1/3 width), it will be a stack of 3 cards. Which is good.
             */}
            <div className="h-full flex flex-col justify-center">
              {/* We pass stats to the component */}
            </div>
          </div>
        </div>

        {/* Wait, I want StatsOverview to be prominent. Let's put it full width below Rep Card. */}
        <StatsOverview stats={mockStats} />

        {/* Achievements & Activity */}
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <AchievementGrid achievements={mockAchievements} />
          </div>
          <div className="lg:col-span-1">
            <ActivityFeed />
          </div>
        </div>

        {/* Social feed (planned) */}
        <ProfileSocialSection feedItems={mockFeed} />

      </div>

      {/* user profile  */}
      {/* <div className="z-10 max-w-5xl w-full items-center justify-center font-mono text-sm lg:flex flex-col gap-6 text-center">
        <h1 className="text-4xl font-bold text-white">Stellar Guilds</h1>
        <p className="text-xl text-slate-400">User Profile & Reputation Dashboard</p>
        
        <Link 
          href="/profile/GABX...9KLM"
          className="group flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 text-white transition-all hover:bg-blue-700"
        >
          View Demo Profile
          <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
        </Link>
        
        <p className="mt-4 text-gray-400">
           Demo Address: GABX...9KLM
        </p>
      </div> */}
    </main>
  );
}
