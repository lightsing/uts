import { ref, shallowRef } from 'vue'
import { SDK, VerifyStatus, UpgradeStatus, Decoder, Encoder } from '@uts/sdk'
import type {
  DetachedTimestamp,
  AttestationStatus,
  UpgradeResult,
  DigestHeader,
  StampEventCallback,
} from '@uts/sdk'
import type { EIP1193Provider } from '@uts/sdk'
import JSZip from 'jszip'

export type StampPhase =
  | 'idle'
  | 'generating-nonce'
  | 'building-merkle-tree'
  | 'broadcasting'
  | 'waiting-attestation'
  | 'building-proof'
  | 'complete'
  | 'upgrading'
  | 'upgraded'
  | 'error'

let _sdkInstance: SDK | null = null

export function getSDK(): SDK {
  if (!_sdkInstance) {
    _sdkInstance = new SDK({ timeout: 15000 })
  }
  return _sdkInstance
}

export function resetSDK(options?: {
  calendars?: URL[]
  web3Provider?: EIP1193Provider | null
}) {
  _sdkInstance = new SDK({
    timeout: 15000,
    calendars: options?.calendars,
    web3Provider: options?.web3Provider,
  })
}

export function setWeb3Provider(provider: EIP1193Provider | null) {
  const sdk = getSDK()
  sdk.web3Provider = provider
}

function triggerDownload(blob: Blob, fileName: string) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = fileName
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export function downloadOtsFile(stamp: DetachedTimestamp, fileName?: string) {
  const encoded = Encoder.encodeDetachedTimestamp(stamp)
  const blob = new Blob([encoded as BlobPart], {
    type: 'application/octet-stream',
  })
  triggerDownload(blob, fileName ? `${fileName}.ots` : 'timestamp.ots')
}

async function downloadStampsAsZip(
  stamps: DetachedTimestamp[],
  names: string[],
) {
  const zip = new JSZip()
  for (let i = 0; i < stamps.length; i++) {
    const fileName = names[i] ?? `file-${i}`
    const encoded = Encoder.encodeDetachedTimestamp(stamps[i]!)
    // Preserve directory structure in zip (fileName may contain path separators)
    zip.file(`${fileName}.ots`, encoded)
  }
  const blob = await zip.generateAsync({ type: 'blob' })
  triggerDownload(blob, 'timestamps.zip')
}

