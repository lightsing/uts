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

// Estimated browser hashing throughput (~200 MB/s), used for fake progress
const ESTIMATED_THROUGHPUT = 200 * 1024 * 1024
const MIN_DURATION_MS = 300
const MAX_DURATION_MS = 30000
const PROGRESS_CAP = 92 // fake progress caps at this before real completion

export function useFileDigest() {
  const isDigesting = ref(false)
  const progress = ref(0)
  const error = ref<string | null>(null)
  const result = ref<FileDigestResult | null>(null)

  let progressTimer: ReturnType<typeof setInterval> | null = null

  function startFakeProgress(totalBytes: number, fileCount: number) {
    progress.value = 0
    const estimatedMs = Math.max(
      MIN_DURATION_MS,
      Math.min(
        ((totalBytes * fileCount) / ESTIMATED_THROUGHPUT) * 1000,
        MAX_DURATION_MS,
      ),
    )
    const intervalMs = 50
    const steps = estimatedMs / intervalMs
    const increment = PROGRESS_CAP / steps

    progressTimer = setInterval(() => {
      progress.value = Math.min(progress.value + increment, PROGRESS_CAP)
    }, intervalMs)
  }

  function stopFakeProgress() {
    if (progressTimer) {
      clearInterval(progressTimer)
      progressTimer = null
    }
    progress.value = 100
  }

  async function digestFile(
    file: File,
    algorithm: SecureDigestOp = 'SHA256',
  ): Promise<FileDigestResult> {
    isDigesting.value = true
    progress.value = 0
    error.value = null
    result.value = null

    startFakeProgress(file.size, 1)

    try {
      const factory = algorithm === 'KECCAK256' ? keccak_256 : sha256
      const hasher = factory.create()

      const reader = file.stream().getReader()
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        hasher.update(value)
        // Yield to main thread periodically for UI updates
        await new Promise((resolve) => setTimeout(resolve, 0))
      }

      stopFakeProgress()

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
      stopFakeProgress()
      const msg = e instanceof Error ? e.message : 'Failed to digest file'
      error.value = msg
      throw new Error(msg)
    } finally {
      isDigesting.value = false
    }
  }

  async function digestFiles(
    files: File[],
    algorithm: SecureDigestOp = 'SHA256',
  ): Promise<FileDigestResult[]> {
    isDigesting.value = true
    progress.value = 0
    error.value = null
    result.value = null

    const totalBytes = files.reduce((sum, f) => sum + f.size, 0)
    startFakeProgress(totalBytes, files.length)

    try {
      const results: FileDigestResult[] = []
      for (const file of files) {
        const factory = algorithm === 'KECCAK256' ? keccak_256 : sha256
        const hasher = factory.create()

        const reader = file.stream().getReader()
        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          hasher.update(value)
          await new Promise((resolve) => setTimeout(resolve, 0))
        }

        const digestBytes = hasher.digest()
        results.push({
          fileName: file.webkitRelativePath || file.name,
          fileSize: file.size,
          algorithm,
          digest: hexlify(digestBytes),
          header: { kind: algorithm, digest: digestBytes },
        })
      }

      stopFakeProgress()
      if (results.length > 0) result.value = results[results.length - 1]!
      return results
    } catch (e) {
      stopFakeProgress()
      const msg = e instanceof Error ? e.message : 'Failed to digest files'
      error.value = msg
      throw new Error(msg)
    } finally {
      isDigesting.value = false
    }
  }

  function reset() {
    if (progressTimer) {
      clearInterval(progressTimer)
      progressTimer = null
    }
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
    digestFiles,
    reset,
  }
}
