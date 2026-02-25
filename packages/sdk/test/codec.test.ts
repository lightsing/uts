import { describe, it, expect, beforeAll } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
import Encoder from '../src/codec/encode.ts'
import Decoder from '../src/codec/decode.ts'
import type { DetachedTimestamp } from '../src/types.ts'

const __filename = import.meta.filename
const __dirname = import.meta.dirname

const OTS_FILE_PATH = path.join(__dirname, '../fixtures/test.ots')

describe('Codec', () => {
  let fileBuffer: Uint8Array

  beforeAll(() => {
    if (!fs.existsSync(OTS_FILE_PATH)) {
      throw new Error(
        `Test file not found: ${OTS_FILE_PATH}. Please put a valid .ots file there.`,
      )
    }
    fileBuffer = new Uint8Array(fs.readFileSync(OTS_FILE_PATH))
  })

  it('should decode a valid .ots file successfully', () => {
    const decoder = new Decoder(fileBuffer)
    const result: DetachedTimestamp = decoder.readDetachedTimestamp()

    expect(result).toBeDefined()
    expect(result.header.kind).toMatch(
      /^(SHA1|SHA256|SHA512|RIPEMD160|KECCAK256)$/,
    )
    expect(result.header.digest).toBeInstanceOf(Uint8Array)
    expect(result.timestamp).toBeInstanceOf(Array)
    expect(result.timestamp.length).toBeGreaterThan(0)
  })

  it('should match the original data after encoding and decoding', () => {
    const decoder = new Decoder(fileBuffer)
    const original = decoder.readDetachedTimestamp()

    const encodedBuffer = Encoder.encodeDetachedTimestamp(original)

    const reDecoder = new Decoder(encodedBuffer)
    const decoded = reDecoder.readDetachedTimestamp()
    expect(decoded).toEqual(original)

    expect(encodedBuffer).toEqual(fileBuffer)
  })
})
