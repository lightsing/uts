<script setup lang="ts">
import { ref } from 'vue'
import { ChevronDown, ChevronUp, ExternalLink } from 'lucide-vue-next'
import StatusBadge from '@/components/base/StatusBadge.vue'
import type { AttestationStatus } from '@uts/sdk'
import { WELL_KNOWN_CHAINS } from '@uts/sdk'
import { hexlify } from 'ethers/utils'

const props = defineProps<{
  attestation: AttestationStatus
  defaultExpanded?: boolean
}>()

const expanded = ref(props.defaultExpanded ?? false)

const ETHERSCAN_URLS: Record<number, string> = {
  1: 'https://etherscan.io',
  17000: 'https://holesky.etherscan.io',
  11155111: 'https://sepolia.etherscan.io',
  534352: 'https://scrollscan.com',
  534351: 'https://sepolia.scrollscan.com',
}

const CHAIN_NAMES: Record<number, string> = {
  1: 'Ethereum',
  17000: 'Holesky',
  11155111: 'Sepolia',
  534352: 'Scroll',
  534351: 'Scroll Sepolia',
}

function statusLabel(status: string): 'valid' | 'invalid' | 'pending' | 'unknown' {
  switch (status) {
    case 'VALID': return 'valid'
    case 'PENDING': return 'pending'
    case 'INVALID': return 'invalid'
    default: return 'unknown'
  }
}

function getEtherscanBlockUrl(chainId: number, height: number): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/block/${height}`
}

function getEtherscanTxUrl(chainId: number, txHash: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/tx/${txHash}`
}

