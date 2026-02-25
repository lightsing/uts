import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { DEFAULT_CALENDARS } from '@uts/sdk'
import type { DetachedTimestamp } from '@uts/sdk'
import { getSDK, resetSDK } from '@/composables/useTimestampSDK'

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  54352: 'Scroll',
  54351: 'Scroll Sepolia',
}

export interface EthChainNode {
  chainId: number
  name: string
  status: 'online' | 'offline' | 'checking'
  latency?: number
}

const STORAGE_KEY = 'uts-calendars'
const SETTINGS_KEY = 'uts-settings'

function loadCalendars(): string[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) return JSON.parse(stored)
  } catch { /* ignore */ }
  return DEFAULT_CALENDARS.map((u) => u.toString())
}

function saveCalendars(urls: string[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(urls))
}

function loadSettings(): { keepPending: boolean } {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY)
    if (stored) return JSON.parse(stored)
  } catch { /* ignore */ }
  return { keepPending: false }
}

function saveSettings(settings: { keepPending: boolean }) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

export const useAppStore = defineStore('app', () => {
  const calendarUrls = ref<string[]>(loadCalendars())
  const keepPending = ref(loadSettings().keepPending)
  const ethChains = ref<EthChainNode[]>([])
  const recentStamps = ref<DetachedTimestamp[]>([])

  const onlineCount = computed(
    () => ethChains.value.filter((c) => c.status === 'online').length,
  )

  watch(calendarUrls, (urls) => {
    saveCalendars(urls)
    resetSDK({ calendars: urls.map((u) => new URL(u)) })
  }, { deep: true })

  watch(keepPending, (val) => {
    saveSettings({ keepPending: val })
  })

  async function checkChains() {
    const sdk = getSDK()
    const chainIds = Object.keys(sdk.ethRPCs).map(Number)

    ethChains.value = chainIds.map((chainId) => ({
      chainId,
      name: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
      status: 'checking' as const,
    }))

    for (const chain of ethChains.value) {
      const provider = sdk.getEthProvider(chain.chainId)
      if (!provider) {
        chain.status = 'offline'
        continue
      }
      const start = performance.now()
      try {
        await provider.getBlockNumber()
        chain.latency = Math.round(performance.now() - start)
        chain.status = 'online'
      } catch {
        chain.status = 'offline'
        chain.latency = undefined
      }
    }
  }

  function addStamp(stamp: DetachedTimestamp) {
    recentStamps.value.unshift(stamp)
    if (recentStamps.value.length > 20) {
      recentStamps.value.pop()
    }
  }

  function setCalendars(urls: string[]) {
    calendarUrls.value = urls
  }

  function resetCalendars() {
    calendarUrls.value = DEFAULT_CALENDARS.map((u) => u.toString())
  }

  return {
    calendarUrls,
    keepPending,
    ethChains,
    recentStamps,
    onlineCount,
    checkChains,
    addStamp,
    setCalendars,
    resetCalendars,
  }
})
