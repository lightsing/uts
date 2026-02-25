import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { DEFAULT_CALENDARS } from '@uts/sdk'
import type { DetachedTimestamp } from '@uts/sdk'

export interface CalendarNode {
  url: string
  status: 'online' | 'offline' | 'checking'
  latency?: number
}

export const useAppStore = defineStore('app', () => {
  const calendars = ref<CalendarNode[]>(
    DEFAULT_CALENDARS.map((url) => ({
      url: url.toString(),
      status: 'checking' as const,
    })),
  )

  const recentStamps = ref<DetachedTimestamp[]>([])

  const onlineCount = computed(
    () => calendars.value.filter((c) => c.status === 'online').length,
  )

  async function checkCalendars() {
    for (const cal of calendars.value) {
      cal.status = 'checking'
      const start = performance.now()
      try {
        const response = await fetch(cal.url, {
          method: 'HEAD',
          mode: 'no-cors',
          signal: AbortSignal.timeout(5000),
        })
        // no-cors returns opaque response; treat as online
        cal.latency = Math.round(performance.now() - start)
        cal.status = response.type === 'opaque' || response.ok ? 'online' : 'offline'
      } catch {
        cal.status = 'offline'
        cal.latency = undefined
      }
    }
  }

  function addStamp(stamp: DetachedTimestamp) {
    recentStamps.value.unshift(stamp)
    if (recentStamps.value.length > 20) {
      recentStamps.value.pop()
    }
  }

  return {
    calendars,
    recentStamps,
    onlineCount,
    checkCalendars,
    addStamp,
  }
})
