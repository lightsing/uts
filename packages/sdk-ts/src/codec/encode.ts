import {
  OP_CODE_MAP,
  MAGIC_BYTES,
  PENDING_ATTESTATION_TAG,
  BITCOIN_ATTESTATION_TAG,
  EAS_ATTEST_TAG,
  EAS_TIMESTAMPED_TAG,
  MAX_URI_LEN,
  SAFE_URL_REGEX,
  DIGEST_LENGTHS,
} from './constants.ts'
import type {
  DetachedTimestamp,
  Step,
  ForkStep,
  AttestationStep,
  ExecutionStep,
  DigestHeader,
  Timestamp,
  Op,
  PendingAttestation,
  BitcoinAttestation,
  EASAttestation,
  EASTimestamped,
} from '../types.ts'
import { getBytes } from '../utils.ts'
import { EncodeError, ErrorCode } from '../errors.ts'

export default class Encoder {
  private buffer: Uint8Array
  private offset: number = 0

  private static textEncoder = new TextEncoder()

  constructor(initialSize: number = 1024) {
    this.buffer = new Uint8Array(initialSize)
  }

  toUint8Array(): Uint8Array {
    return this.buffer.slice(0, this.offset)
  }

  private ensureCapacity(required: number) {
    if (this.offset + required > this.buffer.length) {
      const newLen = Math.max(this.buffer.length * 2, this.offset + required)
      const newBuffer = new Uint8Array(newLen)
      newBuffer.set(this.buffer)
      this.buffer = newBuffer
    }
  }

  writeByte(byte: number): this {
    this.ensureCapacity(1)
    this.buffer[this.offset++] = byte
    return this
  }

  writeBytes(data: Uint8Array): this {
    this.ensureCapacity(data.length)
    this.buffer.set(data, this.offset)
    this.offset += data.length
    return this
  }

  writeU32(value: number): this {
    if (value < 0 || !Number.isInteger(value)) {
      throw new EncodeError(
        ErrorCode.NEGATIVE_LEB128_INPUT,
        `LEB128 only supports non-negative integers, got ${value}`,
        { offset: this.offset },
      )
    }
    if (value > 0xffffffff) {
      throw new EncodeError(
        ErrorCode.OVERFLOW,
        `Value exceeds maximum for u32: ${value}, use writeBigUint instead`,
        { offset: this.offset },
      )
    }

    let n = value
    do {
      // Get bottom 7 bits
      let byte = n & 0x7f
      n >>>= 7 // Unsigned right shift

      // If there are more bits to come, set the continuation bit (0x80)
      if (n !== 0) {
        byte |= 0x80
      }

      this.writeByte(byte)
    } while (n !== 0)

    return this
  }

  writeBigUint(value: bigint): this {
    if (value < 0n) {
      throw new EncodeError(
        ErrorCode.NEGATIVE_LEB128_INPUT,
        `LEB128 only supports non-negative integers, got ${value}`,
        { offset: this.offset },
      )
    }

    let n = value
    do {
      // Get bottom 7 bits
      let byte = Number(n & 0x7fn)
      n >>= 7n // Right shift by 7 bits

      // If there are more bits to come, set the continuation bit (0x80)
      if (n !== 0n) {
        byte |= 0x80
      }

      this.writeByte(byte)
    } while (n !== 0n)

    return this
  }

  writeLengthPrefixedBytes(data: Uint8Array): this {
    const len = data.length
    this.writeU32(len)
    this.writeBytes(data)
    return this
  }

  writeOp(op: Op): this {
    const opCode = OP_CODE_MAP[op]
    if (opCode === undefined) {
      throw new EncodeError(ErrorCode.UNKNOWN_OP, `Unknown operation: ${op}`, {
        offset: this.offset,
        context: { op },
      })
    }
    return this.writeByte(opCode)
  }

  writeVersionedMagic(version: number): this {
    this.writeBytes(MAGIC_BYTES)
    this.writeByte(version)
    return this
  }

  writeHeader(header: DigestHeader): this {
    this.writeOp(header.kind)
    const digestBytes = getBytes(header.digest)
    if (digestBytes.length !== DIGEST_LENGTHS[header.kind]) {
      throw new EncodeError(
        ErrorCode.LENGTH_MISMATCH,
        `Digest length mismatch for ${header.kind}: expected ${DIGEST_LENGTHS[header.kind]}, got ${digestBytes.length}`,
        { offset: this.offset, context: { header } },
      )
    }
    this.writeBytes(digestBytes)
    return this
  }

