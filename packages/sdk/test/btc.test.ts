import { describe, it, expect } from 'vitest'
import BitcoinRPC from '../src/rpc/btc.ts'

describe('btc', () => {
  it('get block', async () => {
    const rpc = new BitcoinRPC()
    const blockHash = await rpc.getBlockHash(938073)
    const blockHeader = await rpc.getBlockHeader(blockHash)
    console.debug('Block hash:', blockHash)
    console.debug('Block header:', JSON.stringify(blockHeader, null, 2))

    expect(blockHash).toBe(
      '000000000000000000014a0305931a1a12218059a0dd8a79633c27b3e9153172',
    )
    expect(blockHeader.hash).toBe(
      '000000000000000000014a0305931a1a12218059a0dd8a79633c27b3e9153172',
    )
    expect(blockHeader.height).toBe(938073)
    expect(blockHeader.merkleroot).toBe(
      '015b0dc9e6c4d514f09f0b6bc67ec0382e4cacdfc18f7e069c74e14145447df3',
    )
  })
})
