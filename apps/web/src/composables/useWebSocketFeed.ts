import { ref, onUnmounted } from 'vue'
import { BrowserProvider } from 'ethers'
import type { Eip1193Provider } from 'ethers'
import { SDK } from '@uts/sdk'
import { getSDK } from './useTimestampSDK'

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  534352: 'Scroll',
  534351: 'Scroll Sepolia',
}

export interface FeedEntry {
  id: string
  hash: string
  type: 'ethereum'
  chain: string
  chainId: number
  blockHeight: number
  sender?: string
  txHash?: string
  timestamp: number
}

export function useWebSocketFeed() {
  const entries = ref<FeedEntry[]>([])
  const isConnected = ref(false)
  const seenIds = new Set<string>()
  let intervalId: ReturnType<typeof setInterval> | null = null
  let lastBlockWeb3: Record<number, number> = {}
  let lastBlockRPC: Record<number, number> = {}

  function addEntry(entry: FeedEntry) {
    if (seenIds.has(entry.id)) return
    seenIds.add(entry.id)
    entries.value.unshift(entry)
    if (entries.value.length > 50) {
      const removed = entries.value.splice(50)
      for (const r of removed) seenIds.delete(r.id)
    }
  }

  /** Poll web3Provider. Returns the wallet's chainId on success, or null. */
  async function fetchEventsFromWeb3(
    web3Provider: Eip1193Provider,
  ): Promise<number | null> {
    try {
      const provider = new BrowserProvider(web3Provider)
      const network = await provider.getNetwork()
      const chainId = Number(network.chainId)

      const currentBlock = await provider.getBlockNumber()
      const fromBlock = lastBlockWeb3[chainId]
        ? lastBlockWeb3[chainId] + 1
        : currentBlock - 10

      if (fromBlock > currentBlock) return chainId

      const logs = await provider.getLogs({
        fromBlock,
        toBlock: currentBlock,
        topics: [SDK.utsLogTopic],
      })

      for (const log of logs) {
        const parsed = SDK.utsInterface.parseLog(log)
        if (!parsed) continue

        addEntry({
          id: `${chainId}-${log.blockNumber}-${log.index}`,
          hash: parsed.args[0],
          type: 'ethereum',
          chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
          chainId,
          blockHeight: log.blockNumber,
          sender: parsed.args[1],
          txHash: log.transactionHash,
          timestamp: Number(parsed.args[2] as bigint) * 1000,
        })
      }

      lastBlockWeb3[chainId] = currentBlock
      return chainId
    } catch (e) {
      console.warn('Feed: failed to poll web3Provider:', e)
      return null
    }
  }

  /** Poll ethRPCs, skipping chains already covered by web3Provider. */
  async function fetchEventsFromRPCs(
    sdk: SDK,
    skipChainIds: Set<number> = new Set(),
  ) {
    const chainIds = Object.keys(sdk.ethRPCs).map(Number)

    for (const chainId of chainIds) {
      if (skipChainIds.has(chainId)) continue

      const provider = sdk.getEthProvider(chainId)
      if (!provider) continue

      try {
        const currentBlock = await provider.getBlockNumber()
        const fromBlock = lastBlockRPC[chainId]
          ? lastBlockRPC[chainId] + 1
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

          addEntry({
            id: `${chainId}-${log.blockNumber}-${log.index}`,
            hash: parsed.args[0],
            type: 'ethereum',
            chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
            chainId,
            blockHeight: log.blockNumber,
            sender: parsed.args[1],
            txHash: log.transactionHash,
            timestamp: Number(parsed.args[2] as bigint) * 1000,
          })
        }

        lastBlockRPC[chainId] = currentBlock
      } catch (e) {
        console.warn(`Feed: failed to poll chain ${chainId}:`, e)
      }
    }
  }

  /** Poll both web3Provider and ethRPCs, deduplicating by entry id. */
  async function pollAll() {
    const sdk = getSDK()
    const skipChainIds = new Set<number>()

    // Poll web3Provider first (if available)
    if (sdk.web3Provider) {
      const web3ChainId = await fetchEventsFromWeb3(sdk.web3Provider)
      if (web3ChainId !== null) {
        // Skip this chain in ethRPCs since web3Provider already covers it
        skipChainIds.add(web3ChainId)
      }
    }

    // Also poll ethRPCs for all remaining chains
    await fetchEventsFromRPCs(sdk, skipChainIds)
  }

  async function connect() {
    isConnected.value = true
    lastBlockWeb3 = {}
    lastBlockRPC = {}

    await pollAll()

    intervalId = setInterval(() => {
      pollAll()
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
