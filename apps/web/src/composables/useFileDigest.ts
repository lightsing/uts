import { ref } from 'vue'
import { sha256 } from '@noble/hashes/sha2.js'
import { keccak_256 } from '@noble/hashes/sha3.js'
import { hexlify } from 'ethers/utils'
import type { SecureDigestOp, DigestHeader } from '@uts/sdk'

export interface FileDigestResult {
  fileName: string
  fileSize: number
  algorithm: SecureDigestOp
  digest: string
  header: DigestHeader
}

const CHUNK_SIZE = 64 * 1024 // 64KB chunks for streaming hash

export function useFileDigest() {
  const isDigesting = ref(false)
  const progress = ref(0)
  const error = ref<string | null>(null)
  const result = ref<FileDigestResult | null>(null)

  async function digestFile(
    file: File,
    algorithm: SecureDigestOp = 'SHA256',
  ): Promise<FileDigestResult> {
    isDigesting.value = true
    progress.value = 0
    error.value = null
    result.value = null

    try {
      const factory = algorithm === 'KECCAK256' ? keccak_256 : sha256
      const hasher = factory.create()
      const totalSize = file.size
      let processed = 0

      const reader = file.stream().getReader()

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        // Process in sub-chunks for progress reporting
        let offset = 0
        while (offset < value.length) {
          const end = Math.min(offset + CHUNK_SIZE, value.length)
          hasher.update(value.subarray(offset, end))
          offset = end
          processed += end - offset || (end - (offset - (end - offset)))
        }
        processed = Math.min(
          processed,
          totalSize,
        )
        // Recalculate based on actual bytes seen
        progress.value = totalSize > 0 ? Math.min((processed / totalSize) * 100, 100) : 100

        // Yield to main thread periodically
        await new Promise((resolve) => setTimeout(resolve, 0))
      }

      // Fix progress tracking: just use processed bytes
      progress.value = 100

      const digestBytes = hasher.digest()
      const digestHex = hexlify(digestBytes)

      const digestResult: FileDigestResult = {
        fileName: file.webkitRelativePath || file.name,
        fileSize: file.size,
        algorithm,
        digest: digestHex,
        header: {
          kind: algorithm,
          digest: digestBytes,
        },
      }

      result.value = digestResult
      return digestResult
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to digest file'
      error.value = msg
      throw new Error(msg)
    } finally {
      isDigesting.value = false
    }
  }

  function reset() {
    isDigesting.value = false
    progress.value = 0
    error.value = null
    result.value = null
  }

  return {
    isDigesting,
    progress,
    error,
    result,
    digestFile,
    reset,
  }
}
