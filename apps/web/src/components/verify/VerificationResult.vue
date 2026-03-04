<script setup lang="ts">
import { ref } from 'vue'
import { hexlify } from 'ethers/utils'
import {
  FileUp,
  Search,
  ShieldCheck,
  ChevronDown,
  ChevronUp,
  FileCheck,
  FileX,
} from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import StatusBadge from '@/components/base/StatusBadge.vue'
import MerkleTreeViz from '@/components/verify/MerkleTreeViz.vue'
import AttestationDetail from '@/components/verify/AttestationDetail.vue'
import { useTimestampSDK } from '@/composables/useTimestampSDK'
import { useFileDigest } from '@/composables/useFileDigest'
import { useLingui } from '@/composables/useLingui'
import { VerifyStatus } from '@uts/sdk'
import type { DetachedTimestamp, SecureDigestOp } from '@uts/sdk'

const { t } = useLingui()

const {
  verifyStatus,
  verifyAttestations,
  isVerifying,
  verifyError,
  verify,
  decodeOtsFile,
  resetVerify,
} = useTimestampSDK()

const {
  digestFile,
  isDigesting: isDigestingOriginal,
  progress: digestProgress,
} = useFileDigest()

const otsFileRef = ref<HTMLInputElement>()
const originalFileRef = ref<HTMLInputElement>()
const loadedTimestamp = ref<DetachedTimestamp | null>(null)
const showProofPath = ref(false)
const originalFileMatch = ref<'match' | 'mismatch' | null>(null)
const originalFileName = ref<string | null>(null)

function mapVerifyStatus(
  status: VerifyStatus | null,
): 'valid' | 'invalid' | 'pending' | 'partial' | 'unknown' {
  if (!status) return 'unknown'
  switch (status) {
    case VerifyStatus.VALID:
      return 'valid'
    case VerifyStatus.INVALID:
      return 'invalid'
    case VerifyStatus.PENDING:
      return 'pending'
    case VerifyStatus.PARTIAL_VALID:
      return 'partial'
    default:
      return 'unknown'
  }
}

async function handleOtsUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const data = new Uint8Array(await file.arrayBuffer())

  try {
    const decoded = decodeOtsFile(data)
    loadedTimestamp.value = decoded
    await verify(decoded)
  } catch (e) {
    console.error('Failed to decode or verify .ots file:', e)
  }
}

async function handleOriginalFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file || !loadedTimestamp.value) return

  originalFileName.value = file.name
  originalFileMatch.value = null

  try {
    const algo = loadedTimestamp.value.header.kind as SecureDigestOp
    const result = await digestFile(file, algo)
    const headerHex = hexlify(loadedTimestamp.value.header.digest as Uint8Array)
    originalFileMatch.value = result.digest === headerHex ? 'match' : 'mismatch'
  } catch {
    originalFileMatch.value = 'mismatch'
  }
}

function handleReset() {
  resetVerify()
  loadedTimestamp.value = null
  showProofPath.value = false
  originalFileMatch.value = null
  originalFileName.value = null
  if (otsFileRef.value) otsFileRef.value.value = ''
  if (originalFileRef.value) originalFileRef.value.value = ''
}
</script>

