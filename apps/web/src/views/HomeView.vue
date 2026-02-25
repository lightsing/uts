<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import {
  Zap,
  Shield,
  Settings,
  Plus,
  Trash2,
  RotateCcw,
  Wallet,
  RefreshCw,
  ChevronDown,
  Pencil,
  Check,
} from 'lucide-vue-next'
import HeroTerminal from '@/components/terminal/HeroTerminal.vue'
import StampingWorkflow from '@/components/stamp/StampingWorkflow.vue'
import VerificationResult from '@/components/verify/VerificationResult.vue'
import UpgradePanel from '@/components/upgrade/UpgradePanel.vue'
import LiveFeed from '@/components/feed/LiveFeed.vue'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import {
  useTimestampSDK,
  setWeb3Provider,
  getSDK,
} from '@/composables/useTimestampSDK'
import { useWallet } from '@/composables/useWallet'
import type { FileDigestResult } from '@/composables/useFileDigest'
import { useAppStore } from '@/stores/app'
import ScrollLogo from '@/assets/Scroll_Logomark.svg'

const SCROLL_CHAIN_IDS = new Set([534352, 534351])

const store = useAppStore()
const {
  stampPhase,
  stampError,
  broadcastProgress,
  stamp,
  downloadPendingStamp,
} = useTimestampSDK()
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

type TabId = 'stamp' | 'verify' | 'upgrade'
const savedTab = localStorage.getItem('uts-active-tab') as TabId | null
const validTabs: TabId[] = ['stamp', 'verify', 'upgrade']
const activeTab = ref<TabId>(
  savedTab && validTabs.includes(savedTab) ? savedTab : 'stamp',
)
const showWorkflow = ref(false)
const showSettings = ref(false)
const showChainPanel = ref(false)
const newCalendarUrl = ref('')
const newChainId = ref('')
const editingChainRpc = ref<number | null>(null)
const editRpcValue = ref('')

function startEditRpc(chainId: number, currentRpc?: string) {
  editingChainRpc.value = chainId
  editRpcValue.value = currentRpc ?? ''
}

function saveEditRpc(chainId: number) {
  store.setChainRpc(chainId, editRpcValue.value)
  editingChainRpc.value = null
  editRpcValue.value = ''
  store.checkChains()
}

watch(activeTab, (tab) => {
  localStorage.setItem('uts-active-tab', tab)
})

onMounted(() => {
  store.checkChains()
})

function handleAddChain() {
  const id = parseInt(newChainId.value.trim(), 10)
  if (!id || isNaN(id)) return
  store.addChain(id)
  store.checkChains()
  newChainId.value = ''
}

// Sync wallet provider to SDK when wallet connects/disconnects
watch(walletConnected, (connected) => {
  if (connected) {
    setWeb3Provider(getEip1193Provider())
  } else {
    setWeb3Provider(null)
  }
})

async function handleStampFromDigest(digests: FileDigestResult[]) {
  showWorkflow.value = true

  // Apply internal hash algorithm setting
  getSDK().hashAlgorithm = store.internalHashAlgo

  const fileNames = digests.map((d) => d.fileName)

  try {
    const headers = digests.map((d) => d.header)
    const results = await stamp(headers, fileNames)
    for (const r of results) store.addStamp(r)
  } catch {
    // error is tracked in stampError
  }
}

