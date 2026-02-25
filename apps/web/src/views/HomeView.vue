<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Zap, Shield } from 'lucide-vue-next'
import HeroTerminal from '@/components/terminal/HeroTerminal.vue'
import StampingWorkflow from '@/components/stamp/StampingWorkflow.vue'
import VerificationResult from '@/components/verify/VerificationResult.vue'
import LiveFeed from '@/components/feed/LiveFeed.vue'
import { useTimestampSDK } from '@/composables/useTimestampSDK'
import type { FileDigestResult } from '@/composables/useFileDigest'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const { stampPhase, stampError, stamp, resetStamp } = useTimestampSDK()

const activeTab = ref<'stamp' | 'verify'>('stamp')
const showWorkflow = ref(false)

onMounted(() => {
  store.checkChains()
})

async function handleStampFromDigest(digest: FileDigestResult) {
  showWorkflow.value = true
  try {
    const results = await stamp([digest.header])
    for (const r of results) store.addStamp(r)
  } catch {
    // error is tracked in stampError
  }
}

async function handleStampFromHash(hash: string) {
  showWorkflow.value = true
  const digest = hash.startsWith('0x') ? hash : `0x${hash}`
  const bytes = new Uint8Array(
    (digest.slice(2).match(/.{2}/g) ?? []).map((b) => parseInt(b, 16)),
  )
  try {
    const results = await stamp([{ kind: 'SHA256', digest: bytes }])
    for (const r of results) store.addStamp(r)
  } catch {
    // error is tracked in stampError
  }
}

function handleResetWorkflow() {
  resetStamp()
  showWorkflow.value = false
}
</script>

<template>
  <div class="scanlines min-h-screen bg-deep-black">
    <!-- Header -->
    <header class="border-b border-glass-border bg-midnight/80 backdrop-blur-md">
      <div class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-neon-cyan/10">
            <Zap class="h-5 w-5 text-neon-cyan" />
          </div>
          <div>
            <h1 class="font-heading text-lg font-bold tracking-tight text-white">
              UTS
            </h1>
            <p class="font-mono text-[10px] text-white/30">Universal Timestamps</p>
          </div>
        </div>

        <!-- Ethereum chain status -->
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <span
              v-for="chain in store.ethChains"
              :key="chain.chainId"
              class="group relative h-2 w-2 cursor-help rounded-full"
              :class="{
                'bg-valid animate-glow-pulse': chain.status === 'online',
                'bg-invalid': chain.status === 'offline',
                'bg-pending animate-glow-pulse': chain.status === 'checking',
              }"
              :title="`${chain.name} (${chain.chainId}) — ${chain.status}${chain.latency ? ` (${chain.latency}ms)` : ''}`"
            />
          </div>
          <span class="font-mono text-[10px] text-white/30">
            {{ store.onlineCount }}/{{ store.ethChains.length }} chains
          </span>
        </div>
      </div>
    </header>

    <!-- Main content -->
    <main class="mx-auto max-w-6xl px-6 py-8">
      <!-- Title -->
      <div class="mb-10 text-center">
        <h2 class="font-heading text-3xl font-bold tracking-tight text-white">
          Decentralized <span class="text-neon-cyan glow-text-cyan">Timestamping</span>
        </h2>
        <p class="mt-2 font-mono text-sm text-white/40">
          Cryptographic proof of existence anchored to Ethereum
        </p>
      </div>

      <!-- Tab navigation -->
      <div class="mb-8 flex justify-center gap-1 rounded-xl bg-surface/50 p-1">
        <button
          class="flex items-center gap-2 rounded-lg px-6 py-2.5 font-heading text-sm font-medium transition-all"
          :class="
            activeTab === 'stamp'
              ? 'bg-neon-cyan/10 text-neon-cyan'
              : 'text-white/40 hover:text-white/60'
          "
          @click="activeTab = 'stamp'; handleResetWorkflow()"
        >
          <Zap class="h-4 w-4" />
          Stamp
        </button>
        <button
          class="flex items-center gap-2 rounded-lg px-6 py-2.5 font-heading text-sm font-medium transition-all"
          :class="
            activeTab === 'verify'
              ? 'bg-neon-cyan/10 text-neon-cyan'
              : 'text-white/40 hover:text-white/60'
          "
          @click="activeTab = 'verify'"
        >
          <Shield class="h-4 w-4" />
          Verify
        </button>
      </div>

      <!-- Content -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <!-- Main panel (2/3) -->
        <div class="space-y-6 lg:col-span-2">
          <Transition name="fade" mode="out-in">
            <!-- Stamp tab -->
            <div v-if="activeTab === 'stamp'" key="stamp" class="space-y-6">
              <HeroTerminal
                @submit="handleStampFromDigest"
                @submit-raw="handleStampFromHash"
              />

              <Transition name="fade">
                <StampingWorkflow
                  v-if="showWorkflow"
                  :phase="stampPhase"
                  :error="stampError"
                />
              </Transition>
            </div>

            <!-- Verify tab -->
            <div v-else key="verify">
              <VerificationResult />
            </div>
          </Transition>
        </div>

        <!-- Sidebar (1/3) -->
        <div class="space-y-6">
          <LiveFeed />
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="border-t border-glass-border py-6 text-center">
      <p class="font-mono text-[10px] text-white/20">
        UTS Protocol — Powered by Universal Timestamps
      </p>
    </footer>
  </div>
</template>
