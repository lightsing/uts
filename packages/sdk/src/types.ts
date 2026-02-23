import { type BytesLike } from 'ethers/utils'

export const DIGEST_OPS = ['SHA1', 'SHA256', 'RIPEMD160', 'KECCAK256'] as const

export type DigestOp = (typeof DIGEST_OPS)[number]
export type Op =
  | DigestOp
  | 'APPEND'
  | 'PREPEND'
  | 'REVERSE'
  | 'HEXLIFY'
  | 'ATTESTATION'
  | 'FORK'

export interface DigestHeader {
  kind: DigestOp
  digest: BytesLike
}

export interface StepLike {
  op: Op
}

export interface BaseExecutionStep extends StepLike {
  op: Exclude<Op, 'ATTESTATION' | 'FORK'>
  input?: BytesLike
  output?: BytesLike
}

export interface DataExecutionStep extends BaseExecutionStep {
  op: 'APPEND' | 'PREPEND'
  data: BytesLike
}

export interface UnaryExecutionStep extends BaseExecutionStep {
  op: DigestOp | 'REVERSE' | 'HEXLIFY'
}

export type AttestationKind = 'pending' | 'bitcoin' | 'ethereum-uts' | 'unknown'

export type PendingAttestation = { kind: 'pending'; url: URL }
export type BitcoinAttestation = { kind: 'bitcoin'; height: number }
export type EthereumUTSAttestation = {
  kind: 'ethereum-uts'
  chain: number
  height: number
  metadata?: EthereumUTSAttestationExtraMetadata
}
export type UnknownAttestation = {
  kind: 'unknown'
  tag: BytesLike
  data: BytesLike
}

export type Attestation =
  | PendingAttestation
  | BitcoinAttestation
  | EthereumUTSAttestation
  | UnknownAttestation

export type AttestationStep = { op: 'ATTESTATION'; attestation: Attestation }

export type ForkStep = { op: 'FORK'; steps: Timestamp[] }

export interface EthereumUTSAttestationExtraMetadata {
  contract?: BytesLike
  txHash?: BytesLike
}

export type ExecutionStep = DataExecutionStep | UnaryExecutionStep

export type Step = ExecutionStep | AttestationStep | ForkStep

export type Timestamp = Step[]

export interface DetachedTimestamp {
  header: DigestHeader
  timestamp: Timestamp
}
