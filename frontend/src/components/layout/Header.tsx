'use client'

import React from 'react'
import { Menu, X, Vote } from 'lucide-react'
import Link from 'next/link'
import Image from 'next/image'
import { useSidebarStore } from '@/store/sidebarStore'
import { cn } from '@/lib/utils'
import { WalletConnectButton } from '@/components/WalletConnector/WalletConnectButton'
import { WalletModal } from '@/components/WalletConnector/WalletModal'
import { ThemeToggle } from '@/components/ui/ThemeToggle'

interface HeaderProps {
  className?: string
}

const Header = ({ className }: HeaderProps) => {
  const { isOpen, toggleSidebar } = useSidebarStore()

  return (
    <>
      <header className={cn(
        "sticky top-0 z-50 w-full border-b border-stellar-lightNavy bg-stellar-darkNavy/80 backdrop-blur",
        className
      )}>
        <div className="container flex h-16 items-center px-4">
          <button
            onClick={toggleSidebar}
            className="mr-4 p-2 rounded-md hover:bg-stellar-lightNavy transition-colors md:hidden"
            aria-label="Toggle sidebar"
          >
            {isOpen ? <X size={20} /> : <Menu size={20} />}
          </button>

          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 rounded-lg overflow-hidden border border-slate-700/50 flex items-center justify-center bg-slate-800 relative">
              <Image src="/guild logo.jpg" alt="SG" width={32} height={32} className="w-full h-full object-cover" />
            </div>
            <h1 className="text-xl font-bold text-stellar-white">Stellar Guilds</h1>
          </div>

          <nav className="hidden md:flex items-center space-x-6 ml-auto">
            <Link href="/guilds" className="text-stellar-slate hover:text-stellar-lightSlate transition-colors">Guilds</Link>
            <Link href="/bounties" className="text-stellar-slate hover:text-stellar-lightSlate transition-colors">Bounties</Link>
            <Link href="/social/forum" className="text-stellar-slate hover:text-stellar-lightSlate transition-colors">Governance</Link>
          </nav>

          <div className="ml-4 flex items-center space-x-3">
            <ThemeToggle />
            <button
              className="p-2 rounded-full hover:bg-stellar-lightNavy transition-colors"
              aria-label="Open governance"
              title="Open governance"
            >
              <Vote size={20} className="text-stellar-slate" />
            </button>
            <WalletConnectButton />
          </div>
        </div>
      </header>

      {/* Wallet selection modal — rendered outside the header for z-index */}
      <WalletModal />
    </>
  )
}

export { Header }