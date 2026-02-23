import { getBytes, hexlify, type BytesLike } from 'ethers'
import type { DetachedTimestamp, DigestOp, ForkStep, Timestamp } from './types'
import type { CHash } from '@noble/hashes/utils.js'
import { sha256 } from '@noble/hashes/sha2.js'
import { keccak_256 } from '@noble/hashes/sha3.js'
import { INNER_NODE_PREFIX, NodePosition, UnorderedMerkleTree } from './bmt'
import Decoder from './codec/decode'

export default class SDK {
  public readonly calendars: URL[]

  public timeout: number = 10000

  /**
   * Consider the timestamp complete if at least M calendars reply prior to the timeout
   */
  public quorum: number

  private hashAlg: DigestOp = 'KECCAK256'
  private hasher: CHash = keccak_256

  public nonceSize: number = 32

  constructor(calendars: URL[]) {
    this.calendars = calendars
    this.quorum = Math.ceil(calendars.length * 0.66)
  }

  public setHashAlgorithm(alg: DigestOp) {
    this.hashAlg = alg
    switch (alg) {
      case 'SHA256':
        this.hasher = sha256
        break
      case 'KECCAK256':
        this.hasher = keccak_256
        break
      default:
        throw new Error(`Unsupported hash algorithm: ${alg}`)
    }
  }

  /**
   * Stamp the provided digests by submitting them to the configured calendars.
   *
   * @param digests The digests to be stamped.
   * @param timeout The maximum time to wait for calendar responses.
   */
  async stamp(
    digests: BytesLike[],
    timeout: number = 10000,
  ): Promise<DetachedTimestamp[]> {
    let nonceDigests: Uint8Array[] = []

    for (const digest of digests) {
      const hasher = this.hasher.create()
      hasher.update(getBytes(digest))
      const nonce = crypto.getRandomValues(new Uint8Array(this.nonceSize))
      hasher.update(nonce)
      const nonceDigest = new Uint8Array(hasher.digest())
      nonceDigests.push(nonceDigest)
    }

    const internalMerkleTree = UnorderedMerkleTree.new(
      nonceDigests,
      this.hasher,
    )
    const root = internalMerkleTree.root()
    console.debug(`Internal Merkle root: ${hexlify(root)}`)

    const calendarResponses = await Promise.allSettled(
      this.calendars.map((calendar) =>
        requestCalendar(calendar, timeout, root),
      ),
    )

    const successfulResponses = calendarResponses.filter(
      (res) => res.status === 'fulfilled',
    ) as PromiseFulfilledResult<Timestamp>[]
    if (successfulResponses.length < this.quorum) {
      throw new Error(
        `Only received ${successfulResponses.length} valid responses from calendars, which does not meet the quorum of ${this.quorum}`,
      )
    }

    const mergedTimestamp =
      successfulResponses.length === 1
        ? successfulResponses[0].value
        : [
            {
              op: 'FORK',
              steps: successfulResponses.map((res) => res.value),
            } as ForkStep,
          ]

    return digests.map((digest, i) => {
      let timestamp: Timestamp = [
        { op: 'APPEND', data: nonceDigests[i] },
        { op: this.hashAlg },
      ]

      const proofIter = internalMerkleTree.getProofIter(nonceDigests[i])
      if (proofIter === null) {
        throw new Error(
          `Failed to generate proof for digest ${hexlify(digest)}`,
        )
      }

      for (const step of proofIter) {
        if (step.position === NodePosition.Left) {
          timestamp.push({
            op: 'PREPEND',
            data: new Uint8Array([INNER_NODE_PREFIX]),
          })
          timestamp.push({ op: 'APPEND', data: step.sibling })
          timestamp.push({ op: this.hashAlg })
        } else {
          timestamp.push({ op: 'PREPEND', data: step.sibling })
          timestamp.push({
            op: 'PREPEND',
            data: new Uint8Array([INNER_NODE_PREFIX]),
          })
          timestamp.push({ op: this.hashAlg })
        }
      }
      timestamp.push(...mergedTimestamp)

      return {
        header: { kind: this.hashAlg, digest: getBytes(digests[i]) },
        timestamp,
      }
    })
  }
}

const requestCalendar = async (
  calendar: URL,
  timeout: number,
  root: Uint8Array,
): Promise<Timestamp> => {
  console.debug(`Submitting to remote calendar: ${calendar}`)
  const url = new URL('/digest', calendar)
  try {
    const response = await fetch(url.toString(), {
      body: root as BodyInit,
      method: 'POST',
      headers: { Accept: 'application/vnd.opentimestamps.v1' },
      signal: AbortSignal.timeout(timeout),
    })
    if (!response.ok) {
      throw new Error(
        `Calendar ${calendar} responded with status ${response.status}`,
      )
    }
    const responseData = await response.arrayBuffer()
    const decoder = new Decoder(new Uint8Array(responseData))
    return decoder.readTimestamp()
  } catch (e) {
    console.error(`Failed to submit to calendar ${calendar}:`, e)
    throw e
  }
}