  writeExecutionStep(step: ExecutionStep): this {
    this.writeOp(step.op)
    switch (step.op) {
      case 'APPEND':
      case 'PREPEND':
        this.writeLengthPrefixedBytes(getBytes(step.data))
        break
    }
    return this
  }

  writeForkStep(step: ForkStep): this {
    if (step.steps.length < 2) {
      throw new EncodeError(
        ErrorCode.INVALID_STRUCTURE,
        'FORK step must have at least 2 branches',
        { offset: this.offset, context: { step } },
      )
    }
    for (const branch of step.steps.slice(0, step.steps.length - 1)) {
      this.writeOp(step.op)
      this.writeTimestamp(branch)
    }
    this.writeTimestamp(step.steps.at(-1)!)
    return this
  }

  writePendingAttestation(attestation: PendingAttestation): this {
    let urlStr = attestation.url.toString()
    // trim url ends with slash
    if (urlStr.endsWith('/')) {
      urlStr = urlStr.slice(0, -1)
    }

    if (urlStr.length > MAX_URI_LEN) {
      throw new EncodeError(
        ErrorCode.INVALID_URI,
        `URL in pending attestation exceeds maximum length of ${MAX_URI_LEN}: ${urlStr}`,
        { offset: this.offset, context: { url: urlStr } },
      )
    }
    if (!SAFE_URL_REGEX.test(urlStr)) {
      throw new EncodeError(
        ErrorCode.INVALID_URI,
        `Invalid URL in pending attestation: ${urlStr}`,
        { offset: this.offset, context: { url: urlStr } },
      )
    }
    // Encode URL as UTF-8 bytes
    const urlBytes = Encoder.textEncoder.encode(urlStr)
    this.writeLengthPrefixedBytes(urlBytes)
    return this
  }

  writeBitcoinAttestation(attestation: BitcoinAttestation): this {
    this.writeU32(attestation.height)
    return this
  }

  writeEAS(attestation: EASAttestation | EASTimestamped): this {
    this.writeU32(attestation.chain)
    if ('uid' in attestation) {
      this.writeBytes(getBytes(attestation.uid))
    }
    return this
  }

  writeAttestationStep(step: AttestationStep): this {
    this.writeOp('ATTESTATION')
    const encoder = new Encoder()
    switch (step.attestation.kind) {
      case 'pending':
        this.writeBytes(PENDING_ATTESTATION_TAG)
        encoder.writePendingAttestation(step.attestation)
        this.writeLengthPrefixedBytes(encoder.toUint8Array())
        break
      case 'bitcoin':
        this.writeBytes(BITCOIN_ATTESTATION_TAG)
        encoder.writeBitcoinAttestation(step.attestation)
        this.writeLengthPrefixedBytes(encoder.toUint8Array())
        break
      case 'eas-attestation':
        this.writeBytes(EAS_ATTEST_TAG)
        encoder.writeEAS(step.attestation)
        this.writeLengthPrefixedBytes(encoder.toUint8Array())
        break
      case 'eas-timestamped':
        this.writeBytes(EAS_TIMESTAMPED_TAG)
        encoder.writeEAS(step.attestation)
        this.writeLengthPrefixedBytes(encoder.toUint8Array())
        break
      case 'unknown':
        this.writeBytes(getBytes(step.attestation.tag))
        this.writeLengthPrefixedBytes(getBytes(step.attestation.data))
        break
      default:
        throw new EncodeError(
          ErrorCode.GENERAL_ERROR,
          `Unsupported attestation: ${step.attestation}`,
          { offset: this.offset, context: { attestation: step.attestation } },
        )
    }
    return this
  }

  writeStep(step: Step): this {
    switch (step.op) {
      case 'FORK':
        return this.writeForkStep(step as ForkStep)
      case 'ATTESTATION':
        return this.writeAttestationStep(step as AttestationStep)
      default:
        return this.writeExecutionStep(step as ExecutionStep)
    }
  }

  writeTimestamp(timestamp: Timestamp): this {
    for (const step of timestamp) {
      this.writeStep(step)
    }
    return this
  }

  static encodeDetachedTimestamp(ots: DetachedTimestamp): Uint8Array {
    return new Encoder()
      .writeVersionedMagic(0x01)
      .writeHeader(ots.header)
      .writeTimestamp(ots.timestamp)
      .toUint8Array()
  }
}