function getEtherscanAddressUrl(chainId: number, address: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/address/${address}`
}

function getChainName(chainId: number): string {
  return CHAIN_NAMES[chainId] ?? WELL_KNOWN_CHAINS[chainId]?.chainName ?? `Chain ${chainId}`
}

function truncateHex(hex: string): string {
  if (hex.length <= 18) return hex
  return `${hex.slice(0, 10)}...${hex.slice(-6)}`
}

function formatTimestamp(ts: bigint | number): string {
  const date = new Date(Number(ts) * 1000)
  return date.toISOString().replace('T', ' ').replace('.000Z', ' UTC')
}
</script>

<template>
  <div class="rounded-lg border border-glass-border bg-surface/40 transition-colors hover:border-white/10">
    <!-- Header (always visible) -->
    <button
      class="flex w-full items-center gap-2 px-3 py-2.5 text-left"
      @click="expanded = !expanded"
    >
      <StatusBadge
        :status="statusLabel(attestation.status)"
        size="sm"
      />
      <span class="flex-1 font-mono text-xs text-white/60">
        <template v-if="attestation.attestation.kind === 'bitcoin'">
          Bitcoin block #{{ attestation.attestation.height }}
        </template>
        <template v-else-if="attestation.attestation.kind === 'ethereum-uts'">
          {{ getChainName(attestation.attestation.chain) }} block #{{ attestation.attestation.height }}
        </template>
        <template v-else-if="attestation.attestation.kind === 'pending'">
          Pending → {{ attestation.attestation.url }}
        </template>
        <template v-else>Unknown attestation</template>
      </span>
      <component :is="expanded ? ChevronUp : ChevronDown" class="h-3.5 w-3.5 text-white/30" />
    </button>

    <!-- Expanded details -->
    <Transition name="fade">
      <div v-if="expanded" class="border-t border-glass-border px-3 pb-3 pt-2">
        <!-- Bitcoin attestation details -->
        <template v-if="attestation.attestation.kind === 'bitcoin'">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">Type</span>
              <span class="text-pending">Bitcoin</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">Block Height</span>
              <span class="text-white/70">{{ attestation.attestation.height }}</span>
            </div>
            <template v-if="attestation.status === 'VALID' && attestation.additionalInfo?.header">
              <div class="flex items-center justify-between">
                <span class="text-white/30">Merkle Root</span>
                <span class="text-white/50">{{ truncateHex(attestation.additionalInfo.header.merkleroot) }}</span>
              </div>
            </template>
          </div>
        </template>

        <!-- Ethereum UTS attestation details -->
        <template v-else-if="attestation.attestation.kind === 'ethereum-uts'">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">Type</span>
              <span class="text-neon-purple">Ethereum UTS</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">Chain</span>
              <span class="text-white/70">{{ getChainName(attestation.attestation.chain) }} ({{ attestation.attestation.chain }})</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">Block Height</span>
              <span class="flex items-center gap-1">
                <span class="text-white/70">{{ attestation.attestation.height }}</span>
                <a
                  v-if="getEtherscanBlockUrl(attestation.attestation.chain, attestation.attestation.height)"
                  :href="getEtherscanBlockUrl(attestation.attestation.chain, attestation.attestation.height)!"
                  target="_blank"
                  rel="noopener"
                  class="text-neon-cyan hover:text-neon-cyan/80"
                >
                  <ExternalLink class="h-3 w-3" />
                </a>
              </span>
            </div>
            <template v-if="attestation.status === 'VALID' && attestation.additionalInfo">
              <div v-if="attestation.additionalInfo.sender" class="flex items-center justify-between">
                <span class="text-white/30">Sender</span>
                <span class="flex items-center gap-1">
                  <span class="text-white/50">{{ truncateHex(attestation.additionalInfo.sender) }}</span>
                  <a
                    v-if="getEtherscanAddressUrl(attestation.attestation.chain, attestation.additionalInfo.sender)"
                    :href="getEtherscanAddressUrl(attestation.attestation.chain, attestation.additionalInfo.sender)!"
                    target="_blank"
                    rel="noopener"
                    class="text-neon-cyan hover:text-neon-cyan/80"
                  >
                    <ExternalLink class="h-3 w-3" />
                  </a>
                </span>
              </div>
              <div v-if="attestation.additionalInfo.timestamp" class="flex items-center justify-between">
                <span class="text-white/30">Timestamp</span>
                <span class="text-white/50">{{ formatTimestamp(attestation.additionalInfo.timestamp) }}</span>
              </div>
              <div v-if="attestation.additionalInfo.root" class="flex items-center justify-between">
                <span class="text-white/30">Root</span>
                <span class="text-white/50">{{ truncateHex(attestation.additionalInfo.root) }}</span>
              </div>
            </template>
            <template v-if="attestation.attestation.metadata">
              <div v-if="attestation.attestation.metadata.contract" class="flex items-center justify-between">
                <span class="text-white/30">Contract</span>
                <span class="flex items-center gap-1">
                  <span class="text-white/50">{{ truncateHex(hexlify(attestation.attestation.metadata.contract as Uint8Array)) }}</span>
                  <a
                    v-if="getEtherscanAddressUrl(attestation.attestation.chain, hexlify(attestation.attestation.metadata.contract as Uint8Array))"
                    :href="getEtherscanAddressUrl(attestation.attestation.chain, hexlify(attestation.attestation.metadata.contract as Uint8Array))!"
                    target="_blank"
                    rel="noopener"
                    class="text-neon-cyan hover:text-neon-cyan/80"
                  >
                    <ExternalLink class="h-3 w-3" />
                  </a>
                </span>
              </div>
              <div v-if="attestation.attestation.metadata.txHash" class="flex items-center justify-between">
                <span class="text-white/30">Tx Hash</span>
                <span class="flex items-center gap-1">
                  <span class="text-white/50">{{ truncateHex(hexlify(attestation.attestation.metadata.txHash as Uint8Array)) }}</span>
                  <a
                    v-if="getEtherscanTxUrl(attestation.attestation.chain, hexlify(attestation.attestation.metadata.txHash as Uint8Array))"
                    :href="getEtherscanTxUrl(attestation.attestation.chain, hexlify(attestation.attestation.metadata.txHash as Uint8Array))!"
                    target="_blank"
                    rel="noopener"
                    class="text-neon-cyan hover:text-neon-cyan/80"
                  >
                    <ExternalLink class="h-3 w-3" />
                  </a>
                </span>
              </div>
            </template>
          </div>
        </template>

        <!-- Pending attestation details -->
        <template v-else-if="attestation.attestation.kind === 'pending'">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">Type</span>
              <span class="text-pending">Pending</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">Calendar</span>
              <span class="text-white/50">{{ attestation.attestation.url }}</span>
            </div>
          </div>
        </template>

        <!-- Error info -->
        <div
          v-if="(attestation.status === 'INVALID' || attestation.status === 'UNKNOWN') && 'error' in attestation"
          class="mt-2 rounded border border-invalid/20 bg-invalid/5 px-2 py-1.5 font-mono text-[10px] text-invalid"
        >
          {{ attestation.error?.message }}
        </div>
      </div>
    </Transition>
  </div>
</template>
