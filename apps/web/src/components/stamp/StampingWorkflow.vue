<script setup lang="ts">
import { computed } from 'vue'
import {
  KeyRound,
  GitBranch,
  Radio,
  Blocks,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  ShieldCheck,
  Download,
} from 'lucide-vue-next'
import type { StampPhase } from '@/composables/useTimestampSDK'
import GlassCard from '@/components/base/GlassCard.vue'
import BaseButton from '@/components/base/BaseButton.vue'
import { useLingui } from '@/composables/useLingui'

const { t } = useLingui()

const props = defineProps<{
  phase: StampPhase
  error?: string | null
  broadcastProgress?: string
}>()

const emit = defineEmits<{
  download: []
}>()

interface WorkflowStep {
  id: StampPhase
  labelKey: string
  descriptionKey: string
  icon: typeof KeyRound
}

const steps: WorkflowStep[] = [
  {
    id: 'generating-nonce',
    labelKey: 'Generating Nonce',
    descriptionKey: 'Creating random nonce for privacy',
    icon: KeyRound,
  },
  {
    id: 'building-merkle-tree',
    labelKey: 'Building Merkle Tree',
    descriptionKey: 'Constructing proof tree from leaves',
    icon: GitBranch,
  },
  {
    id: 'broadcasting',
    labelKey: 'Broadcasting',
    descriptionKey: 'Submitting to calendar nodes',
    icon: Radio,
  },
  {
    id: 'building-proof',
    labelKey: 'Building Proof',
    descriptionKey: 'Constructing Merkle proof paths',
    icon: Blocks,
  },
  {
    id: 'complete',
    labelKey: 'Complete',
    descriptionKey: 'Timestamp recorded (pending attestation)',
    icon: CheckCircle2,
  },
  {
    id: 'upgrading',
    labelKey: 'Polling for Upgrade',
    descriptionKey: 'Waiting for on-chain attestation...',
    icon: RefreshCw,
  },
  {
    id: 'upgraded',
    labelKey: 'Upgraded',
    descriptionKey: 'Attestation confirmed on-chain',
    icon: ShieldCheck,
  },
]

const phaseOrder: StampPhase[] = [
  'idle',
  'generating-nonce',
  'building-merkle-tree',
  'broadcasting',
  'building-proof',
  'complete',
  'upgrading',
  'upgraded',
]

const currentIndex = computed(() => phaseOrder.indexOf(props.phase))

function getStepStatus(stepId: StampPhase) {
  if (props.phase === 'error') return 'error'
  const stepIndex = phaseOrder.indexOf(stepId)
  if (stepIndex < currentIndex.value) return 'done'
  if (stepIndex === currentIndex.value) return 'active'
  return 'pending'
}

function getStepDescription(step: WorkflowStep): string {
  if (
    step.id === 'broadcasting' &&
    props.broadcastProgress &&
    getStepStatus(step.id) === 'active'
  ) {
    return t('Calendar responses: {progress}', { progress: props.broadcastProgress })
  }
  return t(step.descriptionKey)
}
</script>

<template>
  <GlassCard glow="purple">
    <div class="mb-4 flex items-center gap-2">
      <GitBranch class="h-4 w-4 text-neon-purple" />
      <h3 class="font-heading text-sm font-semibold text-white/80">
        {{ t('Stamping Pipeline') }}
      </h3>
    </div>

    <div class="space-y-1">
      <div
        v-for="(step, i) in steps"
        :key="step.id"
        class="flex items-start gap-3 rounded-lg px-3 py-2.5 transition-all duration-300"
        :class="{
          'bg-neon-cyan/5': getStepStatus(step.id) === 'active',
          'bg-valid/5': getStepStatus(step.id) === 'done',
        }"
      >
        <!-- Step indicator -->
        <div class="relative mt-0.5 flex flex-col items-center">
          <div
            class="flex h-7 w-7 items-center justify-center rounded-full border transition-all duration-300"
            :class="{
              'border-neon-cyan/50 bg-neon-cyan/10 text-neon-cyan':
                getStepStatus(step.id) === 'active',
              'border-valid/50 bg-valid/10 text-valid':
                getStepStatus(step.id) === 'done',
              'border-glass-border text-white/20':
                getStepStatus(step.id) === 'pending',
              'border-invalid/50 bg-invalid/10 text-invalid':
                getStepStatus(step.id) === 'error',
            }"
          >
            <CheckCircle2
              v-if="getStepStatus(step.id) === 'done'"
              class="h-4 w-4"
            />
            <AlertCircle
              v-else-if="getStepStatus(step.id) === 'error'"
              class="h-4 w-4"
            />
            <component
              :is="step.icon"
              v-else
              class="h-3.5 w-3.5"
              :class="{
                'animate-spin':
                  getStepStatus(step.id) === 'active' &&
                  step.id === 'upgrading',
              }"
            />
          </div>
          <!-- Connecting line -->
          <div
            v-if="i < steps.length - 1"
            class="mt-1 h-3 w-px"
            :class="{
              'bg-valid/30': getStepStatus(step.id) === 'done',
              'bg-neon-cyan/20': getStepStatus(step.id) === 'active',
              'bg-glass-border': getStepStatus(step.id) === 'pending',
            }"
          />
        </div>

        <!-- Step content -->
        <div class="min-w-0 flex-1">
          <div
            class="font-heading text-sm font-medium transition-colors duration-300"
            :class="{
              'text-neon-cyan glow-text-cyan':
                getStepStatus(step.id) === 'active',
              'text-valid': getStepStatus(step.id) === 'done',
              'text-white/30': getStepStatus(step.id) === 'pending',
              'text-invalid': getStepStatus(step.id) === 'error',
            }"
          >
            {{ t(step.labelKey) }}
          </div>
          <div class="font-mono text-[11px] text-white/30">
            {{ getStepDescription(step) }}
          </div>

          <!-- Download button inside the Complete step — visible while pending (complete or upgrading), hidden once upgraded -->
          <div
            v-if="
              step.id === 'complete' &&
              (phase === 'complete' || phase === 'upgrading')
            "
            class="mt-2"
          >
            <BaseButton variant="secondary" @click="emit('download')">
              <Download class="h-3.5 w-3.5" />
              {{ t('Download pending .ots') }}
            </BaseButton>
          </div>
        </div>

        <!-- Active indicator -->
        <div
          v-if="getStepStatus(step.id) === 'active'"
          class="mt-1 h-1.5 w-1.5 animate-glow-pulse rounded-full bg-neon-cyan"
        />
      </div>
    </div>

    <!-- Error message -->
    <div
      v-if="phase === 'error' && error"
      class="mt-4 rounded-lg border border-invalid/20 bg-invalid/5 px-4 py-3 font-mono text-xs text-invalid"
    >
      &gt; {{ error }}
    </div>
  </GlassCard>
</template>
