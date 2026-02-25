<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDropZone } from '@vueuse/core'
import { Upload, Hash, FileText, X } from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import { useFileDigest, type FileDigestResult } from '@/composables/useFileDigest'
import type { SecureDigestOp } from '@uts/sdk'

const emit = defineEmits<{
  submit: [digest: FileDigestResult]
  submitRaw: [hash: string]
}>()

const dropZoneRef = ref<HTMLDivElement>()
const fileInputRef = ref<HTMLInputElement>()
const manualHash = ref('')
const selectedAlgorithm = ref<SecureDigestOp>('SHA256')

const { isDigesting, progress, error: digestError, result: digestResult, digestFile, reset: resetDigest } = useFileDigest()

const { isOverDropZone } = useDropZone(dropZoneRef, {
  onDrop: handleDrop,
  dataTypes: ['Files'],
})

const hasResult = computed(() => digestResult.value !== null)

function handleDrop(files: File[] | null) {
  if (files && files.length > 0) {
    processFile(files[0]!)
  }
}

function handleFileInput(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    processFile(file)
  }
}

function openFilePicker() {
  fileInputRef.value?.click()
}

async function processFile(file: File) {
  await digestFile(file, selectedAlgorithm.value)
}

function handleSubmit() {
  if (digestResult.value) {
    emit('submit', digestResult.value)
  } else if (manualHash.value.trim()) {
    emit('submitRaw', manualHash.value.trim())
  }
}

function handleClear() {
  resetDigest()
  manualHash.value = ''
  if (fileInputRef.value) fileInputRef.value.value = ''
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <GlassCard glow="cyan">
    <!-- Terminal header -->
    <div class="mb-4 flex items-center gap-2">
      <div class="flex gap-1.5">
        <span class="h-3 w-3 rounded-full bg-invalid/60" />
        <span class="h-3 w-3 rounded-full bg-pending/60" />
        <span class="h-3 w-3 rounded-full bg-valid/60" />
      </div>
      <span class="ml-2 font-mono text-xs text-white/40">uts://terminal</span>
      <div class="ml-auto flex items-center gap-3">
        <label class="flex items-center gap-2 text-xs text-white/50">
          <span class="font-mono">algo:</span>
          <select
            v-model="selectedAlgorithm"
            class="rounded border border-glass-border bg-surface px-2 py-0.5 font-mono text-xs text-neon-cyan outline-none focus:border-neon-cyan/40"
          >
            <option value="SHA256">SHA-256</option>
            <option value="KECCAK256">Keccak-256</option>
          </select>
        </label>
      </div>
    </div>

    <!-- Drop zone / file input area -->
    <div
      ref="dropZoneRef"
      class="relative rounded-lg border-2 border-dashed p-8 text-center transition-all duration-300"
      :class="{
        'border-neon-cyan/50 bg-neon-cyan/5': isOverDropZone,
        'border-glass-border hover:border-white/20': !isOverDropZone && !hasResult,
        'border-valid/30 bg-valid/5': hasResult,
      }"
    >
      <!-- Digesting progress -->
      <div v-if="isDigesting" class="space-y-4">
        <Hash class="mx-auto h-10 w-10 animate-spin text-neon-cyan" />
        <div class="font-mono text-sm text-neon-cyan">
          &gt; Computing {{ selectedAlgorithm }} digest...
        </div>
        <div class="mx-auto h-1.5 w-64 overflow-hidden rounded-full bg-surface">
          <div
            class="h-full rounded-full bg-neon-cyan transition-all duration-200"
            :style="{ width: `${progress}%` }"
          />
        </div>
        <div class="font-mono text-xs text-white/40">{{ progress.toFixed(0) }}%</div>
      </div>

      <!-- Result display -->
      <div v-else-if="hasResult && digestResult" class="space-y-3">
        <div class="flex items-center justify-center gap-2">
          <FileText class="h-5 w-5 text-valid" />
          <span class="font-heading text-sm font-semibold text-valid">File Digested</span>
          <button
            class="ml-2 rounded p-1 text-white/30 transition hover:bg-white/10 hover:text-white"
            @click="handleClear"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
        <div class="font-mono text-xs text-white/50">
          {{ digestResult.fileName }} ({{ formatSize(digestResult.fileSize) }})
        </div>
        <div
          class="mx-auto max-w-lg break-all rounded bg-surface/80 px-4 py-2 font-mono text-xs text-neon-cyan"
        >
          {{ digestResult.digest }}
        </div>
        <div class="font-mono text-[10px] uppercase tracking-widest text-white/30">
          {{ digestResult.algorithm }}
        </div>
      </div>

      <!-- Default empty state -->
      <div v-else class="space-y-4">
        <Upload class="mx-auto h-10 w-10 text-white/20" />
        <div class="space-y-1">
          <p class="font-heading text-sm font-medium text-white/60">
            Drop a file here to compute its digest
          </p>
          <p class="font-mono text-xs text-white/30">or</p>
        </div>
        <BaseButton variant="secondary" @click="openFilePicker">
          <FileText class="h-4 w-4" />
          Choose File
        </BaseButton>
      </div>

      <input
        ref="fileInputRef"
        type="file"
        class="hidden"
        @change="handleFileInput"
      />
    </div>

    <!-- Error display -->
    <div v-if="digestError" class="mt-3 rounded bg-invalid/10 px-3 py-2 font-mono text-xs text-invalid">
      &gt; Error: {{ digestError }}
    </div>

    <!-- Manual hash input -->
    <div class="mt-4 space-y-2">
      <div class="flex items-center gap-2 font-mono text-xs text-white/40">
        <span class="text-neon-cyan">&gt;</span>
        <span>Or paste a hash directly:</span>
      </div>
      <div class="flex gap-2">
        <input
          v-model="manualHash"
          type="text"
          placeholder="0x..."
          class="flex-1 rounded-lg border border-glass-border bg-surface px-4 py-2.5 font-mono text-sm text-white/80 outline-none transition placeholder:text-white/20 focus:border-neon-cyan/40"
          :disabled="hasResult"
          @keyup.enter="handleSubmit"
        />
        <BaseButton
          :disabled="!hasResult && !manualHash.trim()"
          :loading="isDigesting"
          @click="handleSubmit"
        >
          <Hash class="h-4 w-4" />
          Stamp
        </BaseButton>
      </div>
    </div>
  </GlassCard>
</template>
