<script setup lang="ts">
import { ref } from 'vue'
import {
  ChevronDown,
  ChevronUp,
  ExternalLink,
  AlertTriangle,
} from 'lucide-vue-next'
import StatusBadge from '@/components/base/StatusBadge.vue'
import { useLingui } from '@/composables/useLingui'
import type { AttestationStatus } from '@uts/sdk'
import { WELL_KNOWN_CHAINS } from '@uts/sdk'
import { hexlify } from 'ethers/utils'
import ScrollLogo from '@/assets/Scroll_Logomark.svg'

const { t } = useLingui()

const SCROLL_CHAIN_IDS = new Set([534352, 534351])

const props = defineProps<{
  attestation: AttestationStatus
  defaultExpanded?: boolean
}>()

const expanded = ref(props.defaultExpanded ?? false)

const MAINNET_CHAIN_IDS = new Set([1, 534352])

function isTestnetOrUnknown(chainId: number): boolean {
  return !MAINNET_CHAIN_IDS.has(chainId)
}

function getNetworkWarning(chainId: number): string | null {
  if (MAINNET_CHAIN_IDS.has(chainId)) return null
  if (ETHERSCAN_URLS[chainId])
    return t('Testnet attestation — not suitable for production use')
  return t('Unknown network — cannot verify on-chain')
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

function statusLabel(
  status: string,
): 'valid' | 'invalid' | 'pending' | 'unknown' {
  switch (status) {
    case 'VALID':
      return 'valid'
    case 'PENDING':
      return 'pending'
    case 'INVALID':
      return 'invalid'
    default:
      return 'unknown'
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

function getEtherscanAddressUrl(
  chainId: number,
  address: string,
): string | null {
  const base = ETHERSCAN_URLS[chainId]
  if (!base) return null
  return `${base}/address/${address}`
}

function getChainName(chainId: number): string {
  return (
    CHAIN_NAMES[chainId] ??
    WELL_KNOWN_CHAINS[chainId]?.chainName ??
    `Chain ${chainId}`
  )
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
  <div
    class="rounded-lg border border-glass-border bg-surface/40 transition-colors hover:border-white/10"
  >
    <!-- Header (always visible) -->
    <button
      class="flex w-full items-center gap-2 px-3 py-2.5 text-left"
      @click="expanded = !expanded"
    >
      <StatusBadge :status="statusLabel(attestation.status)" size="sm" />
      <span class="flex-1 font-mono text-xs text-white/60">
        <template v-if="attestation.attestation.kind === 'bitcoin'">
          {{ t('Bitcoin block #{height}', { height: attestation.attestation.height }) }}
        </template>
        <template v-else-if="attestation.attestation.kind.startsWith('eas')">
          <span class="inline-flex items-center gap-1">
            <img
              v-if="SCROLL_CHAIN_IDS.has(attestation.attestation.chain)"
              :src="ScrollLogo"
              alt="Scroll"
              class="inline h-3.5 w-3.5"
            />
            {{ t('{chain} block #{height}', { chain: getChainName(attestation.attestation.chain), height: attestation.attestation.height }) }}
          </span>
        </template>
        <template v-else-if="attestation.attestation.kind === 'pending'">
          {{ t('Pending → {url}', { url: attestation.attestation.url }) }}
        </template>
        <template v-else>{{ t('Unknown attestation') }}</template>
      </span>
      <AlertTriangle
        v-if="
          attestation.attestation.kind.startsWith('eas') &&
          isTestnetOrUnknown(attestation.attestation.chain)
        "
        class="h-3.5 w-3.5 shrink-0 text-pending"
        :title="getNetworkWarning(attestation.attestation.chain) ?? ''"
      />
      <component
        :is="expanded ? ChevronUp : ChevronDown"
        class="h-3.5 w-3.5 text-white/30"
      />
    </button>

    <!-- Expanded details -->
    <Transition name="fade">
      <div v-if="expanded" class="border-t border-glass-border px-3 pb-3 pt-2">
        <!-- Bitcoin attestation details -->
        <template v-if="attestation.attestation.kind === 'bitcoin'">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Type') }}</span>
              <span class="text-pending">{{ t('Bitcoin') }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Block Height') }}</span>
              <span class="text-white/70">{{
                attestation.attestation.height
              }}</span>
            </div>
            <template
              v-if="
                attestation.status === 'VALID' &&
                attestation.additionalInfo?.header
              "
            >
              <div class="flex items-center justify-between">
                <span class="text-white/30">{{ t('Merkle Root') }}</span>
                <span class="text-white/50">{{
                  truncateHex(attestation.additionalInfo.header.merkleroot)
                }}</span>
              </div>
            </template>
          </div>
        </template>

        <!-- Ethereum UTS attestation details -->
        <template v-else-if="attestation.attestation.kind.startsWith('eas')">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Type') }}</span>
              <span class="flex items-center gap-1 text-neon-purple">
                <img
                  v-if="SCROLL_CHAIN_IDS.has(attestation.attestation.chain)"
                  :src="ScrollLogo"
                  alt="Scroll"
                  class="h-3.5 w-3.5"
                />
                {{ t('EAS') }}
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Chain') }}</span>
              <span class="text-white/70"
                >{{ getChainName(attestation.attestation.chain) }} ({{
                  attestation.attestation.chain
                }})</span
              >
            </div>
            <template
              v-if="
                attestation.status === 'VALID' && attestation.additionalInfo
              "
            >
              <div
                v-if="attestation.additionalInfo.time"
                class="flex items-center justify-between"
              >
                <span class="text-white/30">{{ t('Timestamp') }}</span>
                <span class="text-white/50">{{
                  formatTimestamp(attestation.additionalInfo.time)
                }}</span>
              </div>
              <div
                v-if="attestation.additionalInfo.root"
                class="flex items-center justify-between"
              >
                <span class="text-white/30">{{ t('Root') }}</span>
                <span class="text-white/50">{{
                  truncateHex(attestation.additionalInfo.root)
                }}</span>
              </div>
            </template>
            <!-- Testnet / unknown network warning -->
            <div
              v-if="isTestnetOrUnknown(attestation.attestation.chain)"
              class="mt-2 flex items-center gap-1.5 rounded bg-pending/10 px-2 py-1.5 font-mono text-[10px] text-pending"
            >
              <AlertTriangle class="h-3 w-3 shrink-0" />
              <span>{{
                getNetworkWarning(attestation.attestation.chain)
              }}</span>
            </div>
          </div>
        </template>

        <!-- Pending attestation details -->
        <template v-else-if="attestation.attestation.kind === 'pending'">
          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Type') }}</span>
              <span class="text-pending">{{ t('Pending') }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-white/30">{{ t('Calendar') }}</span>
              <span class="text-white/50">{{
                attestation.attestation.url
              }}</span>
            </div>
          </div>
        </template>

        <!-- Error info -->
        <div
          v-if="
            (attestation.status === 'INVALID' ||
              attestation.status === 'UNKNOWN') &&
            'error' in attestation
          "
          class="mt-2 rounded border border-invalid/20 bg-invalid/5 px-2 py-1.5 font-mono text-[10px] text-invalid"
        >
          {{ attestation.error?.message }}
        </div>
      </div>
    </Transition>
  </div>
</template>
