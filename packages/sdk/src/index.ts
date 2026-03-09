import { toHex } from 'viem'

export type * from './types.ts'

export type { BytesLike } from './utils.ts'

export type {
  Attestation,
  PendingAttestation,
  BitcoinAttestation,
  EASAttestation,
  EASTimestamped,
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

export { default as SDK, DEFAULT_CALENDARS, WELL_KNOWN_CHAINS } from './sdk.ts'
export type { SDKOptions, StampEvent, StampEventCallback } from './sdk.ts'

export const hexlify = (obj: any): any => {
  if (obj instanceof URL) {
    return obj
  }
  if (obj instanceof Uint8Array) {
    return toHex(obj)
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
