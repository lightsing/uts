<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDropZone } from '@vueuse/core'
import { Upload, Hash, FileText, FolderOpen, X } from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import { useFileDigest, type FileDigestResult } from '@/composables/useFileDigest'
import type { SecureDigestOp } from '@uts/sdk'

const emit = defineEmits<{
  submit: [digests: FileDigestResult[]]
  submitRaw: [hash: string]
}>()

const dropZoneRef = ref<HTMLDivElement>()
const fileInputRef = ref<HTMLInputElement>()
const dirInputRef = ref<HTMLInputElement>()
const manualHash = ref('')
const selectedAlgorithm = ref<SecureDigestOp>('KECCAK256')

const { isDigesting, progress, error: digestError, result: digestResult, digestFile, reset: resetDigest } = useFileDigest()

const { isOverDropZone } = useDropZone(dropZoneRef, {
  onDrop: handleDrop,
  dataTypes: ['Files'],
})

// Store selected files (not yet hashed)
const selectedFiles = ref<File[]>([])
const isStamping = ref(false)

const hasFiles = computed(() => selectedFiles.value.length > 0)
const hasResult = computed(() => digestResult.value !== null)
const canSubmit = computed(() => (hasFiles.value || manualHash.value.trim()) && !isStamping.value)

function handleDrop(files: File[] | null) {
  if (files && files.length > 0) {
    selectedFiles.value = [...files]
    resetDigest()
  }
}

function handleFileInput(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    selectedFiles.value = Array.from(target.files)
    resetDigest()
  }
}

function handleDirInput(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    selectedFiles.value = Array.from(target.files)
    resetDigest()
  }
}

function openFilePicker() {
  fileInputRef.value?.click()
}

function openDirPicker() {
  dirInputRef.value?.click()
}

async function handleSubmit() {
  if (hasFiles.value) {
    isStamping.value = true
    try {
      const results: FileDigestResult[] = []
      for (const file of selectedFiles.value) {
        const result = await digestFile(file, selectedAlgorithm.value)
        results.push(result)
      }
      emit('submit', results)
    } catch {
      // error tracked in digestError
    } finally {
      isStamping.value = false
    }
  } else if (manualHash.value.trim()) {
    emit('submitRaw', manualHash.value.trim())
  }
}

function handleClear() {
  selectedFiles.value = []
  resetDigest()
  manualHash.value = ''
  if (fileInputRef.value) fileInputRef.value.value = ''
  if (dirInputRef.value) dirInputRef.value.value = ''
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function totalSize(): number {
  return selectedFiles.value.reduce((sum, f) => sum + f.size, 0)
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
            <option value="KECCAK256">Keccak-256</option>
            <option value="SHA256">SHA-256</option>
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
        'border-glass-border hover:border-white/20': !isOverDropZone && !hasFiles && !isDigesting && !hasResult,
        'border-valid/30 bg-valid/5': hasFiles || hasResult,
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

      <!-- Files selected (not yet hashed) -->
      <div v-else-if="hasFiles" class="space-y-3">
        <div class="flex items-center justify-center gap-2">
          <FileText class="h-5 w-5 text-neon-cyan" />
          <span class="font-heading text-sm font-semibold text-neon-cyan">
            {{ selectedFiles.length }} file{{ selectedFiles.length > 1 ? 's' : '' }} selected
          </span>
          <button
            class="ml-2 rounded p-1 text-white/30 transition hover:bg-white/10 hover:text-white"
            @click="handleClear"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
        <div class="mx-auto max-w-md space-y-1">
          <div
            v-for="(file, i) in selectedFiles.slice(0, 5)"
            :key="i"
            class="font-mono text-xs text-white/50"
          >
            {{ file.name }} <span class="text-white/30">({{ formatSize(file.size) }})</span>
          </div>
          <div v-if="selectedFiles.length > 5" class="font-mono text-xs text-white/30">
            ... and {{ selectedFiles.length - 5 }} more
          </div>
        </div>
        <div class="font-mono text-[10px] text-white/30">
          Total: {{ formatSize(totalSize()) }} · Hash on stamp with {{ selectedAlgorithm }}
        </div>
      </div>

      <!-- Default empty state -->
      <div v-else class="space-y-4">
        <Upload class="mx-auto h-10 w-10 text-white/20" />
        <div class="space-y-1">
          <p class="font-heading text-sm font-medium text-white/60">
            Drop files here
          </p>
          <p class="font-mono text-xs text-white/30">or</p>
        </div>
        <div class="flex items-center justify-center gap-2">
          <BaseButton variant="secondary" @click="openFilePicker">
            <FileText class="h-4 w-4" />
            Choose File
          </BaseButton>
          <BaseButton variant="secondary" @click="openDirPicker">
            <FolderOpen class="h-4 w-4" />
            Choose Directory
          </BaseButton>
        </div>
      </div>

      <input
        ref="fileInputRef"
        type="file"
        multiple
        class="hidden"
        @change="handleFileInput"
      />
      <!-- Directory picker (webkitdirectory) -->
      <input
        ref="dirInputRef"
        type="file"
        class="hidden"
        webkitdirectory
        @change="handleDirInput"
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
          :disabled="hasFiles"
          @keyup.enter="handleSubmit"
        />
        <BaseButton
          :disabled="!canSubmit"
          :loading="isDigesting || isStamping"
          @click="handleSubmit"
        >
          <Hash class="h-4 w-4" />
          Stamp
        </BaseButton>
      </div>
    </div>
  </GlassCard>
</template>
