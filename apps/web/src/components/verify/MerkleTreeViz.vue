<script setup lang="ts">
import type { Step } from '@uts/sdk'
import { hexlify } from 'ethers/utils'
import { ChevronRight, GitBranch } from 'lucide-vue-next'

const props = defineProps<{
  steps: Step[]
  depth?: number
}>()

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
      return formatAttestation(step)
    case 'FORK':
      return `FORK (${step.steps.length} branches)`
    default:
      return 'UNKNOWN'
  }
}

function formatAttestation(step: Step): string {
  if (step.op !== 'ATTESTATION') return ''
  const a = step.attestation
  switch (a.kind) {
    case 'bitcoin':
      return `Bitcoin @ block ${a.height}`
    case 'ethereum-uts':
      return `Ethereum (chain ${a.chain}) @ block ${a.height}`
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
      <!-- Regular step -->
      <div
        v-if="step.op !== 'FORK'"
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
