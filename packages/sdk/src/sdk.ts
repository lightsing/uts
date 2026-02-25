import {
  type AbstractProvider,
  getBytes,
  hexlify,
  id,
  Interface,
  JsonRpcProvider,
} from 'ethers'
import {
  AttestationStatusKind,
  UpgradeStatus,
  VerifyStatus,
  type Attestation,
  type AttestationStatus,
  type BitcoinAttestation,
  type DetachedTimestamp,
  type DigestHeader,
  type EthereumUTSAttestation,
  type ExecutionStep,
  type ForkStep,
  type PendingAttestation,
  type SecureDigestOp,
  type Timestamp,
  type UpgradeResult,
} from './types'
import type { CHash } from '@noble/hashes/utils.js'
import { sha256 } from '@noble/hashes/sha2.js'
import { keccak_256 } from '@noble/hashes/sha3.js'
import { INNER_NODE_PREFIX, NodePosition, UnorderedMerkleTree } from './bmt'
import Decoder from './codec/decode'
import { EncodeError, ErrorCode, RemoteError, VerifyError } from './errors'
import { ripemd160, sha1 } from '@noble/hashes/legacy.js'
import BitcoinRPC from './rpc/btc'
import { FallbackProvider } from 'ethers'

export interface SDKOptions {
  calendars?: URL[]
  btcRPC?: BitcoinRPC
  ethRPCs?: Record<number, AbstractProvider>
  timeout?: number
  quorum?: number
  nonceSize?: number
  hashAlgorithm?: SecureDigestOp
}

export const DEFAULT_CALENDARS = [
  new URL('https://a.pool.opentimestamps.org/'),
  new URL('https://b.pool.opentimestamps.org/'),
  new URL('https://a.pool.eternitywall.com/'),
  new URL('https://ots.btc.catallaxy.com/'),
]

export const UTS_ABI = [
  'event Attested(bytes32 indexed root, address indexed sender, uint256 timestamp)',
  'function attest(bytes32 root) external',
  'function timestamp(bytes32 root) external view returns (uint256)',
]

export default class SDK {
  readonly calendars: URL[]
  readonly btcRPC: BitcoinRPC
  readonly ethRPCs: Record<number, AbstractProvider>

  /**
   * Maximum time to wait for calendar responses in milliseconds.
   *
   * Calendars that do not respond within this time will be ignored,
   * and the timestamp will be generated from the successful responses received prior to the timeout.
   */
  timeout: number = 10000

  /**
   * Consider the timestamp complete if at least M calendars reply prior to the timeout
   */
  quorum: number

  /**
   * Number of random bytes to append to each digest before stamping, which are required to generate the internal Merkle proof.
   *
   * This is needed to prevent leaking information about the original digest to the calendar servers,
   * which could be used to censor or preimage attack the digest.
   *
   * The nonce is included in the timestamp and can be safely revealed without compromising the security of the original digest.
   */
  nonceSize: number

  private hashAlg: SecureDigestOp = 'KECCAK256'
  private hasher: CHash = keccak_256

  private static encoder = new TextEncoder()

  // 0x61cae4201bb8c0117495b22a70f5202410666b349c27302dac280dc054b60f2a
  static readonly utsLogTopic = id('Attested(bytes32,address,uint256)')
  static readonly utsInterface = new Interface(UTS_ABI)

  constructor(options: SDKOptions = {}) {
    const {
      calendars = DEFAULT_CALENDARS,
      btcRPC = new BitcoinRPC(),
      ethRPCs = {
        1: new FallbackProvider([
          new JsonRpcProvider('https://eth.drpc.org'),
          new JsonRpcProvider('https://eth.llamarpc.com'),
          new JsonRpcProvider('https://eth.llamarpc.com'),
        ]),
        17000: new FallbackProvider([
          new JsonRpcProvider('https://holesky.drpc.org'),
          new JsonRpcProvider('https://1rpc.io/holesky'),
        ]),
        11155111: new FallbackProvider([
          new JsonRpcProvider('https://sepolia.drpc.org'),
          new JsonRpcProvider('https://0xrpc.io/sep'),
          new JsonRpcProvider('https://rpc.sepolia.org'),
        ]),
        54352: new JsonRpcProvider('https://rpc.scroll.io'),
        54351: new JsonRpcProvider('https://sepolia-rpc.scroll.io'),
      },
      timeout = 10000,
      nonceSize = 32,
      hashAlgorithm = 'KECCAK256',
      quorum,
    } = options

    this.calendars = calendars
    this.btcRPC = btcRPC
    this.ethRPCs = ethRPCs

    this.timeout = timeout
    this.nonceSize = nonceSize

    this.quorum = quorum ?? Math.ceil(this.calendars.length * 0.66)
    this.hashAlgorithm = hashAlgorithm
  }

