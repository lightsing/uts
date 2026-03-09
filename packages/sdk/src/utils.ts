import { toHex, hexToBytes, type Hex } from 'viem'

export type BytesLike = Hex | Uint8Array

/**
 * Convert a BytesLike value to a Uint8Array.
 * If the input is already a Uint8Array, it is returned as-is.
 * If the input is a hex string, it is decoded.
 */
export function getBytes(value: BytesLike): Uint8Array {
  if (value instanceof Uint8Array) return value
  return hexToBytes(value)
}

/**
 * Convert a BytesLike value to a hex string.
 * If the input is already a hex string, it is returned as-is.
 * If the input is a Uint8Array, it is encoded.
 */
export function hexlify(value: BytesLike): Hex {
  if (typeof value === 'string') return value as Hex
  return toHex(value)
}
