import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DetachedTimestamp } from '@uts/sdk'
import { getSDK } from '@/composables/useTimestampSDK'

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

export const useAppStore = defineStore('app', () => {
  const ethChains = ref<EthChainNode[]>([])
  const recentStamps = ref<DetachedTimestamp[]>([])

  const onlineCount = computed(
    () => ethChains.value.filter((c) => c.status === 'online').length,
  )

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

  return {
    ethChains,
    recentStamps,
    onlineCount,
    checkChains,
    addStamp,
  }
})
