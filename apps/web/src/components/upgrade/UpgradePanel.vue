<script setup lang="ts">
import { ref } from 'vue'
import { toHex } from 'viem'
import {
  FileUp,
  RefreshCw,
  Download,
  CheckCircle2,
  AlertTriangle,
} from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import AttestationDetail from '@/components/verify/AttestationDetail.vue'
import { useTimestampSDK, downloadOtsFile } from '@/composables/useTimestampSDK'
import { useLingui } from '@/composables/useLingui'
import { UpgradeStatus } from '@uts/sdk'
import type { DetachedTimestamp, UpgradeResult } from '@uts/sdk'
import { useAppStore } from '@/stores/app'

const { t } = useLingui()

const store = useAppStore()
const { upgrade, decodeOtsFile, verify, verifyAttestations } = useTimestampSDK()

const otsFileRef = ref<HTMLInputElement>()
const loadedTimestamp = ref<DetachedTimestamp | null>(null)
const originalFileName = ref('')
const isUpgrading = ref(false)
const upgradeError = ref<string | null>(null)
const upgradeResults = ref<UpgradeResult[] | null>(null)
const hasUpgraded = ref(false)

async function handleOtsUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  originalFileName.value = file.name.replace(/\.ots$/, '')
  upgradeError.value = null
  upgradeResults.value = null
  hasUpgraded.value = false

  const data = new Uint8Array(await file.arrayBuffer())
  try {
    const decoded = decodeOtsFile(data)
    loadedTimestamp.value = decoded
    await verify(decoded)
  } catch (e) {
    upgradeError.value =
      e instanceof Error ? e.message : 'Failed to decode .ots file'
  }
}

async function handleUpgrade() {
  if (!loadedTimestamp.value) return
  isUpgrading.value = true
  upgradeError.value = null
  upgradeResults.value = null

  try {
    const results = await upgrade(loadedTimestamp.value, store.keepPending)
    upgradeResults.value = results
    if (results.some((r) => r.status === UpgradeStatus.Upgraded)) {
      hasUpgraded.value = true
    }
  } catch (e) {
    upgradeError.value = e instanceof Error ? e.message : 'Upgrade failed'
  } finally {
    isUpgrading.value = false
  }
}

function handleDownload() {
  if (!loadedTimestamp.value) return
  downloadOtsFile(loadedTimestamp.value, originalFileName.value || undefined)
}

function handleReset() {
  loadedTimestamp.value = null
  originalFileName.value = ''
  isUpgrading.value = false
  upgradeError.value = null
  upgradeResults.value = null
  hasUpgraded.value = false
  if (otsFileRef.value) otsFileRef.value.value = ''
}
</script>

<template>
  <GlassCard glow="purple">
    <div class="mb-4 flex items-center gap-2">
      <RefreshCw class="h-4 w-4 text-neon-purple" />
      <h3 class="font-heading text-sm font-semibold text-white/80">
        {{ t('Manual Upgrade') }}
      </h3>
    </div>

    <!-- Upload .ots file -->
    <div v-if="!loadedTimestamp" class="space-y-4">
      <p class="font-mono text-xs text-white/40">
        &gt; {{ t('Upload a pending .ots file to upgrade it with on-chain attestations') }}
      </p>
      <BaseButton variant="secondary" @click="otsFileRef?.click()">
        <FileUp class="h-4 w-4" />
        {{ t('Upload .ots') }}
      </BaseButton>
      <input
        ref="otsFileRef"
        type="file"
        accept=".ots"
        class="hidden"
        @change="handleOtsUpload"
      />
    </div>

    <!-- Loaded file -->
    <div v-else class="space-y-5">
      <!-- Digest info -->
      <div class="space-y-2 rounded-lg bg-surface/60 p-4">
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Digest ({algo})', { algo: loadedTimestamp.header.kind }) }}
        </div>
        <div class="break-all font-mono text-xs text-neon-cyan">
          {{ toHex(loadedTimestamp.header.digest as Uint8Array) }}
        </div>
      </div>

      <!-- Current attestations -->
      <div v-if="verifyAttestations.length > 0" class="space-y-2">
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Current Attestations ({count})', { count: verifyAttestations.length }) }}
        </div>
        <AttestationDetail
          v-for="(att, i) in verifyAttestations"
          :key="i"
          :attestation="att"
        />
      </div>

      <!-- Upgrade results -->
      <div v-if="upgradeResults" class="space-y-2">
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Upgrade Results') }}
        </div>
        <div
          v-for="(result, i) in upgradeResults"
          :key="i"
          class="flex items-center gap-2 rounded-lg border border-glass-border bg-surface/40 px-3 py-2"
        >
          <CheckCircle2
            v-if="result.status === UpgradeStatus.Upgraded"
            class="h-4 w-4 text-valid"
          />
          <RefreshCw
            v-else-if="result.status === UpgradeStatus.Pending"
            class="h-4 w-4 text-pending"
          />
          <AlertTriangle v-else class="h-4 w-4 text-invalid" />
          <span class="font-mono text-xs text-white/60">
            {{ result.original.url }} — {{ result.status }}
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex flex-wrap gap-2">
        <BaseButton
          variant="primary"
          :disabled="isUpgrading"
          @click="handleUpgrade"
        >
          <RefreshCw class="h-4 w-4" :class="{ 'animate-spin': isUpgrading }" />
          {{ isUpgrading ? t('Upgrading...') : t('Upgrade Now') }}
        </BaseButton>

        <BaseButton
          v-if="hasUpgraded"
          variant="secondary"
          @click="handleDownload"
        >
          <Download class="h-4 w-4" />
          {{ t('Download Upgraded .ots') }}
        </BaseButton>

        <BaseButton variant="secondary" @click="handleReset">
          {{ t('Reset') }}
        </BaseButton>
      </div>

      <!-- Error -->
      <div
        v-if="upgradeError"
        class="rounded-lg border border-invalid/20 bg-invalid/5 px-4 py-3 font-mono text-xs text-invalid"
      >
        &gt; {{ upgradeError }}
      </div>
    </div>
  </GlassCard>
</template>
