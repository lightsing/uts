import { hexlify } from 'ethers/utils'
import { DecodeError, ErrorCode } from '../errors.ts'
import {
  DIGEST_OPS,
  type AttestationStep,
  type BitcoinAttestation,
  type DetachedTimestamp,
  type DigestHeader,
  type DigestOp,
  type EthereumUTSAttestation,
  type ExecutionStep,
  type ForkStep,
  type Op,
  type PendingAttestation,
  type Step,
  type Timestamp,
} from '../types.ts'
import {
  ATTESTATION_TAG_LENGTH,
  BITCOIN_ATTESTATION_TAG,
  DIGEST_LENGTHS,
  ETHEREUM_UTS_ATTESTATION_TAG,
  getOpName,
  MAGIC_BYTES,
  MAX_URI_LEN,
  PENDING_ATTESTATION_TAG,
  SAFE_URL_REGEX,
} from './constants.ts'

const MAX_SAFE_INTEGER = BigInt(Number.MAX_SAFE_INTEGER)
export default class Decoder {
  private readonly view: DataView
  private offset: number = 0
  private readonly length: number

  private static textDecoder = new TextDecoder()

  constructor(buffer: Uint8Array) {
    this.view = new DataView(
      buffer.buffer,
      buffer.byteOffset,
      buffer.byteLength,
    )
    this.length = buffer.byteLength
    this.offset = 0
  }

  get remaining(): number {
    return this.length - this.offset
  }

  private checkBounds(required: number): void {
    if (this.offset + required > this.length) {
      throw new DecodeError(
        ErrorCode.UNEXPECTED_EOF,
        `Unexpected end of stream: needed ${required} bytes but only ${this.remaining} available`,
        { offset: this.offset },
      )
    }
  }

  checkEOF(): void {
    if (this.remaining > 0) {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Expected end of stream but ${this.remaining} bytes remain`,
        { offset: this.offset },
      )
    }
  }

  readByte(): number {
    this.checkBounds(1)
    return this.view.getUint8(this.offset++)
  }

  readBytes(length: number): Uint8Array {
    this.checkBounds(length)
    const slice = new Uint8Array(
      this.view.buffer,
      this.view.byteOffset + this.offset,
      length,
    )
    this.offset += length
    return slice
  }

  readNumber(): number {
    const result = this.readBigUint()
    if (result > MAX_SAFE_INTEGER) {
      throw new DecodeError(
        ErrorCode.OVERFLOW,
        `Decoded number exceeds MAX_SAFE_INTEGER: ${result}`,
        { offset: this.offset },
      )
    }
    return Number(result)
  }

  readBigUint(): bigint {
    let result = 0n
    let shift = 0n

    while (true) {
      const byte = this.readByte()
      result |= BigInt(byte & 0x7f) << shift

      if ((byte & 0x80) === 0) break

      shift += 7n
    }

    return result
  }

  readLengthPrefixedBytes(): Uint8Array {
    const len = this.readNumber()
    return this.readBytes(len)
  }

  peekOp(): Op | null {
    if (this.remaining === 0) return null
    const code = this.view.getUint8(this.offset)
    return getOpName(code)
  }

  readOp(): Op {
    const code = this.readByte()
    const op = getOpName(code)
    if (!op) {
      throw new DecodeError(
        ErrorCode.UNKNOWN_OP,
        `Unknown opcode: 0x${code.toString(16).padStart(2, '0')}`,
        { offset: this.offset - 1, context: { code } },
      )
    }
    return op
  }

  readVersionedMagic(): number {
    const magic = this.readBytes(MAGIC_BYTES.length)
    if (!magic.every((val, idx) => val === MAGIC_BYTES[idx])) {
      throw new DecodeError(ErrorCode.BAD_MAGIC, 'Invalid magic bytes', {
        offset: this.offset - MAGIC_BYTES.length,
        context: { found: hexlify(magic) },
      })
    }
    return this.readByte()
  }

  readHeader(): DigestHeader {
    const op = this.readOp()

    if (!(DIGEST_OPS as readonly string[]).includes(op)) {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Expected digest op in header, got: ${op}`,
        { offset: this.offset - 1, context: { op } },
      )
    }

    const kind = op as DigestOp
    const len = DIGEST_LENGTHS[kind]
    const digest = this.readBytes(len)

