import { create } from 'zustand'
import { GuildDetail, GuildMember, GuildActivity, CreateGuildFormData, GuildRole } from '@/features/guilds/types'
import { mockGuildDetails } from '@/lib/mocks/guilds'

interface GuildState {
  guilds: GuildDetail[]
  currentGuild: GuildDetail | null
  isLoading: boolean
  error: string | null

  // Actions
  fetchGuilds: () => void
  fetchGuildById: (id: string) => void
  createGuild: (data: CreateGuildFormData) => Promise<string>
  updateGuild: (id: string, data: Partial<CreateGuildFormData>) => void
  addMember: (guildId: string, member: GuildMember) => void
  removeMember: (guildId: string, memberId: string) => void
  changeMemberRole: (guildId: string, memberId: string, newRole: GuildRole) => void
  addActivity: (guildId: string, activity: GuildActivity) => void
}

export const useGuildStore = create<GuildState>((set, get) => ({
  guilds: mockGuildDetails,
  currentGuild: null,
  isLoading: false,
  error: null,

  fetchGuilds: () => {
    set({ isLoading: true, error: null })
    // Simulate API call
    setTimeout(() => {
      set({ guilds: mockGuildDetails, isLoading: false })
    }, 300)
  },

  fetchGuildById: (id: string) => {
    set({ isLoading: true, error: null })
    // Simulate API call
    setTimeout(() => {
      const guild = get().guilds.find((g) => g.id === id)
      if (guild) {
        set({ currentGuild: guild, isLoading: false })
      } else {
        set({ error: 'Guild not found', isLoading: false })
      }
    }, 300)
  },

  createGuild: async (data: CreateGuildFormData): Promise<string> => {
    set({ isLoading: true, error: null })

    // Simulate API call
    return new Promise((resolve) => {
      setTimeout(() => {
        const newGuild: GuildDetail = {
          id: `guild-${Date.now()}`,
          name: data.name,
          description: data.description,
          tier: 'bronze',
          memberCount: 1,
          reputation: 0,
          createdAt: new Date().toISOString().split('T')[0],
          logo: data.logo || '/guild logo.jpg',
          category: data.category,
          members: [
            {
              id: 'current-user',
              username: 'You',
              role: 'owner',
              joinedAt: new Date().toISOString().split('T')[0],
              reputation: 0
            }
          ],
          activities: [
            {
              id: `activity-${Date.now()}`,
              type: 'member_joined',
              description: 'Guild created',
              timestamp: new Date().toISOString()
            }
          ]
        }

        set((state) => ({
          guilds: [...state.guilds, newGuild],
          isLoading: false
        }))

        resolve(newGuild.id)
      }, 500)
    })
  },

  updateGuild: (id: string, data: Partial<CreateGuildFormData>) => {
    set((state) => ({
      guilds: state.guilds.map((guild) =>
        guild.id === id
          ? {
              ...guild,
              name: data.name || guild.name,
              description: data.description || guild.description,
              logo: data.logo || guild.logo,
              category: data.category || guild.category
            }
          : guild
      ),
      currentGuild:
        state.currentGuild?.id === id
          ? {
              ...state.currentGuild,
              name: data.name || state.currentGuild.name,
              description: data.description || state.currentGuild.description,
              logo: data.logo || state.currentGuild.logo,
              category: data.category || state.currentGuild.category
            }
          : state.currentGuild
    }))
  },

  addMember: (guildId: string, member: GuildMember) => {
    set((state) => ({
      guilds: state.guilds.map((guild) =>
        guild.id === guildId
          ? {
              ...guild,
              members: [...guild.members, member],
              memberCount: guild.memberCount + 1
            }
          : guild
      ),
      currentGuild:
        state.currentGuild?.id === guildId
          ? {
              ...state.currentGuild,
              members: [...state.currentGuild.members, member],
              memberCount: state.currentGuild.memberCount + 1
            }
          : state.currentGuild
    }))

    // Add activity
    get().addActivity(guildId, {
      id: `activity-${Date.now()}`,
      type: 'member_joined',
      description: `${member.username} joined the guild`,
      userId: member.id,
      username: member.username,
      timestamp: new Date().toISOString()
    })
  },

  removeMember: (guildId: string, memberId: string) => {
    set((state) => ({
      guilds: state.guilds.map((guild) =>
        guild.id === guildId
          ? {
              ...guild,
              members: guild.members.filter((m) => m.id !== memberId),
              memberCount: guild.memberCount - 1
            }
          : guild
      ),
      currentGuild:
        state.currentGuild?.id === guildId
          ? {
              ...state.currentGuild,
              members: state.currentGuild.members.filter((m) => m.id !== memberId),
              memberCount: state.currentGuild.memberCount - 1
            }
          : state.currentGuild
    }))
  },

  changeMemberRole: (guildId: string, memberId: string, newRole: GuildRole) => {
    set((state) => ({
      guilds: state.guilds.map((guild) =>
        guild.id === guildId
          ? {
              ...guild,
              members: guild.members.map((m) =>
                m.id === memberId ? { ...m, role: newRole } : m
              )
            }
          : guild
      ),
      currentGuild:
        state.currentGuild?.id === guildId
          ? {
              ...state.currentGuild,
              members: state.currentGuild.members.map((m) =>
                m.id === memberId ? { ...m, role: newRole } : m
              )
            }
          : state.currentGuild
    }))

    // Add activity
    const member = get().currentGuild?.members.find((m) => m.id === memberId)
    if (member) {
      get().addActivity(guildId, {
        id: `activity-${Date.now()}`,
        type: 'role_changed',
        description: `${member.username} role changed to ${newRole}`,
        userId: member.id,
        username: member.username,
        timestamp: new Date().toISOString()
      })
    }
  },

  addActivity: (guildId: string, activity: GuildActivity) => {
    set((state) => ({
      guilds: state.guilds.map((guild) =>
        guild.id === guildId
          ? {
              ...guild,
              activities: [activity, ...guild.activities]
            }
          : guild
      ),
      currentGuild:
        state.currentGuild?.id === guildId
          ? {
              ...state.currentGuild,
              activities: [activity, ...state.currentGuild.activities]
            }
          : state.currentGuild
    }))
  }
}))
