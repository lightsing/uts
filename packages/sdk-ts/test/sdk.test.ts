import { describe, it, expect, beforeAll } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
import Decoder from '../src/codec/decode.ts'
import type { DetachedTimestamp } from '../src/types.ts'
import SDK from '../src/sdk.ts'
import { hexToBytes } from 'viem'

const __filename = import.meta.filename
const __dirname = import.meta.dirname

const OTS_FILE_PATH = path.join(__dirname, '../fixtures/test.ots')

describe.skip('Stamp', () => {
  it('should stamp', async () => {
    const sdk = new SDK()

    const testDigest = '0x' + 'aa'.repeat(32)
    const results = await sdk.stamp([
      {
        kind: 'SHA256',
        digest: testDigest,
      },
    ])

    expect(results).toHaveLength(1)
    const result = results[0]

    expect(result.header.digest).toEqual(
      hexToBytes(testDigest as `0x${string}`),
    )
    expect(result.timestamp).toBeDefined()
    console.debug('Timestamp:', JSON.stringify(result, null, 2))
  })
})

describe('Verify', () => {
  let fileBuffer: Uint8Array

  beforeAll(() => {
    if (!fs.existsSync(OTS_FILE_PATH)) {
      throw new Error(
        `Test file not found: ${OTS_FILE_PATH}. Please put a valid .ots file there.`,
      )
    }
    fileBuffer = new Uint8Array(fs.readFileSync(OTS_FILE_PATH))
  })

  it('should verify ethereum attestation', async () => {
    const sdk = new SDK()

    const verified = await sdk.verifyAttestation(
      hexToBytes(
        '0x7eb06fdbe20e402a8125775968899b4ab87b9af1c20a81d4af8d5bb0c96d7c64',
      ),
      {
        kind: 'ethereum-uts',
        chain: 54351,
        height: 16862454,
      },
    )

    expect(verified.status).toBe('VALID')
  })

  it('should verify fixture timestamp', async () => {
    const sdk = new SDK()

    const decoder = new Decoder(fileBuffer)
    const detachedTimestamp: DetachedTimestamp = decoder.readDetachedTimestamp()

    const verified = await sdk.verify(detachedTimestamp)
    expect(verified).toBeDefined()
    console.debug('Verification details:', JSON.stringify(verified, null, 2))

    const result = sdk.transformResult(verified)
    expect(result).toBeDefined()
    console.debug('Verification result:', JSON.stringify(result, null, 2))
  })
})
