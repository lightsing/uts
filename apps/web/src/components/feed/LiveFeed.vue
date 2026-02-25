<script setup lang="ts">
import { onMounted } from 'vue'
import { formatDistanceToNow } from 'date-fns'
import { Activity, Globe, Clock } from 'lucide-vue-next'
import GlassCard from '@/components/base/GlassCard.vue'
import { useWebSocketFeed } from '@/composables/useWebSocketFeed'

const { entries, isConnected, connect } = useWebSocketFeed()

onMounted(() => {
  connect()
})

function truncate(hash: string): string {
  return `${hash.slice(0, 10)}...${hash.slice(-8)}`
}
</script>

<template>
  <GlassCard>
    <div class="mb-4 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Activity class="h-4 w-4 text-neon-cyan" />
        <h3 class="font-heading text-sm font-semibold text-white/80">Live Feed</h3>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="h-2 w-2 rounded-full"
          :class="isConnected ? 'bg-valid animate-glow-pulse' : 'bg-invalid'"
        />
        <span class="font-mono text-[10px] text-white/40">
          {{ isConnected ? 'POLLING' : 'DISCONNECTED' }}
        </span>
      </div>
    </div>

    <div class="max-h-80 space-y-1 overflow-y-auto pr-1">
      <TransitionGroup name="list">
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-white/5"
        >
          <Globe class="h-4 w-4 shrink-0 text-neon-purple" />
          <div class="min-w-0 flex-1">
            <div class="font-mono text-xs text-neon-purple">
              {{ truncate(entry.hash) }}
            </div>
            <div class="flex items-center gap-2 font-mono text-[10px] text-white/30">
              <span>{{ entry.chain }}</span>
              <span>#{{ entry.blockHeight }}</span>
            </div>
          </div>
          <div class="shrink-0 font-mono text-[10px] text-white/20">
            {{ formatDistanceToNow(entry.timestamp, { addSuffix: true }) }}
          </div>
        </div>
      </TransitionGroup>

      <div
        v-if="entries.length === 0 && isConnected"
        class="py-8 text-center font-mono text-xs text-white/30"
      >
        <Clock class="mx-auto mb-2 h-5 w-5 text-white/20" />
        Polling Ethereum RPCs for Attested events...
      </div>
      <div
        v-else-if="entries.length === 0"
        class="py-8 text-center font-mono text-xs text-white/30"
      >
        Waiting for connection...
      </div>
    </div>
  </GlassCard>
</template>
