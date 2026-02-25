import { ref, shallowRef } from 'vue'
import {
  SDK,
  VerifyStatus,
  Decoder,
} from '@uts/sdk'
import type {
  DetachedTimestamp,
  AttestationStatus,
  UpgradeResult,
  DigestHeader,
} from '@uts/sdk'

export type StampPhase =
  | 'idle'
  | 'hashing'
  | 'generating-nonce'
  | 'building-merkle-tree'
  | 'broadcasting'
  | 'waiting-attestation'
  | 'complete'
  | 'error'

let _sdkInstance: SDK | null = null

export function getSDK(): SDK {
  if (!_sdkInstance) {
    _sdkInstance = new SDK({ timeout: 15000 })
  }
  return _sdkInstance
}

export function useTimestampSDK() {
  const sdk = getSDK()

  const stampPhase = ref<StampPhase>('idle')
  const stampError = ref<string | null>(null)
  const stampResult = shallowRef<DetachedTimestamp[] | null>(null)

  const verifyStatus = ref<VerifyStatus | null>(null)
  const verifyAttestations = shallowRef<AttestationStatus[]>([])
  const isVerifying = ref(false)
  const verifyError = ref<string | null>(null)

  async function stamp(digests: DigestHeader[]): Promise<DetachedTimestamp[]> {
    stampPhase.value = 'hashing'
    stampError.value = null
    stampResult.value = null

    try {
      await delay(400)
      stampPhase.value = 'generating-nonce'
      await delay(300)
      stampPhase.value = 'building-merkle-tree'
      await delay(300)
      stampPhase.value = 'broadcasting'

      const results = await sdk.stamp(digests)

      stampPhase.value = 'waiting-attestation'
      await delay(500)
      stampPhase.value = 'complete'
      stampResult.value = results
      return results
    } catch (e) {
      stampPhase.value = 'error'
      stampError.value = e instanceof Error ? e.message : 'Stamping failed'
      throw e
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
  ): Promise<UpgradeResult[]> {
    return sdk.upgrade(detached)
  }

  function decodeOtsFile(data: Uint8Array): DetachedTimestamp {
    const decoder = new Decoder(data)
    return decoder.readDetachedTimestamp()
  }

  function resetStamp() {
    stampPhase.value = 'idle'
    stampError.value = null
    stampResult.value = null
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
    stamp,
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

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
