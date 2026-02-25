<script setup lang="ts">
import { ref } from 'vue'
import type { Step, Attestation } from '@uts/sdk'
import { hexlify } from 'ethers/utils'
import { ChevronRight, ChevronDown, GitBranch, ExternalLink, AlertTriangle } from 'lucide-vue-next'
import { WELL_KNOWN_CHAINS } from '@uts/sdk'
import ScrollLogo from '@/assets/Scroll_Logomark.svg'

const SCROLL_CHAIN_IDS = new Set([534352, 534351])

const props = defineProps<{
  steps: Step[]
  depth?: number
}>()

const expandedAttestations = ref<Set<number>>(new Set())

function toggleAttestation(index: number) {
  if (expandedAttestations.value.has(index)) {
    expandedAttestations.value.delete(index)
  } else {
    expandedAttestations.value.add(index)
  }
}

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

function getChainName(chainId: number): string {
  return CHAIN_NAMES[chainId] ?? WELL_KNOWN_CHAINS[chainId]?.chainName ?? `Chain ${chainId}`
}

function getEtherscanBlockUrl(chainId: number, height: number): string | null {
  const base = ETHERSCAN_URLS[chainId]
  return base ? `${base}/block/${height}` : null
}

function getEtherscanTxUrl(chainId: number, txHash: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  return base ? `${base}/tx/${txHash}` : null
}

function getEtherscanAddressUrl(chainId: number, address: string): string | null {
  const base = ETHERSCAN_URLS[chainId]
  return base ? `${base}/address/${address}` : null
}

const MAINNET_CHAIN_IDS = new Set([1, 534352])

function isTestnetOrUnknown(chainId: number): boolean {
  return !MAINNET_CHAIN_IDS.has(chainId)
}

function getNetworkWarning(chainId: number): string | null {
  if (MAINNET_CHAIN_IDS.has(chainId)) return null
  if (ETHERSCAN_URLS[chainId]) return 'Testnet attestation — not suitable for production use'
  return 'Unknown network — cannot verify on-chain'
}

function formatOp(step: Step): string {
  switch (step.op) {
    case 'APPEND':
    case 'PREPEND':
      return `${step.op}(${truncateHex(hexlify(step.data))})`
    case 'SHA256':
    case 'KECCAK256':
    case 'SHA1':
    case 'RIPEMD160':
      return step.op
    case 'REVERSE':
      return 'REVERSE'
    case 'HEXLIFY':
      return 'HEXLIFY'
    case 'ATTESTATION':
      return formatAttestation(step.attestation)
    case 'FORK':
      return `FORK (${step.steps.length} branches)`
    default:
      return 'UNKNOWN'
  }
}

function formatAttestation(a: Attestation): string {
  switch (a.kind) {
    case 'bitcoin':
      return `Bitcoin @ block ${a.height}`
    case 'ethereum-uts':
      return `${getChainName(a.chain)} @ block ${a.height}`
    case 'pending':
      return `Pending → ${a.url}`
    default:
      return 'Unknown attestation'
  }
}

function getOpColor(step: Step): string {
  switch (step.op) {
    case 'SHA256':
    case 'KECCAK256':
    case 'SHA1':
    case 'RIPEMD160':
      return 'text-neon-cyan'
    case 'APPEND':
    case 'PREPEND':
      return 'text-neon-purple'
    case 'ATTESTATION':
      if (step.attestation.kind === 'bitcoin') return 'text-pending'
      if (step.attestation.kind === 'ethereum-uts') return 'text-neon-purple'
      return 'text-white/50'
    case 'FORK':
      return 'text-neon-orange'
    default:
      return 'text-white/40'
  }
}

function truncateHex(hex: string): string {
  if (hex.length <= 18) return hex
  return `${hex.slice(0, 10)}...${hex.slice(-6)}`
}

const currentDepth = props.depth ?? 0
</script>

