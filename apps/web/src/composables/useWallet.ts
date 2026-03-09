import { ref, computed } from 'vue'
import type { EIP1193Provider } from '@uts/sdk'
import { WELL_KNOWN_CHAINS } from '@uts/sdk'

const walletAddress = ref<string | null>(null)
const walletChainId = ref<number | null>(null)
const isConnecting = ref(false)
const walletError = ref<string | null>(null)

const isConnected = computed(() => walletAddress.value !== null)
const hasWallet = computed(
  () => typeof window !== 'undefined' && !!(window as any).ethereum,
)

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  534352: 'Scroll',
  534351: 'Scroll Sepolia',
}

const walletChainName = computed(() => {
  if (!walletChainId.value) return null
  return CHAIN_NAMES[walletChainId.value] ?? `Chain ${walletChainId.value}`
})

function getEip1193Provider(): EIP1193Provider | null {
  if (typeof window === 'undefined') return null
  return (window as any).ethereum ?? null
}

function truncateAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

export function useWallet() {
  async function connect() {
    const ethereum = getEip1193Provider()
    if (!ethereum) {
      walletError.value =
        'No wallet detected. Install MetaMask or another EIP-1193 wallet.'
      return
    }

    isConnecting.value = true
    walletError.value = null

    try {
      const accounts = (await ethereum.request({
        method: 'eth_requestAccounts',
      })) as string[]
      if (accounts.length === 0) {
        walletError.value = 'No accounts returned from wallet'
        return
      }
      walletAddress.value = accounts[0] ?? null

      const chainIdHex = (await ethereum.request({
        method: 'eth_chainId',
      })) as string
      walletChainId.value = parseInt(chainIdHex, 16)

      // Listen for account/chain changes
      ;(ethereum as any).on?.('accountsChanged', handleAccountsChanged)
      ;(ethereum as any).on?.('chainChanged', handleChainChanged)
    } catch (e) {
      walletError.value =
        e instanceof Error ? e.message : 'Failed to connect wallet'
    } finally {
      isConnecting.value = false
    }
  }

  function disconnect() {
    const ethereum = getEip1193Provider()
    if (ethereum) {
      ;(ethereum as any).removeListener?.(
        'accountsChanged',
        handleAccountsChanged,
      )
      ;(ethereum as any).removeListener?.('chainChanged', handleChainChanged)
    }
    walletAddress.value = null
    walletChainId.value = null
    walletError.value = null
  }

  async function switchChain(chainId: number) {
    const ethereum = getEip1193Provider()
    if (!ethereum) return

    const knownChain = WELL_KNOWN_CHAINS[chainId]
    if (!knownChain) return

    try {
      await ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: knownChain.chainId }],
      })
    } catch {
      // switch failed
    }
  }

  return {
    walletAddress,
    walletChainId,
    walletChainName,
    isConnected,
    isConnecting,
    hasWallet,
    walletError,
    connect,
    disconnect,
    switchChain,
    getEip1193Provider,
    truncateAddress,
  }
}

function handleAccountsChanged(accounts: string[]) {
  if (accounts.length === 0) {
    walletAddress.value = null
    walletChainId.value = null
  } else {
    walletAddress.value = accounts[0] ?? null
  }
}

function handleChainChanged(chainIdHex: string) {
  walletChainId.value = parseInt(chainIdHex, 16)
}
