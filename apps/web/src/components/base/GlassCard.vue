<script setup lang="ts">
defineProps<{
  glow?: 'cyan' | 'purple' | 'none'
}>()
</script>

<template>
  <div
    class="glass rounded-xl p-6 transition-all duration-300"
    :class="{
      'glow-cyan': glow === 'cyan',
      'glow-purple': glow === 'purple',
    }"
  >
    <slot />
  </div>
</template>
