<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { formatDistanceToNow } from 'date-fns'
import { Activity, Globe, Clock, ChevronDown, ChevronUp, ExternalLink, AlertTriangle } from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import { useWebSocketFeed } from '@/composables/useWebSocketFeed'
import ScrollLogo from '@/assets/Scroll_Logomark.svg'

const SCROLL_CHAIN_IDS = new Set([534352, 534351])

const { entries, isConnected, connect } = useWebSocketFeed()

const expandedEntries = ref<Set<string>>(new Set())

onMounted(() => {
  connect()
})

function truncate(hash: string): string {
  return `${hash.slice(0, 10)}...${hash.slice(-8)}`
}

function toggleEntry(id: string) {
  if (expandedEntries.value.has(id)) {
    expandedEntries.value.delete(id)
  } else {
    expandedEntries.value.add(id)
  }
}

const ETHERSCAN_URLS: Record<number, string> = {
  1: 'https://etherscan.io',
  17000: 'https://holesky.etherscan.io',
  11155111: 'https://sepolia.etherscan.io',
  534352: 'https://scrollscan.com',
  534351: 'https://sepolia.scrollscan.com',
}

const MAINNET_CHAIN_IDS = new Set([1, 534352])

function isTestnetOrUnknown(chainId: number): boolean {
  return !MAINNET_CHAIN_IDS.has(chainId)
}

function getBlockUrl(chainId: number, height: number): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/block/${height}`
}

function getAddressUrl(chainId: number, address: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/address/${address}`
}

function getTxUrl(chainId: number, txHash: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/tx/${txHash}`
}
</script>

<template>
  <GlassCard>
    <div class="mb-4 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Activity class="h-4 w-4 text-neon-cyan" />
        <h3 class="font-heading text-sm font-semibold text-white/80">Live Feed</h3>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="h-2 w-2 rounded-full"
          :class="isConnected ? 'bg-valid animate-glow-pulse' : 'bg-invalid'"
        />
        <span class="font-mono text-[10px] text-white/40">
          {{ isConnected ? 'POLLING' : 'DISCONNECTED' }}
        </span>
      </div>
    </div>

    <div class="max-h-80 space-y-1 overflow-y-auto pr-1">
      <TransitionGroup name="list">
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="rounded-lg transition-colors hover:bg-white/5"
        >
          <!-- Entry header -->
          <button
            class="flex w-full items-center gap-3 px-3 py-2 text-left"
            @click="toggleEntry(entry.id)"
          >
            <img v-if="SCROLL_CHAIN_IDS.has(entry.chainId)" :src="ScrollLogo" alt="Scroll" class="h-4 w-4 shrink-0" />
            <Globe v-else class="h-4 w-4 shrink-0 text-neon-purple" />
            <div class="min-w-0 flex-1">
              <div class="font-mono text-xs text-neon-purple">
                {{ truncate(entry.hash) }}
              </div>
              <div class="flex items-center gap-2 font-mono text-[10px] text-white/30">
                <span>{{ entry.chain }}</span>
                <span>#{{ entry.blockHeight }}</span>
                <AlertTriangle
                  v-if="isTestnetOrUnknown(entry.chainId)"
                  class="h-3 w-3 text-pending"
                  :title="ETHERSCAN_URLS[entry.chainId] ? 'Testnet attestation' : 'Unknown network'"
                />
              </div>
            </div>
            <div class="flex shrink-0 items-center gap-1">
              <span class="font-mono text-[10px] text-white/20">
                {{ formatDistanceToNow(entry.timestamp, { addSuffix: true }) }}
              </span>
              <component :is="expandedEntries.has(entry.id) ? ChevronUp : ChevronDown" class="h-3 w-3 text-white/20" />
            </div>
          </button>

          <!-- Expanded details -->
          <Transition name="fade">
            <div v-if="expandedEntries.has(entry.id)" class="border-t border-glass-border/50 px-3 pb-2 pt-1.5">
              <div class="space-y-1 font-mono text-[10px]">
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Root</span>
                  <span class="text-white/50">{{ truncate(entry.hash) }}</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Chain</span>
                  <span class="flex items-center gap-1">
                    <span class="text-white/50">{{ entry.chain }} ({{ entry.chainId }})</span>
                    <AlertTriangle
                      v-if="isTestnetOrUnknown(entry.chainId)"
                      class="h-2.5 w-2.5 text-pending"
                      :title="ETHERSCAN_URLS[entry.chainId] ? 'Testnet' : 'Unknown network'"
                    />
                  </span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Block</span>
                  <span class="flex items-center gap-1">
                    <span class="text-white/50">{{ entry.blockHeight }}</span>
                    <a
                      v-if="getBlockUrl(entry.chainId, entry.blockHeight)"
                      :href="getBlockUrl(entry.chainId, entry.blockHeight)!"
                      target="_blank"
                      rel="noopener"
                      class="text-neon-cyan hover:text-neon-cyan/80"
                    >
                      <ExternalLink class="h-2.5 w-2.5" />
                    </a>
                  </span>
                </div>
                <div v-if="entry.sender" class="flex items-center justify-between">
                  <span class="text-white/30">Sender</span>
                  <span class="flex items-center gap-1">
                    <span class="text-white/50">{{ truncate(entry.sender) }}</span>
                    <a
                      v-if="getAddressUrl(entry.chainId, entry.sender)"
                      :href="getAddressUrl(entry.chainId, entry.sender)!"
                      target="_blank"
                      rel="noopener"
                      class="text-neon-cyan hover:text-neon-cyan/80"
                    >
                      <ExternalLink class="h-2.5 w-2.5" />
                    </a>
                  </span>
                </div>
                <div v-if="entry.txHash" class="flex items-center justify-between">
                  <span class="text-white/30">Tx Hash</span>
                  <span class="flex items-center gap-1">
                    <span class="text-white/50">{{ truncate(entry.txHash) }}</span>
                    <a
                      v-if="getTxUrl(entry.chainId, entry.txHash)"
                      :href="getTxUrl(entry.chainId, entry.txHash)!"
                      target="_blank"
                      rel="noopener"
                      class="text-neon-cyan hover:text-neon-cyan/80"
                    >
                      <ExternalLink class="h-2.5 w-2.5" />
                    </a>
                  </span>
                </div>
                <!-- Warning for testnet / unknown network -->
                <div
                  v-if="isTestnetOrUnknown(entry.chainId)"
                  class="mt-1 flex items-center gap-1.5 rounded bg-pending/10 px-2 py-1 text-pending"
                >
                  <AlertTriangle class="h-3 w-3 shrink-0" />
                  <span>{{ ETHERSCAN_URLS[entry.chainId] ? 'Testnet attestation — not suitable for production use' : 'Unknown network — cannot verify on-chain' }}</span>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </TransitionGroup>

      <div
        v-if="entries.length === 0 && isConnected"
        class="py-8 text-center font-mono text-xs text-white/30"
      >
        <Clock class="mx-auto mb-2 h-5 w-5 text-white/20" />
        Polling for Attested events...
      </div>
      <div
        v-else-if="entries.length === 0"
        class="py-8 text-center font-mono text-xs text-white/30"
      >
        Waiting for connection...
      </div>
    </div>
  </GlassCard>
</template>
