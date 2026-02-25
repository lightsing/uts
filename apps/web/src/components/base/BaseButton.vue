<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'danger'
  disabled?: boolean
  loading?: boolean
}>()

defineEmits<{
  click: [e: MouseEvent]
}>()
</script>

<template>
  <button
    class="relative inline-flex items-center justify-center gap-2 rounded-lg px-5 py-2.5 font-heading text-sm font-semibold tracking-wide uppercase transition-all duration-200 active:scale-95 disabled:pointer-events-none disabled:opacity-40"
    :class="{
      'border border-neon-cyan/40 bg-neon-cyan/10 text-neon-cyan hover:bg-neon-cyan/20 hover:shadow-[0_0_20px_rgba(0,243,255,0.2)]':
        variant === 'primary' || !variant,
      'border border-glass-border bg-glass text-white/70 hover:border-white/20 hover:text-white':
        variant === 'secondary',
      'border border-invalid/40 bg-invalid/10 text-invalid hover:bg-invalid/20':
        variant === 'danger',
    }"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <svg
      v-if="loading"
      class="h-4 w-4 animate-spin"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
    <slot />
  </button>
</template>
