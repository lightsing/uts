import { ref, onUnmounted } from 'vue'
import { createPublicClient, custom, parseAbiItem } from 'viem'
import type { EIP1193Provider } from '@uts/sdk'
import { SDK } from '@uts/sdk'
import { getSDK } from './useTimestampSDK'

const UTS_ATTESTED_EVENT = parseAbiItem(
  'event Attested(bytes32 indexed hash, address indexed sender, uint256 timestamp)',
)

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
    web3Provider: EIP1193Provider,
  ): Promise<number | null> {
    try {
      const client = createPublicClient({ transport: custom(web3Provider) })
      const chainId = await client.getChainId()

      const currentBlock = Number(await client.getBlockNumber())
      const fromBlock = lastBlockWeb3[chainId]
        ? lastBlockWeb3[chainId] + 1
        : currentBlock - 10

      if (fromBlock > currentBlock) return chainId

      const logs = await client.getLogs({
        fromBlock: BigInt(fromBlock),
        toBlock: BigInt(currentBlock),
        event: UTS_ATTESTED_EVENT,
      })

      for (const log of logs) {
        addEntry({
          id: `${chainId}-${Number(log.blockNumber)}-${log.logIndex}`,
          hash: log.args.hash!,
          type: 'ethereum',
          chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
          chainId,
          blockHeight: Number(log.blockNumber),
          sender: log.args.sender!,
          txHash: log.transactionHash!,
          timestamp: Number(log.args.timestamp!) * 1000,
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

      const client = sdk.getEthProvider(chainId)
      if (!client) continue

      try {
        const currentBlock = Number(await client.getBlockNumber())
        const fromBlock = lastBlockRPC[chainId]
          ? lastBlockRPC[chainId] + 1
          : currentBlock - 5

        if (fromBlock > currentBlock) continue

        const logs = await client.getLogs({
          fromBlock: BigInt(fromBlock),
          toBlock: BigInt(currentBlock),
          event: UTS_ATTESTED_EVENT,
        })

        for (const log of logs) {
          addEntry({
            id: `${chainId}-${Number(log.blockNumber)}-${log.logIndex}`,
            hash: log.args.hash!,
            type: 'ethereum',
            chain: CHAIN_NAMES[chainId] ?? `Chain ${chainId}`,
            chainId,
            blockHeight: Number(log.blockNumber),
            sender: log.args.sender!,
            txHash: log.transactionHash!,
            timestamp: Number(log.args.timestamp!) * 1000,
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
