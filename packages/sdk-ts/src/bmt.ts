import { type CHash } from '@noble/hashes/utils.js'

export const INNER_NODE_PREFIX: number = 0x01
const prefixBuffer = new Uint8Array([INNER_NODE_PREFIX])

export enum NodePosition {
  /** The sibling is a right child, `APPEND` its hash when computing the parent */
  Left = 'LEFT',
  /** The sibling is a left child, `PREPEND` its hash when computing the parent */
  Right = 'RIGHT',
}

export class UnhashedFlatMerkleTree<T extends CHash> {
  constructor(
    public readonly buffer: Uint8Array[],
    public readonly len: number,
  ) {}

  /**
   * Finalizes the Merkle tree by hashing internal nodes
   */
  public finalize(factory: T): UnorderedMerkleTree<T> {
    const nodes = this.buffer
    const len = this.len

    // Build the tree (from bottom to top)
    for (let i = len - 1; i >= 1; i--) {
      const left = nodes[2 * i]
      const right = nodes[2 * i + 1]

      const hasher = factory.create()
      hasher.update(prefixBuffer)
      hasher.update(left)
      hasher.update(right)
      nodes[i] = hasher.digest()
    }

    return new UnorderedMerkleTree(nodes, len, factory)
  }
}

/**
 * Flat, Fixed-Size, Read only Merkle Tree
 * * Expects the length of leaves to be equal or near(less) to a power of two.
 * Leaves are **sorted** starting at index `len`.
 */
export class UnorderedMerkleTree<T extends CHash> {
  /** Index 0 is not used, leaves start at index `len` */
  protected nodes: Uint8Array[]
  protected len: number

  private readonly factory: T

  constructor(nodes: Uint8Array[], len: number, factory: T) {
    this.nodes = nodes
    this.len = len
    this.factory = factory
  }

  /**
   * Constructs a new Merkle tree from the given hash leaves.
   */
  public static new<T extends CHash>(
    data: Uint8Array[],
    factory: T,
  ): UnorderedMerkleTree<T> {
    return UnorderedMerkleTree.newUnhashed(data, factory).finalize(factory)
  }

  /**
   * Constructs a new Merkle tree from the given hash leaves, without hashing internal nodes.
   */
  public static newUnhashed<T extends CHash>(
    prehashedLeaves: Uint8Array[],
    factory: T,
  ): UnhashedFlatMerkleTree<T> {
    const rawLen = prehashedLeaves.length
    if (rawLen === 0) {
      throw new Error('Cannot create Merkle tree with zero leaves')
    }

    const len = nextPowerOfTwo(rawLen)
    const nodes = new Array<Uint8Array>(2 * len)

    // index 0, we will never use it
    nodes[0] = new Uint8Array(factory.outputLen)

    // Prepare leaves block
    const leavesBlock = new Array<Uint8Array>(len)
    for (let i = 0; i < len; i++) {
      if (i < rawLen) {
        if (prehashedLeaves[i].length !== factory.outputLen) {
          throw new Error(
            `Invalid leaf at index ${i}: expected length ${factory.outputLen}, got ${prehashedLeaves[i].length}`,
          )
        }
        leavesBlock[i] = prehashedLeaves[i]
      } else {
        // Pad with default (zeroed) hash
        leavesBlock[i] = new Uint8Array(factory.outputLen)
      }
    }

    // Copy back to tree nodes
    for (let i = 0; i < len; i++) {
      nodes[len + i] = leavesBlock[i]
    }

    return new UnhashedFlatMerkleTree(nodes, len)
  }

  /**
   * Returns the root hash of the Merkle tree
   */
  public root(): Uint8Array {
    return this.nodes[1]
  }

  /**
   * Returns the leaves of the Merkle tree
   */
  public leaves(): Uint8Array[] {
    return this.nodes.slice(this.len, this.len * 2)
  }

  /**
   * Checks if the given leaf is contained in the Merkle tree
   */
  public contains(leaf: Uint8Array): boolean {
    return binarySearch(this.leaves(), leaf) !== -1
  }

  /**
   * Get proof for a given leaf
   */
  public getProofIter(leaf: Uint8Array): SiblingIter | null {
    const leafIndexInSlice = binarySearch(this.leaves(), leaf)
    if (leafIndexInSlice === -1) {
      return null
    }

    return new SiblingIter(this.nodes, this.len + leafIndexInSlice)
  }

  /**
   * Returns the raw bytes of the Merkle tree nodes (mimicking bytemuck::cast_slice)
   */
  public asRawBytes(): Uint8Array {
    const totalSize = this.nodes.length * this.factory.outputLen
    const bytes = new Uint8Array(totalSize)

    for (let i = 0; i < this.nodes.length; i++) {
      bytes.set(this.nodes[i], i * this.factory.outputLen)
    }

    return bytes
  }

  /**
   * From raw bytes, reconstruct the Merkle tree
   */
  public static fromRawBytes<T extends CHash>(
    bytes: Uint8Array,
    factory: T,
  ): UnorderedMerkleTree<T> {
    if (bytes.length % factory.outputLen !== 0) {
      throw new Error('Bytes length must be a multiple of hashLength')
    }
    const totalNodes = bytes.length / factory.outputLen
    if (totalNodes % 2 !== 0) {
      throw new Error('Invalid tree structure: node count is not even')
    }

    const len = totalNodes / 2
    const nodes = new Array<Uint8Array>(totalNodes)

    for (let i = 0; i < totalNodes; i++) {
      nodes[i] = bytes.slice(i * factory.outputLen, (i + 1) * factory.outputLen)
    }

    return new UnorderedMerkleTree<T>(nodes, len, factory)
  }
}

/**
 * Iterator over the sibling nodes of a leaf in the Merkle tree
 */
export class SiblingIter implements IterableIterator<{
  position: NodePosition
  sibling: Uint8Array
}> {
  private readonly nodes: Uint8Array[]
  private current: number

  constructor(nodes: Uint8Array[], current: number) {
    this.nodes = nodes
    this.current = current
  }

  public next(): IteratorResult<{
    position: NodePosition
    sibling: Uint8Array
  }> {
    if (this.current <= 1) {
      return { done: true, value: undefined }
    }

    const isLeft = (this.current & 1) === 0
    const position = isLeft ? NodePosition.Left : NodePosition.Right

    const siblingIndex = this.current ^ 1
    const sibling = this.nodes[siblingIndex]

    this.current >>= 1

    return {
      done: false,
      value: { position, sibling },
    }
  }

  public [Symbol.iterator](): IterableIterator<{
    position: NodePosition
    sibling: Uint8Array
  }> {
    return this
  }

  /** Returns the exact remaining size of the iterator */
  public get length(): number {
    if (this.current <= 1) return 0
    return 31 - Math.clz32(this.current)
  }
}

// --- Helper Functions ---

function nextPowerOfTwo(n: number): number {
  if (n <= 1) return 1
  let p = 1
  while (p < n) p *= 2
  return p
}

function compareBytes(a: Uint8Array, b: Uint8Array): number {
  const len = Math.min(a.length, b.length)
  for (let i = 0; i < len; i++) {
    if (a[i] !== b[i]) {
      return a[i] - b[i]
    }
  }
  return a.length - b.length
}

function binarySearch(arr: Uint8Array[], target: Uint8Array): number {
  let left = 0
  let right = arr.length - 1

  while (left <= right) {
    const mid = (left + right) >>> 1
    const cmp = compareBytes(arr[mid], target)

    if (cmp === 0) {
      return mid
    } else if (cmp < 0) {
      left = mid + 1
    } else {
      right = mid - 1
    }
  }

  return -1
}
