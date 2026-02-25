<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { Zap, Shield, Settings, Plus, Trash2, RotateCcw, Wallet } from 'lucide-vue-next'
import HeroTerminal from '@/components/terminal/HeroTerminal.vue'
import StampingWorkflow from '@/components/stamp/StampingWorkflow.vue'
import VerificationResult from '@/components/verify/VerificationResult.vue'
import LiveFeed from '@/components/feed/LiveFeed.vue'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import { useTimestampSDK, setWeb3Provider } from '@/composables/useTimestampSDK'
import { useWallet } from '@/composables/useWallet'
import type { FileDigestResult } from '@/composables/useFileDigest'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const { stampPhase, stampError, broadcastProgress, stamp, resetStamp } = useTimestampSDK()
const {
  walletAddress,
  walletChainName,
  isConnected: walletConnected,
  isConnecting: walletConnecting,
  hasWallet,
  connect: connectWallet,
  disconnect: disconnectWallet,
  getEip1193Provider,
  truncateAddress,
} = useWallet()

const activeTab = ref<'stamp' | 'verify'>('stamp')
const showWorkflow = ref(false)
const showSettings = ref(false)
const newCalendarUrl = ref('')

onMounted(() => {
  store.checkChains()
})

// Sync wallet provider to SDK when wallet connects/disconnects
watch(walletConnected, (connected) => {
  if (connected) {
    setWeb3Provider(getEip1193Provider())
  } else {
    setWeb3Provider(null)
  }
})

async function handleStampFromDigest(digest: FileDigestResult) {
  showWorkflow.value = true
  try {
    const results = await stamp([digest.header], digest.fileName)
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

function addCalendar() {
  const url = newCalendarUrl.value.trim()
  if (!url) return
  try {
    new URL(url)
    if (!store.calendarUrls.includes(url)) {
      store.setCalendars([...store.calendarUrls, url])
    }
    newCalendarUrl.value = ''
  } catch {
    // invalid URL, ignore
  }
}

function removeCalendar(index: number) {
  const urls = [...store.calendarUrls]
  urls.splice(index, 1)
  store.setCalendars(urls)
}

function handleWalletClick() {
  if (walletConnected.value) {
    disconnectWallet()
  } else {
    connectWallet()
  }
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

        <div class="flex items-center gap-4">
          <!-- Ethereum chain status -->
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
          <!-- Connect Wallet button -->
          <button
            class="flex items-center gap-2 rounded-lg border px-3 py-1.5 font-mono text-xs transition-all"
            :class="walletConnected
              ? 'border-valid/30 bg-valid/5 text-valid hover:bg-valid/10'
              : 'border-neon-purple/30 bg-neon-purple/5 text-neon-purple hover:bg-neon-purple/10'
            "
            :disabled="walletConnecting"
            @click="handleWalletClick"
          >
            <Wallet class="h-3.5 w-3.5" />
            <span v-if="walletConnecting">Connecting...</span>
            <span v-else-if="walletConnected && walletAddress">
              {{ truncateAddress(walletAddress) }}
              <span v-if="walletChainName" class="text-white/30"> · {{ walletChainName }}</span>
            </span>
            <span v-else-if="!hasWallet">No Wallet</span>
            <span v-else>Connect Wallet</span>
          </button>
          <!-- Settings button -->
          <button
            class="rounded-lg p-1.5 text-white/40 transition hover:bg-white/5 hover:text-white/60"
            :class="{ 'bg-neon-cyan/10 text-neon-cyan': showSettings }"
            title="Calendar settings"
            @click="showSettings = !showSettings"
          >
            <Settings class="h-4 w-4" />
          </button>
        </div>
      </div>
    </header>

    <!-- Calendar settings panel -->
    <Transition name="fade">
      <div v-if="showSettings" class="border-b border-glass-border bg-midnight/60 backdrop-blur-md">
        <div class="mx-auto max-w-6xl px-6 py-4">
          <GlassCard>
            <div class="mb-3 flex items-center justify-between">
              <h3 class="font-heading text-sm font-semibold text-white/80">Calendar Nodes</h3>
              <BaseButton variant="secondary" @click="store.resetCalendars()">
                <RotateCcw class="h-3 w-3" />
                Reset to defaults
              </BaseButton>
            </div>
            <div class="space-y-2">
              <div
                v-for="(url, i) in store.calendarUrls"
                :key="i"
                class="flex items-center gap-2"
              >
                <span class="flex-1 truncate rounded border border-glass-border bg-surface px-3 py-1.5 font-mono text-xs text-white/60">
                  {{ url }}
                </span>
                <button
                  class="rounded p-1 text-white/30 transition hover:bg-invalid/10 hover:text-invalid"
                  @click="removeCalendar(i)"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </div>
              <div class="flex gap-2">
                <input
                  v-model="newCalendarUrl"
                  type="text"
                  placeholder="https://calendar.example.com/"
                  class="flex-1 rounded-lg border border-glass-border bg-surface px-3 py-1.5 font-mono text-xs text-white/80 outline-none placeholder:text-white/20 focus:border-neon-cyan/40"
                  @keyup.enter="addCalendar"
                />
                <BaseButton variant="secondary" @click="addCalendar">
                  <Plus class="h-3 w-3" />
                  Add
                </BaseButton>
              </div>
            </div>
          </GlassCard>
        </div>
      </div>
    </Transition>

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
                  :broadcast-progress="broadcastProgress"
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