    return { kind, digest }
  }

  readExecutionStep(): ExecutionStep {
    const op = this.readOp()
    switch (op) {
      case 'APPEND':
      case 'PREPEND':
        const data = this.readLengthPrefixedBytes()
        return { op, data }
      case 'FORK':
      case 'ATTESTATION':
        throw new DecodeError(
          ErrorCode.INVALID_STRUCTURE,
          `Unexpected ${op} step in execution steps, should be handled separately`,
          { offset: this.offset - 1, context: { op } },
        )
      default:
        return { op }
    }
  }

  readForkStep(): ForkStep {
    const steps: Timestamp[] = []
    if (this.peekOp() !== 'FORK') {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Expected FORK op at the beginning of fork step, got: ${this.peekOp()}`,
        { offset: this.offset, context: { op: this.peekOp() } },
      )
    }

    while (true) {
      const op = this.peekOp()
      if (op === 'FORK') {
        // not the last branch, consume the FORK op and continue
        this.readOp()
        steps.push(this.readTimestamp())
      } else {
        // last branch, read it and break
        steps.push(this.readTimestamp())
        break
      }
    }

    return { op: 'FORK', steps }
  }

  readPendingAttestation(): PendingAttestation {
    const urlBytes = this.readLengthPrefixedBytes()
    const urlStr = Decoder.textDecoder.decode(urlBytes)

    if (urlStr.length > MAX_URI_LEN) {
      throw new DecodeError(
        ErrorCode.INVALID_URI,
        `Attestation URL exceeds maximum length of ${MAX_URI_LEN} characters`,
        { offset: this.offset - urlBytes.length, context: { url: urlStr } },
      )
    }

    if (!SAFE_URL_REGEX.test(urlStr)) {
      throw new DecodeError(
        ErrorCode.INVALID_URI,
        `Invalid URL in pending attestation: ${urlStr}`,
        { offset: this.offset, context: { url: urlStr } },
      )
    }

    try {
      const url = new URL(urlStr)
      return { kind: 'pending', url }
    } catch (error) {
      throw new DecodeError(
        ErrorCode.INVALID_URI,
        `Malformed URL in pending attestation: ${urlStr}`,
        {
          offset: this.offset,
          context: {
            url: urlStr,
            error: error instanceof Error ? error.message : String(error),
          },
        },
      )
    }
  }

  readBitcoinAttestation(): BitcoinAttestation {
    return {
      kind: 'bitcoin',
      height: this.readNumber(),
    }
  }

  readEthereumUTSAttestation(): EthereumUTSAttestation {
    const chain = this.readNumber()
    const height = this.readNumber()

    if (this.remaining === 0) {
      return {
        kind: 'ethereum-uts',
        chain,
        height,
      }
    }

    if (this.remaining > 0 && this.remaining < 20) {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Invalid extra metadata length for Ethereum UTS attestation: expected 0 or at least 20 bytes, got ${this.remaining}`,
        { offset: this.offset, context: { remaining: this.remaining } },
      )
    }
    // Extra metadata is optional, only read if there's remaining data
    const contract = this.readBytes(20)
    if (this.remaining === 0) {
      return {
        kind: 'ethereum-uts',
        chain,
        height,
        metadata: { contract },
      }
    }

    if (this.remaining > 0 && this.remaining < 32) {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Invalid extra metadata length for Ethereum UTS attestation with contract: expected 0 or 32 bytes, got ${this.remaining}`,
        { offset: this.offset, context: { remaining: this.remaining } },
      )
    }

    const txHash = this.readBytes(32)
    return {
      kind: 'ethereum-uts',
      chain,
      height,
      metadata: { contract, txHash },
    }
  }

  readAttestationStep(strict?: boolean): AttestationStep {
    const op = this.readOp()
    if (op !== 'ATTESTATION') {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Expected ATTESTATION op, got: ${op}`,
        { offset: this.offset - 1, context: { op } },
      )
    }

    const tag = this.readBytes(ATTESTATION_TAG_LENGTH)
    const data = this.readLengthPrefixedBytes()
    const decoder = new Decoder(data)
    if (tag.every((byte, idx) => byte === BITCOIN_ATTESTATION_TAG[idx])) {
      const attestation = decoder.readBitcoinAttestation()
      if (strict) decoder.checkEOF()
      return { op: 'ATTESTATION', attestation }
    } else if (
      tag.every((byte, idx) => byte === PENDING_ATTESTATION_TAG[idx])
    ) {
      const attestation = decoder.readPendingAttestation()
      if (strict) decoder.checkEOF()
      return { op: 'ATTESTATION', attestation }
    } else if (
      tag.every((byte, idx) => byte === ETHEREUM_UTS_ATTESTATION_TAG[idx])
    ) {
      const attestation = decoder.readEthereumUTSAttestation()
      if (strict) decoder.checkEOF()
      return { op: 'ATTESTATION', attestation }
    } else {
      return {
        op: 'ATTESTATION',
        attestation: {
          kind: 'unknown',
          tag,
          data,
        },
      }
    }
  }

  readStep(strict?: boolean): Step {
    const op = this.peekOp()
    switch (op) {
      case 'FORK':
        return this.readForkStep()
      case 'ATTESTATION':
        return this.readAttestationStep(strict)
      default:
        return this.readExecutionStep()
    }
  }

  readTimestamp(strict?: boolean): Timestamp {
    const steps: Step[] = []
    while (this.remaining > 0) {
      const step = this.readStep(strict)
      steps.push(step)
      if (step.op === 'FORK' || step.op === 'ATTESTATION') break
    }
    return steps
  }

  readDetachedTimestamp(strict?: boolean): DetachedTimestamp {
    const version = this.readVersionedMagic()
    if (version !== 0x01) {
      throw new DecodeError(
        ErrorCode.INVALID_STRUCTURE,
        `Unsupported detached timestamp version: 0x${version.toString(16)}`,
        { offset: this.offset - 1, context: { version } },
      )
    }
    const header = this.readHeader()
    const timestamp = this.readTimestamp(strict)
    const detached = { header, timestamp }
    if (strict) this.checkEOF()
    return detached
  }
}
