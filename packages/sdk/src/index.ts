export type HexString = string

export type DigestOp = 'SHA1' | 'SHA512' | 'RIPEMD160' | 'KECCAK256'

export interface DigestHeader {
  kind: DigestOp
  digest: HexString
}

export interface BaseExecutionStep {
  input: HexString
  output: HexString
}

export interface DataExecutionStep extends BaseExecutionStep {
  op: 'APPEND' | 'PREPEND'
  data: HexString
}

export interface UnaryExecutionStep extends BaseExecutionStep {
  op: DigestOp | 'REVERSE' | 'HEXLIFY'
}

export type ExecutionStep = DataExecutionStep | UnaryExecutionStep

export type AttestationStep =
  | { kind: 'pending'; url: string }
  | { kind: 'bitcoin'; height: number }
  | { kind: 'unknown'; tag: HexString; data: HexString }

export type TraceNode = ExecutionStep | AttestationStep | TraceNode[]

export type TraceResult = TraceNode[]
