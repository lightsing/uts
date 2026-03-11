import { describe, it, expect } from 'vitest'
import { UnorderedMerkleTree } from '../src/bmt.ts'
import { sha256 } from '@noble/hashes/sha2.js'

describe('UnorderedMerkleTree', () => {
  it('should create a tree and compute root', () => {
    const leaves = [
      new Uint8Array(Array(32).fill(1)),
      new Uint8Array(Array(32).fill(2)),
      new Uint8Array(Array(32).fill(3)),
    ]

    const tree = UnorderedMerkleTree.new(leaves, sha256)
    const root = tree.root()

    expect(root).toBeDefined()
    expect(root.length).toBe(32) // SHA256 output size
  })

  it('should verify leaf existence', () => {
    const leaves = [
      new Uint8Array(Array(32).fill(1)),
      new Uint8Array(Array(32).fill(2)),
    ]

    const tree = UnorderedMerkleTree.new(leaves, sha256)

    expect(tree.contains(new Uint8Array(Array(32).fill(1)))).toBe(true)
    expect(tree.contains(new Uint8Array(Array(32).fill(9)))).toBe(false)
  })

  it('should generate valid proof iteration', () => {
    const leaves = [
      new Uint8Array(Array(32).fill(1)),
      new Uint8Array(Array(32).fill(2)),
      new Uint8Array(Array(32).fill(3)),
      new Uint8Array(Array(32).fill(4)),
    ]

    const tree = UnorderedMerkleTree.new(leaves, sha256)
    const targetLeaf = new Uint8Array(Array(32).fill(4))

    const proofIter = tree.getProofIter(targetLeaf)
    expect(proofIter).not.toBeNull()

    const steps = Array.from(proofIter!)
    expect(steps.length).toBe(2)

    steps.forEach((step) => {
      expect(step.position).toBeDefined()
      expect(step.sibling).toBeInstanceOf(Uint8Array)
    })
  })

  it('should serialize and deserialize correctly', () => {
    const leaves = [
      new Uint8Array(Array(32).fill(1)),
      new Uint8Array(Array(32).fill(2)),
      new Uint8Array(Array(32).fill(3)),
    ]

    const tree = UnorderedMerkleTree.new(leaves, sha256)
    const rawBytes = tree.asRawBytes()

    const deserializedTree = UnorderedMerkleTree.fromRawBytes(rawBytes, sha256)
    expect(deserializedTree.root()).toEqual(tree.root())
  })
})