<template>
  <div class="space-y-0.5" :style="{ paddingLeft: currentDepth > 0 ? '16px' : '0' }">
    <div
      v-for="(step, i) in steps"
      :key="i"
      class="group"
    >
      <!-- Attestation step — collapsible with details -->
      <div v-if="step.op === 'ATTESTATION'">
        <button
          class="flex w-full items-center gap-2 rounded px-2 py-1 font-mono text-xs transition-colors hover:bg-white/5 text-left"
          @click="toggleAttestation(i)"
        >
          <component :is="expandedAttestations.has(i) ? ChevronDown : ChevronRight" class="h-3 w-3 text-white/20" />
          <span :class="getOpColor(step)">{{ formatOp(step) }}</span>
          <AlertTriangle
            v-if="step.attestation.kind === 'ethereum-uts' && isTestnetOrUnknown(step.attestation.chain)"
            class="h-3 w-3 text-pending"
            :title="getNetworkWarning(step.attestation.chain) ?? ''"
          />
        </button>

        <Transition name="fade">
          <div v-if="expandedAttestations.has(i)" class="ml-5 mt-1 rounded-lg border border-glass-border/50 bg-surface/30 px-3 py-2">
            <!-- Bitcoin attestation details -->
            <template v-if="step.attestation.kind === 'bitcoin'">
              <div class="space-y-1 font-mono text-[10px]">
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Type</span>
                  <span class="text-pending">Bitcoin</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Block Height</span>
                  <span class="text-white/70">{{ step.attestation.height }}</span>
                </div>
              </div>
            </template>

            <!-- Ethereum UTS attestation details -->
            <template v-else-if="step.attestation.kind === 'ethereum-uts'">
              <div class="space-y-1 font-mono text-[10px]">
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Type</span>
                  <span class="flex items-center gap-1 text-neon-purple">
                    <img v-if="SCROLL_CHAIN_IDS.has(step.attestation.chain)" :src="ScrollLogo" alt="Scroll" class="h-3 w-3" />
                    Ethereum UTS
                  </span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Chain</span>
                  <span class="text-white/70">{{ getChainName(step.attestation.chain) }} ({{ step.attestation.chain }})</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Block Height</span>
                  <span class="flex items-center gap-1">
                    <span class="text-white/70">{{ step.attestation.height }}</span>
                    <a
                      v-if="getEtherscanBlockUrl(step.attestation.chain, step.attestation.height)"
                      :href="getEtherscanBlockUrl(step.attestation.chain, step.attestation.height)!"
                      target="_blank"
                      rel="noopener"
                      class="text-neon-cyan hover:text-neon-cyan/80"
                    >
                      <ExternalLink class="h-2.5 w-2.5" />
                    </a>
                  </span>
                </div>
                <template v-if="step.attestation.metadata">
                  <div v-if="step.attestation.metadata.contract" class="flex items-center justify-between">
                    <span class="text-white/30">Contract</span>
                    <span class="flex items-center gap-1">
                      <span class="text-white/50">{{ truncateHex(hexlify(step.attestation.metadata.contract as Uint8Array)) }}</span>
                      <a
                        v-if="getEtherscanAddressUrl(step.attestation.chain, hexlify(step.attestation.metadata.contract as Uint8Array))"
                        :href="getEtherscanAddressUrl(step.attestation.chain, hexlify(step.attestation.metadata.contract as Uint8Array))!"
                        target="_blank"
                        rel="noopener"
                        class="text-neon-cyan hover:text-neon-cyan/80"
                      >
                        <ExternalLink class="h-2.5 w-2.5" />
                      </a>
                    </span>
                  </div>
                  <div v-if="step.attestation.metadata.txHash" class="flex items-center justify-between">
                    <span class="text-white/30">Tx Hash</span>
                    <span class="flex items-center gap-1">
                      <span class="text-white/50">{{ truncateHex(hexlify(step.attestation.metadata.txHash as Uint8Array)) }}</span>
                      <a
                        v-if="getEtherscanTxUrl(step.attestation.chain, hexlify(step.attestation.metadata.txHash as Uint8Array))"
                        :href="getEtherscanTxUrl(step.attestation.chain, hexlify(step.attestation.metadata.txHash as Uint8Array))!"
                        target="_blank"
                        rel="noopener"
                        class="text-neon-cyan hover:text-neon-cyan/80"
                      >
                        <ExternalLink class="h-2.5 w-2.5" />
                      </a>
                    </span>
                  </div>
                </template>
                <!-- Testnet / unknown network warning -->
                <div
                  v-if="isTestnetOrUnknown(step.attestation.chain)"
                  class="mt-1.5 flex items-center gap-1.5 rounded bg-pending/10 px-2 py-1 text-pending"
                >
                  <AlertTriangle class="h-2.5 w-2.5 shrink-0" />
                  <span>{{ getNetworkWarning(step.attestation.chain) }}</span>
                </div>
              </div>
            </template>

            <!-- Pending attestation details -->
            <template v-else-if="step.attestation.kind === 'pending'">
              <div class="space-y-1 font-mono text-[10px]">
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Type</span>
                  <span class="text-pending">Pending</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-white/30">Calendar</span>
                  <span class="text-white/50">{{ step.attestation.url }}</span>
                </div>
              </div>
            </template>
          </div>
        </Transition>
      </div>

      <!-- Regular step (non-attestation, non-fork) -->
      <div
        v-else-if="step.op !== 'FORK'"
        class="flex items-center gap-2 rounded px-2 py-1 font-mono text-xs transition-colors hover:bg-white/5"
      >
        <ChevronRight class="h-3 w-3 text-white/20" />
        <span :class="getOpColor(step)">{{ formatOp(step) }}</span>
      </div>

      <!-- Fork step - recursive -->
      <div v-else class="mt-1">
        <div class="flex items-center gap-2 px-2 py-1 font-mono text-xs text-neon-orange">
          <GitBranch class="h-3 w-3" />
          <span>FORK ({{ step.steps.length }} branches)</span>
        </div>
        <div
          v-for="(branch, bi) in step.steps"
          :key="bi"
          class="ml-2 mt-1 border-l border-glass-border pl-2"
        >
          <div class="mb-1 font-mono text-[10px] text-white/20">branch[{{ bi }}]</div>
          <MerkleTreeViz :steps="branch" :depth="currentDepth + 1" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export default { name: 'MerkleTreeViz' }
</script>