async function handleStampFromHash(hash: string) {
  showWorkflow.value = true

  // Apply internal hash algorithm setting
  getSDK().hashAlgorithm = store.internalHashAlgo

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
    <header
      class="relative z-50 border-b border-glass-border bg-midnight/80 backdrop-blur-md"
    >
      <div
        class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4"
      >
        <div class="flex items-center gap-3">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-neon-cyan/10"
          >
            <Zap class="h-5 w-5 text-neon-cyan" />
          </div>
          <div>
            <h1
              class="font-heading text-lg font-bold tracking-tight text-white"
            >
              UTS
            </h1>
            <p class="font-mono text-[10px] text-white/30">
              Universal Timestamps
            </p>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <!-- Ethereum chain status — clickable -->
          <div class="relative">
            <button
              class="flex items-center gap-2 rounded-lg border border-glass-border px-3 py-1.5 font-mono text-xs text-white/50 transition hover:border-white/20 hover:text-white/70"
              @click="showChainPanel = !showChainPanel"
            >
              <span
                v-for="chain in store.ethChains"
                :key="chain.chainId"
                class="flex items-center gap-0.5"
              >
                <span
                  class="h-1.5 w-1.5 rounded-full"
                  :class="{
                    'bg-valid': chain.status === 'online',
                    'bg-invalid': chain.status === 'offline',
                    'bg-pending': chain.status === 'checking',
                  }"
                />
              </span>
              <span
                >{{ store.onlineCount }}/{{
                  store.ethChains.length
                }}
                chains</span
              >
              <ChevronDown class="h-3 w-3" />
            </button>

            <!-- Chain detail dropdown -->
            <Transition name="fade">
              <div
                v-if="showChainPanel"
                class="absolute right-0 top-full z-50 mt-2 w-80 rounded-xl border border-glass-border bg-midnight/95 p-4 shadow-xl backdrop-blur-lg"
              >
                <div class="mb-3 flex items-center justify-between">
                  <h4 class="font-heading text-xs font-semibold text-white/80">
                    Ethereum Chains
                  </h4>
                  <BaseButton variant="secondary" @click="store.checkChains()">
                    <RefreshCw class="h-3 w-3" />
                    Refresh
                  </BaseButton>
                </div>

                <div class="space-y-1.5">
                  <div
                    v-for="chain in store.ethChains"
                    :key="chain.chainId"
                    class="rounded-lg bg-surface/40 px-3 py-2"
                  >
                    <div class="flex items-center gap-2">
                      <span
                        class="h-2 w-2 rounded-full"
                        :class="{
                          'bg-valid': chain.status === 'online',
                          'bg-invalid': chain.status === 'offline',
                          'bg-pending animate-glow-pulse':
                            chain.status === 'checking',
                        }"
                      />
                      <div class="min-w-0 flex-1">
                        <div
                          class="font-heading text-xs font-medium text-white/70"
                        >
                          {{ chain.name }}
                        </div>
                        <div class="font-mono text-[10px] text-white/30">
                          Chain ID: {{ chain.chainId }}
                        </div>
                      </div>
                      <div class="text-right">
                        <div
                          class="font-mono text-[10px]"
                          :class="{
                            'text-valid': chain.status === 'online',
                            'text-invalid': chain.status === 'offline',
                            'text-pending': chain.status === 'checking',
                          }"
                        >
                          {{ chain.status }}
                        </div>
                        <div
                          v-if="chain.latency"
                          class="font-mono text-[10px] text-white/20"
                        >
                          {{ chain.latency }}ms
                        </div>
                      </div>
                      <button
                        class="rounded p-0.5 text-white/20 transition hover:bg-neon-cyan/10 hover:text-neon-cyan"
                        title="Edit RPC endpoint"
                        @click="startEditRpc(chain.chainId, chain.rpcUrl)"
                      >
                        <Pencil class="h-3 w-3" />
                      </button>
                      <button
                        class="rounded p-0.5 text-white/20 transition hover:bg-invalid/10 hover:text-invalid"
                        title="Remove chain"
                        @click="store.removeChain(chain.chainId)"
                      >
                        <Trash2 class="h-3 w-3" />
                      </button>
                    </div>
                    <!-- RPC editing row -->
                    <div
                      v-if="editingChainRpc === chain.chainId"
                      class="mt-1.5 flex gap-1.5"
                    >
                      <input
                        v-model="editRpcValue"
                        type="text"
                        placeholder="https://rpc-endpoint.example.com"
                        class="flex-1 rounded border border-glass-border bg-surface px-2 py-1 font-mono text-[10px] text-white/70 outline-none placeholder:text-white/20 focus:border-neon-cyan/40"
                        @keyup.enter="saveEditRpc(chain.chainId)"
                      />
                      <button
                        class="rounded bg-neon-cyan/10 px-1.5 py-1 text-neon-cyan transition hover:bg-neon-cyan/20"
                        title="Save"
                        @click="saveEditRpc(chain.chainId)"
                      >
                        <Check class="h-3 w-3" />
                      </button>
                    </div>
                    <div
                      v-else-if="chain.rpcUrl"
                      class="mt-0.5 truncate font-mono text-[10px] text-white/20 pl-4"
                    >
                      {{ chain.rpcUrl }}
                    </div>
                  </div>
                </div>

                <!-- Add custom chain -->
                <div class="mt-3 border-t border-glass-border pt-3">
                  <div class="font-mono text-[10px] text-white/30 mb-1.5">
                    Add chain by ID
                  </div>
                  <div class="flex gap-2">
                    <input
                      v-model="newChainId"
                      type="text"
                      placeholder="e.g. 42161"
                      class="flex-1 rounded-lg border border-glass-border bg-surface px-2 py-1 font-mono text-xs text-white/80 outline-none placeholder:text-white/20 focus:border-neon-cyan/40"
                      @keyup.enter="handleAddChain"
                    />
                    <BaseButton variant="secondary" @click="handleAddChain">
                      <Plus class="h-3 w-3" />
                      Add
                    </BaseButton>
                  </div>
                  <div class="mt-2 flex justify-end">
                    <BaseButton
                      variant="secondary"
                      @click="
                        store.resetChains()
                        store.checkChains()
                      "
                    >
                      <RotateCcw class="h-3 w-3" />
                      Reset defaults
                    </BaseButton>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
          <!-- Connect Wallet button -->
          <button
            class="flex items-center gap-2 rounded-lg border px-3 py-1.5 font-mono text-xs transition-all"
            :class="
              walletConnected
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
              <span v-if="walletChainName" class="text-white/30">
                · {{ walletChainName }}</span
              >
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
      <div
        v-if="showSettings"
        class="border-b border-glass-border bg-midnight/60 backdrop-blur-md"
      >
        <div class="mx-auto max-w-6xl px-6 py-4">
          <GlassCard>
            <div class="mb-3 flex items-center justify-between">
              <h3 class="font-heading text-sm font-semibold text-white/80">
                Calendar Nodes
              </h3>
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
                <span
                  class="flex-1 truncate rounded border border-glass-border bg-surface px-3 py-1.5 font-mono text-xs text-white/60"
                >
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

            <!-- Upgrade behavior -->
            <div class="mt-4 border-t border-glass-border pt-3 space-y-3">
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="store.keepPending"
                  type="checkbox"
                  class="h-4 w-4 rounded border-glass-border bg-surface accent-neon-cyan"
                />
                <div>
                  <div class="font-heading text-xs font-medium text-white/70">
                    Keep pending attestations after upgrade
                  </div>
                  <div class="font-mono text-[10px] text-white/30">
                    When enabled, the original pending attestation is preserved
                    alongside the upgraded one
                  </div>
                </div>
              </label>

              <!-- Internal hash algorithm -->
              <div class="flex items-center gap-3">
                <label class="font-heading text-xs font-medium text-white/70"
                  >Internal hash algorithm</label
                >
                <select
                  v-model="store.internalHashAlgo"
                  class="rounded border border-glass-border bg-surface px-2 py-1 font-mono text-xs text-neon-cyan outline-none focus:border-neon-cyan/40"
                >
                  <option value="KECCAK256">Keccak-256</option>
                  <option value="SHA256">SHA-256</option>
                </select>
                <span class="font-mono text-[10px] text-white/30"
                  >Used for Merkle tree construction</span
                >
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
          Decentralized
          <span class="text-neon-cyan glow-text-cyan">Timestamping</span>
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
          @click="activeTab = 'stamp'"
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
        <button
          class="flex items-center gap-2 rounded-lg px-6 py-2.5 font-heading text-sm font-medium transition-all"
          :class="
            activeTab === 'upgrade'
              ? 'bg-neon-purple/10 text-neon-purple'
              : 'text-white/40 hover:text-white/60'
          "
          @click="activeTab = 'upgrade'"
        >
          <RefreshCw class="h-4 w-4" />
          Upgrade
        </button>
      </div>

      <!-- Content -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-5">
        <!-- Main panel (3/5) -->
        <div class="space-y-6 lg:col-span-3">
          <!-- Stamp tab (preserved with v-show) -->
          <div v-show="activeTab === 'stamp'" class="space-y-6">
            <HeroTerminal
              @submit="handleStampFromDigest"
              @submit-raw="handleStampFromHash"
            />

            <StampingWorkflow
              v-if="showWorkflow"
              :phase="stampPhase"
              :error="stampError"
              :broadcast-progress="broadcastProgress"
              @download="downloadPendingStamp"
            />
          </div>

          <!-- Verify tab (preserved with v-show) -->
          <div v-show="activeTab === 'verify'">
            <VerificationResult />
          </div>

          <!-- Upgrade tab (preserved with v-show) -->
          <div v-show="activeTab === 'upgrade'">
            <UpgradePanel />
          </div>
        </div>

        <!-- Sidebar (2/5) -->
        <div class="space-y-6 lg:col-span-2">
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
