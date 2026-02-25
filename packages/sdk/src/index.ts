import { hexlify as h } from 'ethers'

export type * from './types'

export type {
  Attestation,
  PendingAttestation,
  BitcoinAttestation,
  EthereumUTSAttestation,
  EthereumUTSAttestationExtraMetadata,
} from './types'

export { default as Encoder } from './codec/encode'
export { default as Decoder } from './codec/decode'

export * from './errors'

export * from './codec/constants'

export * from './bmt'

export { default as BitcoinRPC } from './rpc/btc'

export const hexlify = (obj: any): any => {
  if (obj instanceof URL) {
    return obj
  }
  if (obj instanceof Uint8Array) {
    return h(obj)
  }
  if (Array.isArray(obj)) {
    return obj.map((item) => hexlify(item))
  }
  if (typeof obj === 'object' && obj !== null) {
    const result: any = {}
    for (const key in obj) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        result[key] = hexlify(obj[key])
      }
    }
    return result
  }
  return obj
}
