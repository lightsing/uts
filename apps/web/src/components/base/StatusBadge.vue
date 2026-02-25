<script setup lang="ts">
defineProps<{
  status: 'valid' | 'invalid' | 'pending' | 'partial' | 'unknown'
  size?: 'sm' | 'md' | 'lg'
}>()

const labels: Record<string, string> = {
  valid: 'VERIFIED ON CHAIN',
  invalid: 'INVALID',
  pending: 'PENDING',
  partial: 'PARTIALLY VERIFIED',
  unknown: 'UNKNOWN',
}
</script>

<template>
  <span
    class="inline-flex items-center gap-2 rounded-full font-mono font-bold uppercase tracking-widest"
    :class="[
      {
        'bg-valid/10 text-valid border border-valid/30': status === 'valid',
        'bg-invalid/10 text-invalid border border-invalid/30':
          status === 'invalid',
        'bg-pending/10 text-pending border border-pending/30':
          status === 'pending',
        'bg-neon-purple/10 text-neon-purple border border-neon-purple/30':
          status === 'partial',
        'bg-white/5 text-white/50 border border-white/10': status === 'unknown',
      },
      {
        'px-3 py-1 text-xs': size === 'sm' || !size,
        'px-5 py-2 text-sm': size === 'md',
        'px-8 py-3 text-lg': size === 'lg',
      },
    ]"
  >
    <span
      class="h-2 w-2 rounded-full"
      :class="{
        'bg-valid animate-glow-pulse': status === 'valid',
        'bg-invalid': status === 'invalid',
        'bg-pending animate-glow-pulse': status === 'pending',
        'bg-neon-purple animate-glow-pulse': status === 'partial',
        'bg-white/30': status === 'unknown',
      }"
    />
    {{ labels[status] }}
  </span>
</template>
