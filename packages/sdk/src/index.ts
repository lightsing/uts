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
