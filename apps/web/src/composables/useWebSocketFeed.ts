import { ref, onUnmounted } from 'vue'

export interface FeedEntry {
  id: string
  hash: string
  type: 'bitcoin' | 'ethereum' | 'pending'
  chain?: string
  blockHeight?: number
  timestamp: number
}

export function useWebSocketFeed() {
  const entries = ref<FeedEntry[]>([])
  const isConnected = ref(false)
  let intervalId: ReturnType<typeof setInterval> | null = null

  const MOCK_CHAINS = ['Bitcoin', 'Ethereum', 'Scroll', 'Sepolia']
  const MOCK_TYPES: FeedEntry['type'][] = ['bitcoin', 'ethereum', 'pending']

  function randomHex(len: number): string {
    const bytes = new Uint8Array(len)
    crypto.getRandomValues(bytes)
    return (
      '0x' +
      Array.from(bytes)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('')
    )
  }

  function generateMockEntry(): FeedEntry {
    const type = MOCK_TYPES[Math.floor(Math.random() * MOCK_TYPES.length)]!
    return {
      id: crypto.randomUUID(),
      hash: randomHex(32),
      type,
      chain: type !== 'pending' ? MOCK_CHAINS[Math.floor(Math.random() * MOCK_CHAINS.length)] : undefined,
      blockHeight:
        type !== 'pending'
          ? Math.floor(Math.random() * 1000000) + 19000000
          : undefined,
      timestamp: Date.now(),
    }
  }

  function connect() {
    isConnected.value = true

    // Seed initial entries
    for (let i = 0; i < 8; i++) {
      entries.value.push(generateMockEntry())
    }

    // Simulate live feed
    intervalId = setInterval(() => {
      entries.value.unshift(generateMockEntry())
      if (entries.value.length > 50) {
        entries.value.pop()
      }
    }, 3000 + Math.random() * 2000)
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
