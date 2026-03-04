import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { DEFAULT_CALENDARS } from '@uts/sdk'
import type { DetachedTimestamp, SecureDigestOp } from '@uts/sdk'
import { getSDK, resetSDK } from '@/composables/useTimestampSDK'
import { JsonRpcProvider } from 'ethers'

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  534352: 'Scroll',
  534351: 'Scroll Sepolia',
}

export interface EthChainNode {
  chainId: number
  name: string
  status: 'online' | 'offline' | 'checking'
  latency?: number
  rpcUrl?: string
}

const STORAGE_KEY = 'uts-calendars'
const SETTINGS_KEY = 'uts-settings'
const CHAINS_KEY = 'uts-custom-chains'
const CHAIN_RPCS_KEY = 'uts-chain-rpcs'

// Default chain IDs from SDK ethRPCs
function getDefaultChainIds(): number[] {
  return Object.keys(getSDK().ethRPCs).map(Number)
}

function loadCalendars(): string[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) return JSON.parse(stored)
  } catch {
    /* ignore */
  }
  return DEFAULT_CALENDARS.map((u) => u.toString())
}

function saveCalendars(urls: string[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(urls))
}

function loadSettings(): {
  keepPending: boolean
  internalHashAlgo: SecureDigestOp
} {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      return {
        keepPending: parsed.keepPending ?? false,
        internalHashAlgo: parsed.internalHashAlgo ?? 'KECCAK256',
      }
    }
  } catch {
    /* ignore */
  }
  return { keepPending: false, internalHashAlgo: 'KECCAK256' }
}

function saveSettings(settings: {
  keepPending: boolean
  internalHashAlgo: SecureDigestOp
}) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

function loadCustomChains(): number[] | null {
  try {
    const stored = localStorage.getItem(CHAINS_KEY)
    if (stored) return JSON.parse(stored)
  } catch {
    /* ignore */
  }
  return null
}

function saveCustomChains(chainIds: number[]) {
  localStorage.setItem(CHAINS_KEY, JSON.stringify(chainIds))
}

function loadChainRpcs(): Record<number, string> {
  try {
    const stored = localStorage.getItem(CHAIN_RPCS_KEY)
    if (stored) return JSON.parse(stored)
  } catch {
    /* ignore */
  }
  return {}
}

function saveChainRpcs(rpcs: Record<number, string>) {
  localStorage.setItem(CHAIN_RPCS_KEY, JSON.stringify(rpcs))
}

// Public RPC endpoints for common chains (used when user adds a chain not in SDK ethRPCs)
const PUBLIC_RPCS: Record<number, string> = {
  1: 'https://eth.llamarpc.com',
  17000: 'https://rpc.holesky.ethpandaops.io',
  11155111: 'https://rpc.sepolia.org',
  534352: 'https://rpc.scroll.io',
  534351: 'https://sepolia-rpc.scroll.io',
  42161: 'https://arb1.arbitrum.io/rpc',
  10: 'https://mainnet.optimism.io',
  8453: 'https://mainnet.base.org',
  137: 'https://polygon-rpc.com',
}

export const useAppStore = defineStore('app', () => {
  const calendarUrls = ref<string[]>(loadCalendars())
  const keepPending = ref(loadSettings().keepPending)
  const internalHashAlgo = ref<SecureDigestOp>(loadSettings().internalHashAlgo)
  const ethChains = ref<EthChainNode[]>([])
  const recentStamps = ref<DetachedTimestamp[]>([])

  // Chain IDs to monitor (persisted or default from SDK)
  const customChainIds = ref<number[]>(
    loadCustomChains() ?? getDefaultChainIds(),
  )

  // Custom RPC endpoints per chain (persisted)
  const customRpcs = ref<Record<number, string>>(loadChainRpcs())

  const onlineCount = computed(
    () => ethChains.value.filter((c) => c.status === 'online').length,
  )

  watch(
    calendarUrls,
    (urls) => {
      saveCalendars(urls)
      resetSDK({ calendars: urls.map((u) => new URL(u)) })
    },
    { deep: true },
  )

  watch(keepPending, (val) => {
    saveSettings({ keepPending: val, internalHashAlgo: internalHashAlgo.value })
  })

  watch(internalHashAlgo, (val) => {
    saveSettings({ keepPending: keepPending.value, internalHashAlgo: val })
  })

  async function checkChains() {
    const sdk = getSDK()
    const chainIds = customChainIds.value

    ethChains.value = chainIds.map((chainId) => {
      const rpc = customRpcs.value[chainId] ?? PUBLIC_RPCS[chainId]
      return {
        chainId,
        name: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
        status: 'checking' as const,
        rpcUrl: rpc,
      }
    })

    for (const chain of ethChains.value) {
      // Try custom RPC first, then SDK provider, then public RPC fallback
      let provider: JsonRpcProvider | null = null
      const rpc = customRpcs.value[chain.chainId]
      if (rpc) {
        try {
          provider = new JsonRpcProvider(rpc)
        } catch {
          /* ignore */
        }
      }
      if (!provider) {
        provider = sdk.getEthProvider(chain.chainId)
      }
      if (!provider && PUBLIC_RPCS[chain.chainId]) {
        try {
          provider = new JsonRpcProvider(PUBLIC_RPCS[chain.chainId])
        } catch {
          /* ignore */
        }
      }

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

  function addChain(chainId: number) {
    if (customChainIds.value.includes(chainId)) return
    customChainIds.value.push(chainId)
    saveCustomChains(customChainIds.value)
  }

  function removeChain(chainId: number) {
    customChainIds.value = customChainIds.value.filter((id) => id !== chainId)
    ethChains.value = ethChains.value.filter((c) => c.chainId !== chainId)
    saveCustomChains(customChainIds.value)
  }

  function resetChains() {
    customChainIds.value = getDefaultChainIds()
    customRpcs.value = {}
    localStorage.removeItem(CHAINS_KEY)
    localStorage.removeItem(CHAIN_RPCS_KEY)
  }

  function setChainRpc(chainId: number, rpcUrl: string) {
    const trimmed = rpcUrl.trim()
    if (trimmed) {
      customRpcs.value[chainId] = trimmed
    } else {
      delete customRpcs.value[chainId]
    }
    saveChainRpcs(customRpcs.value)
    // Update the rpcUrl in the displayed chain list
    const chain = ethChains.value.find((c) => c.chainId === chainId)
    if (chain) {
      chain.rpcUrl = trimmed || PUBLIC_RPCS[chainId]
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
    internalHashAlgo,
    ethChains,
    recentStamps,
    onlineCount,
    checkChains,
    addChain,
    removeChain,
    resetChains,
    setChainRpc,
    addStamp,
    setCalendars,
    resetCalendars,
  }
})