  get hashAlgorithm(): SecureDigestOp {
    return this.hashAlg
  }

  /**
   * Set the hash algorithm to be used during stamping.
   *
   * This will affect the internal Merkle tree construction and the proof generation.
   *
   * Supported algorithms are 'SHA256' and 'KECCAK256'.
   * @param alg
   */
  set hashAlgorithm(alg: SecureDigestOp) {
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

  getEthProvider(chainId: number): AbstractProvider | null {
    if (Object.hasOwn(this.ethRPCs, chainId)) {
      return this.ethRPCs[chainId]!
    }
    return null
  }

  /**
   * Stamp the provided digests by submitting them to the configured calendars.
   *
   * @param digests The digests to be stamped, each with its associated header information. Input digests can use different hash algorithms, but the internal Merkle tree will be constructed using the SDK's configured hash algorithm (default KECCAK256).
   */
  async stamp(digests: DigestHeader[]): Promise<DetachedTimestamp[]> {
    const nonces: Uint8Array[] = []
    const nonceDigests: Uint8Array[] = []

    for (const digest of digests) {
      const hasher = this.hasher.create()
      hasher.update(getBytes(digest.digest))
      const nonce = crypto.getRandomValues(new Uint8Array(this.nonceSize))
      hasher.update(nonce)
      const nonceDigest = new Uint8Array(hasher.digest())
      nonces.push(nonce)
      nonceDigests.push(nonceDigest)
    }

    const internalMerkleTree = UnorderedMerkleTree.new(
      nonceDigests,
      this.hasher,
    )
    const root = internalMerkleTree.root()
    console.debug(`Internal Merkle root: ${hexlify(root)}`)

    const calendarResponses = await Promise.allSettled(
      this.calendars.map((calendar) => this.requestAttest(calendar, root)),
    )

    const successfulResponses = calendarResponses.filter(
      (res) => res.status === 'fulfilled',
    ) as Array<PromiseFulfilledResult<Timestamp>>
    if (successfulResponses.length < this.quorum) {
      throw new RemoteError(
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
      const timestamp: Timestamp = [
        { op: 'APPEND', data: nonces[i] },
        { op: this.hashAlg },
      ]

      const proofIter = internalMerkleTree.getProofIter(nonceDigests[i])
      if (proofIter === null) {
        throw new EncodeError(
          ErrorCode.INVALID_STRUCTURE,
          `Failed to generate proof for digest ${hexlify(digest.digest)}`,
        )
      }

      for (const step of proofIter) {
        if (step.position === NodePosition.Left) {
          timestamp.push(
            {
              op: 'PREPEND',
              data: new Uint8Array([INNER_NODE_PREFIX]),
            },
            { op: 'APPEND', data: step.sibling },
            { op: this.hashAlg },
          )
        } else {
          timestamp.push(
            { op: 'PREPEND', data: step.sibling },
            {
              op: 'PREPEND',
              data: new Uint8Array([INNER_NODE_PREFIX]),
            },
            { op: this.hashAlg },
          )
        }
      }
      timestamp.push(...mergedTimestamp)

      return {
        header: digest,
        timestamp,
      }
    })
  }

  /**
   * Submit the root digest to the calendar and receive the timestamp steps in response.
   *
   * @param calendar The URL of the calendar to submit the root digest to.
   * @param root The root digest to be submitted to the calendar.
   * @returns The timestamp steps received from the calendar.
   */
  async requestAttest(calendar: URL, root: Uint8Array): Promise<Timestamp> {
    console.debug(`Submitting to remote calendar: ${calendar}`)
    const url = new URL('/digest', calendar)

    let response: Response
    try {
      response = await fetch(url.toString(), {
        body: root as BodyInit,
        method: 'POST',
        headers: { Accept: 'application/vnd.opentimestamps.v1' },
        signal: AbortSignal.timeout(this.timeout),
      })
    } catch (error) {
      throw new RemoteError(`Failed to submit to calendar ${calendar}`, {
        context: { source: error },
      })
    }

    if (!response.ok) {
      throw new RemoteError(
        `Calendar ${calendar} responded with status ${response.status}`,
        {
          context: { status: response.status },
        },
      )
    }
    const responseData = await response.arrayBuffer()
    const decoder = new Decoder(new Uint8Array(responseData))
    return decoder.readTimestamp()
  }

  /**
   * Perform in-place upgrade of the provided detached timestamp by replacing any pending attestations with their upgraded timestamp steps, if they have become available.
   * @param detached The detached timestamp to be upgraded.
   * @returns The result of the upgrade operation, including the original and upgraded timestamps if applicable.
   */
  async upgrade(detached: DetachedTimestamp): Promise<UpgradeResult[]> {
    return this.upgradeTimestamp(
      getBytes(detached.header.digest),
      detached.timestamp,
    )
  }

  /**
   * Upgrade the provided timestamp steps by replacing any pending attestations with their upgraded timestamp steps, if they have become available.
   * This function will recursively traverse the timestamp steps and perform in-place upgrades of any pending attestations encountered.
   * @param input The original digest input associated with the timestamp, which is needed to verify and upgrade the pending attestations.
   * @param timestamp The timestamp steps to be upgraded, which may contain pending attestations that need to be replaced with their upgraded timestamp steps if they have become available.
   * @returns The result of the upgrade operation, including the original and upgraded timestamps if applicable.
   */
  async upgradeTimestamp(
    input: Uint8Array,
    timestamp: Timestamp,
  ): Promise<UpgradeResult[]> {
    let current = input

    const result: UpgradeResult[] = []

    for (let i = 0; i < timestamp.length; i++) {
      const step = timestamp[i]
      switch (step.op) {
        case 'APPEND':
        case 'PREPEND':
        case 'REVERSE':
        case 'HEXLIFY':
        case 'SHA1':
        case 'RIPEMD160':
        case 'SHA256':
        case 'KECCAK256':
          current = this.executeStep(current, step)
          break

        case 'FORK':
          // upgrade sub stamps
          const results = (
            await Promise.all(
              step.steps.map((branch) => this.upgradeTimestamp(input, branch)),
            )
          ).flat()
          result.push(...results)
          break
        case 'ATTESTATION':
          if (step.attestation.kind !== 'pending') {
            continue
          }
          try {
            const upgraded = await this.upgradeAttestation(
              current,
              step.attestation,
            )
            if (upgraded === null) {
              result.push({
                status: UpgradeStatus.Pending,
                original: step.attestation,
              })
              continue
            }
            // preserve the original attestation in the upgraded timestamp for transparency
            timestamp[i] = {
              op: 'FORK',
              steps: [[step], upgraded],
            }
            result.push({
              status: UpgradeStatus.Upgraded,
              original: step.attestation,
              upgraded,
            })
          } catch (error) {
            console.error(`Error upgrading attestation: ${error}`)
            result.push({
              status: UpgradeStatus.Failed,
              original: step.attestation,
              error: error instanceof Error ? error : new Error(String(error)),
            })
          }
          break
      }
    }
    return result
  }

  /**
   * Upgrade the provided pending attestation by fetching its upgraded timestamp steps from the remote calendar.
   * @param commitment The original digest input associated with the attestation, which is needed to verify and upgrade the pending attestation.
   * @param attestation The pending attestation to be upgraded.
   * @returns The upgraded timestamp steps if available, or null if the attestation is still pending.
   */
  async upgradeAttestation(
    commitment: Uint8Array,
    attestation: PendingAttestation,
  ): Promise<Timestamp | null> {
    const url = new URL(`timestamp/${hexlify(commitment)}`, attestation.url)

    let response: Response
    try {
      response = await fetch(url.toString(), {
        method: 'GET',
        headers: { Accept: 'application/vnd.opentimestamps.v1' },
        signal: AbortSignal.timeout(this.timeout),
      })
    } catch (error) {
      throw new RemoteError(
        `Failed to fetch from calendar ${attestation.url}`,
        {
          context: { source: error },
        },
      )
    }

    if (response.status === 404) {
      console.debug(`Attestation at ${attestation.url} is still pending`)
      return null
    }

    if (!response.ok) {
      throw new RemoteError(
        `Calendar ${attestation.url} responded with status ${response.status}`,
        {
          context: { status: response.status },
        },
      )
    }
    const responseData = await response.arrayBuffer()
    const decoder = new Decoder(new Uint8Array(responseData))
    return decoder.readTimestamp()
  }

  /**
   * Verify the provided detached timestamp by replaying the timestamp steps and validating the attestations.
   *
   * @param stamp Detached timestamp to verify, which includes the original digest header and the associated timestamp steps.
   * @returns An array of attestation statuses resulting from the verification process, which can be used to determine the overall validity of the timestamp.
   */
  async verify(stamp: DetachedTimestamp): Promise<AttestationStatus[]> {
    const input = getBytes(stamp.header.digest)

    return this.verifyTimestamp(input, stamp.timestamp)
  }

  /**
   * Verify the provided timestamp steps against the input digest by replaying the operations and validating any encountered attestations.
   *
   * @param input The original digest input to be verified against the timestamp steps. This should match the digest in the timestamp header.
   * @param timestamp The timestamp steps to verify against the input digest.
   * @returns An array of attestation statuses resulting from the verification process.
   */
  async verifyTimestamp(
    input: Uint8Array,
    timestamp: Timestamp,
  ): Promise<AttestationStatus[]> {
    const attestations: AttestationStatus[] = []

    let current = input
    for (const step of timestamp) {
      switch (step.op) {
        case 'APPEND':
        case 'PREPEND':
        case 'REVERSE':
        case 'HEXLIFY':
        case 'SHA1':
        case 'RIPEMD160':
        case 'SHA256':
        case 'KECCAK256':
          current = this.executeStep(current, step)
          break

        case 'FORK':
          // verify sub stamps
          for (const branch of step.steps) {
            const result = await this.verifyTimestamp(current, branch)
            attestations.push(...result)
          }
          break
        case 'ATTESTATION':
          const status = await this.verifyAttestation(current, step.attestation)
          attestations.push(status)
          break
        default:
          throw new VerifyError(
            ErrorCode.INVALID_STRUCTURE,
            `Unsupported step ${step} in timestamp`,
          )
      }
    }

    return attestations
  }

  async verifyAttestation(
    input: Uint8Array,
    attestation: Attestation,
  ): Promise<AttestationStatus> {
    switch (attestation.kind) {
      case 'pending':
        return {
          attestation,
          status: AttestationStatusKind.PENDING,
        }
      case 'bitcoin':
        return this.verifyBitcoinAttestation(input, attestation)
      case 'ethereum-uts':
        return this.verifyEthereumUTSAttestation(input, attestation)
      case 'unknown':
        return {
          attestation,
          status: AttestationStatusKind.UNKNOWN,
          error: new VerifyError(
            ErrorCode.UNSUPPORTED_ATTESTATION,
            `Unknown attestation with tag ${hexlify(attestation.tag)} cannot be verified`,
          ),
        }
    }
  }

  async verifyBitcoinAttestation(
    input: Uint8Array,
    attestation: BitcoinAttestation,
  ): Promise<AttestationStatus> {
    try {
      const header = await this.btcRPC
        .getBlockHash(attestation.height)
        .then((hash) => this.btcRPC.getBlockHeader(hash))
      // sha256d reverse the displayed hash, so we need to reverse it back to compare with the input
      const merkleRoot = getBytes(`0x${header.merkleroot}`).reverse()
      if (
        merkleRoot.length !== input.length ||
        !merkleRoot.every((byte, i) => byte === input[i])
      ) {
        return {
          attestation,
          status: AttestationStatusKind.INVALID,
          error: new VerifyError(
            ErrorCode.ATTESTATION_MISMATCH,
            `Bitcoin attestation does not match the expected merkle root at height ${attestation.height}`,
          ),
        }
      }
      return {
        attestation,
        status: AttestationStatusKind.VALID,
        additionalInfo: { header },
      }
    } catch (error) {
      console.error(`Error verifying Bitcoin attestation: ${error}`)
      return {
        attestation,
        status: AttestationStatusKind.UNKNOWN,
        error: new VerifyError(
          ErrorCode.REMOTE_ERROR,
          `Failed to verify Bitcoin attestation for height ${attestation.height}`,
          { context: { source: error } },
        ),
      }
    }
  }

  async verifyEthereumUTSAttestation(
    input: Uint8Array,
    attestation: EthereumUTSAttestation,
  ): Promise<AttestationStatus> {
    if (!Object.hasOwn(this.ethRPCs, attestation.chain)) {
      return {
        attestation,
        status: AttestationStatusKind.UNKNOWN,
        error: new VerifyError(
          ErrorCode.UNSUPPORTED_ATTESTATION,
          `No RPC provider configured for Ethereum chain ${attestation.chain}`,
        ),
      }
    }
    const provider = this.ethRPCs[attestation.chain]!

    try {
      const logs = await provider.getLogs({
        fromBlock: attestation.height,
        toBlock: attestation.height,
        topics: [
          SDK.utsLogTopic, // Topic 0: Attested(bytes32,address,uint256)
          hexlify(input), // Topic 1: digest
        ],
      })

      if (logs.length === 0) {
        return {
          attestation,
          status: AttestationStatusKind.INVALID,
          error: new VerifyError(
            ErrorCode.ATTESTATION_MISMATCH,
            `No attestation log found for block ${attestation.height} on chain ${attestation.chain}`,
          ),
        }
      }

      const log = SDK.utsInterface.parseLog(logs[0])!
      const root = log.args[0] // root
      const sender = log.args[1] // sender
      const timestamp = log.args[2] // timestamp

      // sanity check to ensure the root matches
      const rootBytes = getBytes(root)
      if (
        rootBytes.length !== input.length ||
        !rootBytes.every((byte: number, i: number) => byte === input[i])
      ) {
        return {
          attestation,
          status: AttestationStatusKind.INVALID,
          error: new VerifyError(
            ErrorCode.ATTESTATION_MISMATCH,
            `Attestation log found but root does not match expected digest for block ${attestation.height} on chain ${attestation.chain}`,
          ),
        }
      }

      return {
        attestation,
        status: AttestationStatusKind.VALID,
        additionalInfo: { root, sender, timestamp },
      }
    } catch (error) {
      console.error(`Error verifying Ethereum attestation: ${error}`)
      return {
        attestation,
        status: AttestationStatusKind.UNKNOWN,
        error: new VerifyError(
          ErrorCode.REMOTE_ERROR,
          `Failed to verify Ethereum attestation for block ${attestation.height} on chain ${attestation.chain}`,
          { context: { source: error } },
        ),
      }
    }
  }

  /**
   * Transform the individual attestation statuses into an overall verification status for the timestamp.
   *
   * The logic is as follows:
   * - If there is at least one VALID attestation:
   *  - If there are also INVALID or UNKNOWN attestations, the overall status is PARTIAL_VALID
   *  - Otherwise, the overall status is VALID
   * - If there are no VALID attestations, but at least one PENDING attestation, the overall status is PENDING
   * - If there are no VALID or PENDING attestations, the overall status is INVALID
   * @param attestations
   */
  transformResult(attestations: AttestationStatus[]): VerifyStatus {
    const counts = {
      [AttestationStatusKind.VALID]: 0,
      [AttestationStatusKind.INVALID]: 0,
      [AttestationStatusKind.PENDING]: 0,
      [AttestationStatusKind.UNKNOWN]: 0,
    }
    for (const attestation of attestations) {
      counts[attestation.status]++
    }

    let status: VerifyStatus = VerifyStatus.INVALID

    if (counts[AttestationStatusKind.VALID] > 0) {
      if (
        counts[AttestationStatusKind.INVALID] > 0 ||
        counts[AttestationStatusKind.UNKNOWN] > 0
      ) {
        status = VerifyStatus.PARTIAL_VALID
      } else {
        status = VerifyStatus.VALID
      }
    } else if (counts[AttestationStatusKind.PENDING] > 0) {
      status = VerifyStatus.PENDING
    }
    return status
  }

  executeStep(input: Uint8Array, step: ExecutionStep): Uint8Array {
    switch (step.op) {
      case 'APPEND':
        if (!step.data) {
          throw new VerifyError(
            ErrorCode.INVALID_STRUCTURE,
            `Missing data for APPEND operation`,
          )
        }
        return new Uint8Array([...input, ...getBytes(step.data)])
      case 'PREPEND':
        if (!step.data) {
          throw new VerifyError(
            ErrorCode.INVALID_STRUCTURE,
            `Missing data for PREPEND operation`,
          )
        }
        return new Uint8Array([...getBytes(step.data), ...input])
      case 'REVERSE':
        return new Uint8Array(input).reverse()
      case 'HEXLIFY':
        const str = hexlify(input).slice(2) // remove 0x prefix
        return SDK.encoder.encode(str)

      case 'SHA1':
        console.warn(
          'SHA1 encountered during verification, which is considered weak. Consider re-stamping with a stronger hash algorithm.',
        )
        return sha1(input)
      case 'RIPEMD160':
        console.warn(
          'RIPEMD160 encountered during verification, which is not encouraged. Consider re-stamping with a stronger hash algorithm.',
        )
        return ripemd160(input)
      case 'SHA256':
        return sha256(input)
      case 'KECCAK256':
        return keccak_256(input)

      default:
        throw new VerifyError(
          ErrorCode.INVALID_STRUCTURE,
          `Unsupported step ${step} in timestamp`,
        )
    }
  }
}
