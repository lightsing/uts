import { decodeAbiParameters, type Hex, type PublicClient } from 'viem'
import { ieasAbi } from '@uts/contracts'

export const NO_EXPIRATION = 0n

export interface OnChainAttestation {
  uid: Hex
  schema: Hex
  time: bigint
  expirationTime: bigint
  revocationTime: bigint
  refUID: Hex
  recipient: Hex
  attester: Hex
  revocable: boolean
  data: Hex
}

export async function readEASTimestamp(
  client: PublicClient,
  easAddress: Hex,
  data: Hex,
): Promise<bigint> {
  return client.readContract({
    address: easAddress,
    abi: ieasAbi,
    functionName: 'getTimestamp',
    args: [data],
  })
}

export async function readEASAttestation(
  client: PublicClient,
  easAddress: Hex,
  uid: Hex,
): Promise<OnChainAttestation> {
  const result = await client.readContract({
    address: easAddress,
    abi: ieasAbi,
    functionName: 'getAttestation',
    args: [uid],
  })
  return result as unknown as OnChainAttestation
}

export function decodeContentHash(data: Hex): Hex {
  const [contentHash] = decodeAbiParameters(
    [{ name: 'contentHash', type: 'bytes32' }],
    data,
  )
  return contentHash
}