export function useTimestampSDK() {
  const sdk = getSDK()

  const stampPhase = ref<StampPhase>('idle')
  const stampError = ref<string | null>(null)
  const stampResult = shallowRef<DetachedTimestamp[] | null>(null)
  const stampFileNames = ref<string[]>([])
  const broadcastProgress = ref('')
  const upgradeResults = shallowRef<UpgradeResult[] | null>(null)

  const verifyStatus = ref<VerifyStatus | null>(null)
  const verifyAttestations = shallowRef<AttestationStatus[]>([])
  const isVerifying = ref(false)
  const verifyError = ref<string | null>(null)

  let upgradeTimer: ReturnType<typeof setInterval> | null = null

  async function stamp(
    digests: DigestHeader[],
    fileNames?: string[],
  ): Promise<DetachedTimestamp[]> {
    stampPhase.value = 'generating-nonce'
    stampError.value = null
    stampResult.value = null
    stampFileNames.value = fileNames ?? []
    broadcastProgress.value = ''
    upgradeResults.value = null

    const onEvent: StampEventCallback = (event) => {
      switch (event.phase) {
        case 'generating-nonce':
          stampPhase.value = 'generating-nonce'
          break
        case 'building-merkle-tree':
          stampPhase.value = 'building-merkle-tree'
          break
        case 'broadcasting':
          stampPhase.value = 'broadcasting'
          broadcastProgress.value = `0/${event.totalCalendars}`
          break
        case 'calendar-response':
          broadcastProgress.value = `${event.responsesReceived}/${event.totalCalendars}${event.success ? '' : ' (failed: ' + event.calendarUrl + ')'}`
          break
        case 'building-proof':
          stampPhase.value = 'building-proof'
          break
        case 'complete':
          stampPhase.value = 'complete'
          break
      }
    }

    try {
      const results = await sdk.stamp(digests, onEvent)

      stampPhase.value = 'complete'
      stampResult.value = results

      // Start polling for upgrade (no auto-download — user uses the download button)
      startUpgradePolling(results)

      return results
    } catch (e) {
      stampPhase.value = 'error'
      stampError.value = e instanceof Error ? e.message : 'Stamping failed'
      throw e
    }
  }

  async function downloadPendingStamp() {
    if (!stampResult.value) return
    const results = stampResult.value
    const names = stampFileNames.value

    if (results.length === 1) {
      downloadOtsFile(results[0]!, names[0])
    } else if (results.length > 1) {
      await downloadStampsAsZip(results, names)
    }
  }

  function startUpgradePolling(
    stamps: DetachedTimestamp[],
    keepPending?: boolean,
  ) {
    stopUpgradePolling()
    stampPhase.value = 'upgrading'

    let attempts = 0
    const maxAttempts = 40 // ~10 minutes at 15s intervals

    upgradeTimer = setInterval(async () => {
      attempts++
      try {
        const allResults: UpgradeResult[] = []
        for (const s of stamps) {
          const results = await sdk.upgrade(s, keepPending ?? false)
          allResults.push(...results)
        }
        upgradeResults.value = allResults

        const hasUpgraded = allResults.some(
          (r) => r.status === UpgradeStatus.Upgraded,
        )
        if (hasUpgraded) {
          stampPhase.value = 'upgraded'
          // Auto-download upgraded timestamps with correct per-file names
          const names = stampFileNames.value
          if (stamps.length === 1) {
            downloadOtsFile(stamps[0]!, names[0])
          } else if (stamps.length > 1) {
            await downloadStampsAsZip(stamps, names)
          }
          stopUpgradePolling()
        } else if (attempts >= maxAttempts) {
          stopUpgradePolling()
        }
      } catch {
        // Silently retry on next interval
      }
    }, 15000)
  }

  function stopUpgradePolling() {
    if (upgradeTimer) {
      clearInterval(upgradeTimer)
      upgradeTimer = null
    }
  }

  async function verify(
    stamp: DetachedTimestamp,
  ): Promise<{ status: VerifyStatus; attestations: AttestationStatus[] }> {
    isVerifying.value = true
    verifyError.value = null
    verifyStatus.value = null
    verifyAttestations.value = []

    try {
      const attestations = await sdk.verify(stamp)
      const status = sdk.transformResult(attestations)

      verifyStatus.value = status
      verifyAttestations.value = attestations
      return { status, attestations }
    } catch (e) {
      verifyError.value = e instanceof Error ? e.message : 'Verification failed'
      throw e
    } finally {
      isVerifying.value = false
    }
  }

  async function upgrade(
    detached: DetachedTimestamp,
    keepPending?: boolean,
  ): Promise<UpgradeResult[]> {
    return sdk.upgrade(detached, keepPending ?? false)
  }

  function decodeOtsFile(data: Uint8Array): DetachedTimestamp {
    const decoder = new Decoder(data)
    return decoder.readDetachedTimestamp()
  }

  function resetStamp() {
    stampPhase.value = 'idle'
    stampError.value = null
    stampResult.value = null
    stampFileNames.value = []
    broadcastProgress.value = ''
    upgradeResults.value = null
    stopUpgradePolling()
  }

  function resetVerify() {
    verifyStatus.value = null
    verifyAttestations.value = []
    isVerifying.value = false
    verifyError.value = null
  }

  return {
    stampPhase,
    stampError,
    stampResult,
    stampFileNames,
    broadcastProgress,
    upgradeResults,
    stamp,
    downloadPendingStamp,
    resetStamp,

    verifyStatus,
    verifyAttestations,
    isVerifying,
    verifyError,
    verify,
    upgrade,
    decodeOtsFile,
    resetVerify,
  }
}
