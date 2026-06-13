import { GuildDetail, GuildMember, GuildActivity } from '@/features/guilds/types'

export const mockGuildMembers: Record<string, GuildMember[]> = {
  'guild-1': [
    {
      id: 'member-1',
      username: 'stellar_dev',
      avatar: '/avatars/user1.png',
      role: 'owner',
      joinedAt: '2024-01-15',
      reputation: 92
    },
    {
      id: 'member-2',
      username: 'cosmic_builder',
      avatar: '/avatars/user2.png',
      role: 'admin',
      joinedAt: '2024-02-20',
      reputation: 88
    },
    {
      id: 'member-3',
      username: 'space_navigator',
      avatar: '/avatars/user3.png',
      role: 'member',
      joinedAt: '2024-03-10',
      reputation: 75
    },
    {
      id: 'member-4',
      username: 'quantum_coder',
      avatar: '/avatars/user4.png',
      role: 'member',
      joinedAt: '2024-04-05',
      reputation: 82
    }
  ],
  'guild-2': [
    {
      id: 'member-5',
      username: 'blockchain_master',
      avatar: '/avatars/user3.png',
      role: 'owner',
      joinedAt: '2023-11-22',
      reputation: 95
    },
    {
      id: 'member-6',
      username: 'contract_king',
      avatar: '/avatars/user5.png',
      role: 'admin',
      joinedAt: '2024-01-10',
      reputation: 90
    },
    {
      id: 'member-7',
      username: 'defi_expert',
      avatar: '/avatars/user6.png',
      role: 'member',
      joinedAt: '2024-02-15',
      reputation: 78
    }
  ],
  'guild-3': [
    {
      id: 'member-8',
      username: 'soroban_sage',
      avatar: '/avatars/user7.png',
      role: 'owner',
      joinedAt: '2024-03-08',
      reputation: 91
    },
    {
      id: 'member-9',
      username: 'smart_contract_pro',
      avatar: '/avatars/user8.png',
      role: 'admin',
      joinedAt: '2024-03-15',
      reputation: 85
    }
  ],
  'guild-4': [
    {
      id: 'member-10',
      username: 'defi_pioneer',
      avatar: '/avatars/user9.png',
      role: 'owner',
      joinedAt: '2024-05-12',
      reputation: 88
    }
  ]
}

export const mockGuildActivities: Record<string, GuildActivity[]> = {
  'guild-1': [
    {
      id: 'activity-1',
      type: 'bounty_completed',
      description: 'NFT Marketplace Integration bounty completed',
      userId: 'member-2',
      username: 'cosmic_builder',
      timestamp: '2024-06-15T10:30:00Z'
    },
    {
      id: 'activity-2',
      type: 'member_joined',
      description: 'quantum_coder joined the guild',
      userId: 'member-4',
      username: 'quantum_coder',
      timestamp: '2024-04-05T14:20:00Z'
    },
    {
      id: 'activity-3',
      type: 'role_changed',
      description: 'cosmic_builder promoted to Admin',
      userId: 'member-2',
      username: 'cosmic_builder',
      timestamp: '2024-03-01T09:15:00Z'
    },
    {
      id: 'activity-4',
      type: 'bounty_created',
      description: 'New bounty: Build NFT Marketplace Integration',
      userId: 'member-1',
      username: 'stellar_dev',
      timestamp: '2024-02-10T11:00:00Z'
    },
    {
      id: 'activity-5',
      type: 'member_joined',
      description: 'space_navigator joined the guild',
      userId: 'member-3',
      username: 'space_navigator',
      timestamp: '2024-03-10T16:45:00Z'
    }
  ],
  'guild-2': [
    {
      id: 'activity-6',
      type: 'bounty_created',
      description: 'New bounty: Security Audit Smart Contracts',
      userId: 'member-5',
      username: 'blockchain_master',
      timestamp: '2024-05-20T10:00:00Z'
    },
    {
      id: 'activity-7',
      type: 'member_joined',
      description: 'defi_expert joined the guild',
      userId: 'member-7',
      username: 'defi_expert',
      timestamp: '2024-02-15T13:30:00Z'
    }
  ],
  'guild-3': [
    {
      id: 'activity-8',
      type: 'role_changed',
      description: 'smart_contract_pro promoted to Admin',
      userId: 'member-9',
      username: 'smart_contract_pro',
      timestamp: '2024-04-01T12:00:00Z'
    }
  ],
  'guild-4': [
    {
      id: 'activity-9',
      type: 'bounty_completed',
      description: 'Governance Dashboard bounty completed',
      userId: 'member-10',
      username: 'defi_pioneer',
      timestamp: '2024-06-10T15:30:00Z'
    }
  ]
}

export const mockGuildDetails: GuildDetail[] = [
  {
    id: 'guild-1',
    name: 'Cosmic Explorers',
    description: 'Exploring the far reaches of the Stellar network. We focus on innovation, collaboration, and pushing the boundaries of blockchain technology.',
    tier: 'gold',
    memberCount: 127,
    reputation: 95,
    createdAt: '2024-01-15',
    logo: '/guild logo.jpg',
    category: 'Development',
    members: mockGuildMembers['guild-1'],
    activities: mockGuildActivities['guild-1']
  },
  {
    id: 'guild-2',
    name: 'Blockchain Builders',
    description: 'Building the future of decentralized applications. Join us in creating robust and scalable blockchain solutions.',
    tier: 'platinum',
    memberCount: 89,
    reputation: 98,
    createdAt: '2023-11-22',
    logo: '/guild logo.jpg',
    category: 'Development',
    members: mockGuildMembers['guild-2'],
    activities: mockGuildActivities['guild-2']
  },
  {
    id: 'guild-3',
    name: 'Smart Contract Masters',
    description: 'Mastering Soroban smart contract development. Expert guidance for developers at all levels.',
    tier: 'silver',
    memberCount: 203,
    reputation: 87,
    createdAt: '2024-03-08',
    logo: '/guild logo.jpg',
    category: 'Education',
    members: mockGuildMembers['guild-3'],
    activities: mockGuildActivities['guild-3']
  },
  {
    id: 'guild-4',
    name: 'DeFi Pioneers',
    description: 'Creating innovative decentralized finance solutions. Shape the future of finance with us.',
    tier: 'bronze',
    memberCount: 45,
    reputation: 76,
    createdAt: '2024-05-12',
    logo: '/guild logo.jpg',
    category: 'DeFi',
    members: mockGuildMembers['guild-4'],
    activities: mockGuildActivities['guild-4']
  }
]
