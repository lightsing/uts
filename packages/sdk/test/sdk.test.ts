import { describe, it, expect } from 'vitest'
import { getBytes } from 'ethers'
import SDK from '../src/sdk'

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

    expect(result.header.digest).toEqual(getBytes(testDigest))
    expect(result.timestamp).toBeDefined()
    console.debug('Timestamp:', JSON.stringify(result, null, 2))
  })
})

describe('Verify', () => {
  it('should verify ethereum attestation', async () => {
    const sdk = new SDK()

    const verified = await sdk.verifyAttestation(
      getBytes(
        '0x7eb06fdbe20e402a8125775968899b4ab87b9af1c20a81d4af8d5bb0c96d7c64',
      ),
      {
        kind: 'ethereum-uts',
        chain: 54351,
        height: 16862454,
      },
    )
    console.debug('Verified:', verified)
  })
})