<template>
  <GlassCard glow="cyan">
    <div class="mb-4 flex items-center gap-2">
      <ShieldCheck class="h-4 w-4 text-neon-cyan" />
      <h3 class="font-heading text-sm font-semibold text-white/80">
        {{ t('Verification Dashboard') }}
      </h3>
    </div>

    <!-- Upload .ots file -->
    <div v-if="!loadedTimestamp" class="space-y-4">
      <p class="font-mono text-xs text-white/40">
        &gt; {{ t('Upload a .ots file to verify a timestamp proof') }}
      </p>
      <div class="flex gap-2">
        <BaseButton variant="secondary" @click="otsFileRef?.click()">
          <FileUp class="h-4 w-4" />
          {{ t('Upload .ots') }}
        </BaseButton>
      </div>
      <input
        ref="otsFileRef"
        type="file"
        accept=".ots"
        class="hidden"
        @change="handleOtsUpload"
      />
    </div>

    <!-- Verification results -->
    <div v-else class="space-y-5">
      <!-- Status badge -->
      <div class="flex items-center gap-4">
        <StatusBadge
          v-if="verifyStatus !== null"
          :status="mapVerifyStatus(verifyStatus)"
          size="lg"
        />
        <div v-if="isVerifying" class="flex items-center gap-2">
          <Search class="h-4 w-4 animate-spin text-neon-cyan" />
          <span class="font-mono text-xs text-neon-cyan"
            >{{ t('Verifying proof chain...') }}</span
          >
        </div>
      </div>

      <!-- Digest info -->
      <div class="space-y-2 rounded-lg bg-surface/60 p-4">
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Original Digest ({algo})', { algo: loadedTimestamp.header.kind }) }}
        </div>
        <div class="break-all font-mono text-xs text-neon-cyan">
          {{ hexlify(loadedTimestamp.header.digest as Uint8Array) }}
        </div>
      </div>

      <!-- Optional: verify original file matches the header digest -->
      <div
        class="space-y-2 rounded-lg border border-glass-border bg-surface/30 p-4"
      >
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Verify Original File (Optional)') }}
        </div>
        <p class="font-mono text-[10px] text-white/25">
          {{ t('Upload the original file to verify it matches the .ots header digest') }}
        </p>
        <div class="flex items-center gap-3">
          <BaseButton
            variant="secondary"
            @click="originalFileRef?.click()"
            :disabled="isDigestingOriginal"
          >
            <FileUp class="h-3.5 w-3.5" />
            {{ isDigestingOriginal ? t('Digesting...') : t('Upload Original') }}
          </BaseButton>
          <span
            v-if="originalFileName"
            class="font-mono text-[10px] text-white/40"
          >
            {{ originalFileName }}
          </span>
        </div>
        <input
          ref="originalFileRef"
          type="file"
          class="hidden"
          @change="handleOriginalFileUpload"
        />
        <!-- Progress bar -->
        <div
          v-if="isDigestingOriginal"
          class="h-1 w-full overflow-hidden rounded-full bg-surface"
        >
          <div
            class="h-full rounded-full bg-neon-cyan transition-all duration-150"
            :style="{ width: `${digestProgress}%` }"
          />
        </div>
        <!-- Match result -->
        <div
          v-if="originalFileMatch === 'match'"
          class="flex items-center gap-2 rounded-lg bg-valid/10 px-3 py-2"
        >
          <FileCheck class="h-4 w-4 text-valid" />
          <span class="font-mono text-xs text-valid"
            >{{ t('File digest matches — this is the original file') }}</span
          >
        </div>
        <div
          v-else-if="originalFileMatch === 'mismatch'"
          class="flex items-center gap-2 rounded-lg bg-invalid/10 px-3 py-2"
        >
          <FileX class="h-4 w-4 text-invalid" />
          <span class="font-mono text-xs text-invalid"
            >{{ t('File digest does NOT match the .ots header') }}</span
          >
        </div>
      </div>

      <!-- Attestation details (collapsible) -->
      <div v-if="verifyAttestations.length > 0" class="space-y-2">
        <div
          class="font-mono text-[10px] uppercase tracking-widest text-white/30"
        >
          {{ t('Attestations ({count})', { count: verifyAttestations.length }) }}
        </div>
        <AttestationDetail
          v-for="(att, i) in verifyAttestations"
          :key="i"
          :attestation="att"
        />
      </div>

      <!-- Proof path toggle -->
      <button
        class="flex w-full items-center gap-2 rounded-lg border border-glass-border px-4 py-2.5 text-left font-mono text-xs text-white/50 transition hover:border-white/20 hover:text-white/70"
        @click="showProofPath = !showProofPath"
      >
        <component
          :is="showProofPath ? ChevronUp : ChevronDown"
          class="h-4 w-4"
        />
        {{ showProofPath ? t('Hide Proof Path') : t('Show Proof Path') }}
      </button>

      <Transition name="fade">
        <div v-if="showProofPath" class="rounded-lg bg-surface/40 p-4">
          <MerkleTreeViz :steps="loadedTimestamp.timestamp" />
        </div>
      </Transition>

      <!-- Error -->
      <div
        v-if="verifyError"
        class="rounded-lg border border-invalid/20 bg-invalid/5 px-4 py-3 font-mono text-xs text-invalid"
      >
        &gt; {{ verifyError }}
      </div>

      <!-- Reset -->
      <BaseButton variant="secondary" @click="handleReset"> {{ t('Reset') }} </BaseButton>
    </div>
  </GlassCard>
</template>
