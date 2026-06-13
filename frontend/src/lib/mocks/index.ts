import { Guild, Bounty, UserProfile } from '@/types/ui'

// Re-export governance mocks
export * from './governance'

// Mock Guild Data
export const mockGuilds: Guild[] = [
  {
    id: 'guild-1',
    name: 'Cosmic Explorers',
    description: 'Exploring the far reaches of the Stellar network',
    tier: 'gold',
    memberCount: 127,
    reputation: 95,
    createdAt: '2024-01-15',
    logo: '/guild logo.jpg'
  },
  {
    id: 'guild-2',
    name: 'Blockchain Builders',
    description: 'Building the future of decentralized applications',
    tier: 'platinum',
    memberCount: 89,
    reputation: 98,
    createdAt: '2023-11-22',
    logo: '/guild logo.jpg'
  },
  {
    id: 'guild-3',
    name: 'Smart Contract Masters',
    description: 'Mastering Soroban smart contract development',
    tier: 'silver',
    memberCount: 203,
    reputation: 87,
    createdAt: '2024-03-08',
    logo: '/guild logo.jpg'
  },
  {
    id: 'guild-4',
    name: 'DeFi Pioneers',
    description: 'Creating innovative decentralized finance solutions',
    tier: 'bronze',
    memberCount: 45,
    reputation: 76,
    createdAt: '2024-05-12',
    logo: '/guild logo.jpg'
  }
]

// Mock Bounty Data
export const mockBounties: Bounty[] = [
  {
    id: 'bounty-1',
    title: 'Build NFT Marketplace Integration',
    description: 'Create a seamless integration for NFT trading within our platform',
    reward: {
      amount: 5000,
      currency: 'XLM'
    },
    deadline: '2024-12-15',
    status: 'open',
    guildId: 'guild-1',
    createdAt: '2024-10-01'
  },
  {
    id: 'bounty-2',
    title: 'Security Audit Smart Contracts',
    description: 'Perform comprehensive security audit of our core smart contracts',
    reward: {
      amount: 10000,
      currency: 'XLM'
    },
    deadline: '2024-11-30',
    status: 'in-progress',
    guildId: 'guild-2',
    createdAt: '2024-09-15'
  },
  {
    id: 'bounty-3',
    title: 'Mobile Wallet Integration',
    description: 'Develop mobile wallet integration for iOS and Android platforms',
    reward: {
      amount: 7500,
      currency: 'XLM'
    },
    deadline: '2024-12-20',
    status: 'open',
    guildId: 'guild-3',
    createdAt: '2024-10-10'
  },
  {
    id: 'bounty-4',
    title: 'Governance Dashboard',
    description: 'Create intuitive dashboard for guild governance and voting',
    reward: {
      amount: 3000,
      currency: 'XLM'
    },
    deadline: '2024-11-10',
    status: 'completed',
    guildId: 'guild-4',
    createdAt: '2024-08-20'
  }
]

// Mock User Profiles
export const mockProfiles: UserProfile[] = [
  {
    id: 'user-1',
    username: 'stellar_dev',
    email: 'dev@stellarguilds.io',
    avatar: '/avatars/user1.png',
    reputation: 92,
    joinedAt: '2023-12-01',
    guilds: ['guild-1', 'guild-2']
  },
  {
    id: 'user-2',
    username: 'cosmic_builder',
    email: 'builder@stellarguilds.io',
    avatar: '/avatars/user2.png',
    reputation: 88,
    joinedAt: '2024-02-15',
    guilds: ['guild-1', 'guild-3']
  },
  {
    id: 'user-3',
    username: 'blockchain_master',
    email: 'master@stellarguilds.io',
    avatar: '/avatars/user3.png',
    reputation: 95,
    joinedAt: '2023-10-10',
    guilds: ['guild-2', 'guild-4']
  }
]

// Helper functions for generating mock data
export const generateRandomGuild = (): Guild => {
  const tiers: Guild['tier'][] = ['bronze', 'silver', 'gold', 'platinum']
  const adjectives = ['Cosmic', 'Quantum', 'Stellar', 'Galactic', 'Nebula', 'Orion']
  const nouns = ['Explorers', 'Builders', 'Masters', 'Pioneers', 'Guardians', 'Navigators']
  
  return {
    id: `guild-${Math.random().toString(36).substr(2, 9)}`,
    name: `${adjectives[Math.floor(Math.random() * adjectives.length)]} ${nouns[Math.floor(Math.random() * nouns.length)]}`,
    description: 'Generated mock guild description',
    tier: tiers[Math.floor(Math.random() * tiers.length)],
    memberCount: Math.floor(Math.random() * 300) + 10,
    reputation: Math.floor(Math.random() * 100),
    createdAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    logo: '/guild logo.jpg'
  }
}

export const generateRandomBounty = (guildId: string): Bounty => {
  const statuses: Bounty['status'][] = ['open', 'in-progress', 'completed']
  const titles = [
    'Fix Critical Bug in Trading Interface',
    'Optimize Database Queries for Performance',
    'Implement Multi-Signature Wallet Support',
    'Add Dark Mode to Mobile Application',
    'Create Analytics Dashboard for Guild Metrics'
  ]
  
  return {
    id: `bounty-${Math.random().toString(36).substr(2, 9)}`,
    title: titles[Math.floor(Math.random() * titles.length)],
    description: 'Generated mock bounty description',
    reward: {
      amount: Math.floor(Math.random() * 10000) + 1000,
      currency: 'XLM'
    },
    deadline: new Date(Date.now() + Math.random() * 90 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    status: statuses[Math.floor(Math.random() * statuses.length)],
    guildId,
    createdAt: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
  }
}