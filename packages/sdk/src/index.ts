import { hexlify as h } from 'ethers'

export type * from './types.ts'

export type {
  Attestation,
  PendingAttestation,
  BitcoinAttestation,
  EthereumUTSAttestation,
  EthereumUTSAttestationExtraMetadata,
} from './types.ts'

export {
  DIGEST_OPS,
  UpgradeStatus,
  AttestationStatusKind,
  VerifyStatus,
} from './types.ts'

export { default as Encoder } from './codec/encode.ts'
export { default as Decoder } from './codec/decode.ts'

export * from './errors.ts'

export * from './codec/constants.ts'

export * from './bmt.ts'

export { default as BitcoinRPC } from './rpc/btc.ts'

export { default as SDK, DEFAULT_CALENDARS, UTS_ABI } from './sdk.ts'
export type { SDKOptions, StampEvent, StampEventCallback } from './sdk.ts'

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
      if (Object.hasOwn(obj, key)) {
        result[key] = hexlify(obj[key])
      }
    }
    return result
  }
  return obj
}
