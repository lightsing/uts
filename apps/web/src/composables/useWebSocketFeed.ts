import { ref, onUnmounted } from 'vue'
import { BrowserProvider } from 'ethers'
import type { Eip1193Provider } from 'ethers'
import { SDK } from '@uts/sdk'
import { getSDK } from './useTimestampSDK'

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  54352: 'Scroll',
  54351: 'Scroll Sepolia',
}

export interface FeedEntry {
  id: string
  hash: string
  type: 'ethereum'
  chain: string
  chainId: number
  blockHeight: number
  sender?: string
  timestamp: number
}

export function useWebSocketFeed() {
  const entries = ref<FeedEntry[]>([])
  const isConnected = ref(false)
  let intervalId: ReturnType<typeof setInterval> | null = null
  let lastBlock: Record<number, number> = {}

  async function fetchEventsFromWeb3(web3Provider: Eip1193Provider) {
    try {
      const provider = new BrowserProvider(web3Provider)
      const network = await provider.getNetwork()
      const chainId = Number(network.chainId)

      const currentBlock = await provider.getBlockNumber()
      const fromBlock = lastBlock[chainId]
        ? lastBlock[chainId] + 1
        : currentBlock - 10

      if (fromBlock > currentBlock) return

      const logs = await provider.getLogs({
        fromBlock,
        toBlock: currentBlock,
        topics: [SDK.utsLogTopic],
      })

      for (const log of logs) {
        const parsed = SDK.utsInterface.parseLog(log)
        if (!parsed) continue

        const root: string = parsed.args[0]
        const sender: string = parsed.args[1]
        const ts: bigint = parsed.args[2]

        const entryId = `${chainId}-${log.blockNumber}-${log.index}`
        if (!entries.value.some((e) => e.id === entryId)) {
          entries.value.unshift({
            id: entryId,
            hash: root,
            type: 'ethereum',
            chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
            chainId,
            blockHeight: log.blockNumber,
            sender,
            timestamp: Number(ts) * 1000,
          })
        }
      }

      lastBlock[chainId] = currentBlock

      if (entries.value.length > 50) {
        entries.value.length = 50
      }
    } catch (e) {
      console.warn('Feed: failed to poll web3Provider:', e)
    }
  }

  async function fetchEventsFromRPCs(sdk: SDK) {
    const chainIds = Object.keys(sdk.ethRPCs).map(Number)

    for (const chainId of chainIds) {
      const provider = sdk.getEthProvider(chainId)
      if (!provider) continue

      try {
        const currentBlock = await provider.getBlockNumber()
        const fromBlock = lastBlock[chainId]
          ? lastBlock[chainId] + 1
          : currentBlock - 5

        if (fromBlock > currentBlock) continue

        const logs = await provider.getLogs({
          fromBlock,
          toBlock: currentBlock,
          topics: [SDK.utsLogTopic],
        })

        for (const log of logs) {
          const parsed = SDK.utsInterface.parseLog(log)
          if (!parsed) continue

          const root: string = parsed.args[0]
          const sender: string = parsed.args[1]
          const ts: bigint = parsed.args[2]

          entries.value.unshift({
            id: `${chainId}-${log.blockNumber}-${log.index}`,
            hash: root,
            type: 'ethereum',
            chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
            chainId,
            blockHeight: log.blockNumber,
            sender,
            timestamp: Number(ts) * 1000,
          })
        }

        lastBlock[chainId] = currentBlock

        if (entries.value.length > 50) {
          entries.value.length = 50
        }
      } catch (e) {
        console.warn(`Feed: failed to poll chain ${chainId}:`, e)
      }
    }
  }

  async function connect() {
    const sdk = getSDK()
    isConnected.value = true
    lastBlock = {}

    // Initial fetch: prefer web3Provider
    if (sdk.web3Provider) {
      await fetchEventsFromWeb3(sdk.web3Provider)
    } else {
      await fetchEventsFromRPCs(sdk)
    }

    // Poll every 15s — prefer web3Provider when available
    intervalId = setInterval(() => {
      const currentSdk = getSDK()
      if (currentSdk.web3Provider) {
        fetchEventsFromWeb3(currentSdk.web3Provider)
      } else {
        fetchEventsFromRPCs(currentSdk)
      }
    }, 15000)
  }

  function disconnect() {
    isConnected.value = false
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  onUnmounted(() => {
    disconnect()
  })

  return {
    entries,
    isConnected,
    connect,
    disconnect,
  }
}
