import { describe, it, expect } from 'vitest';
import { createHash } from 'crypto';
import {Hasher, UnorderedMerkleTree} from "../src";

class Sha256Hasher implements Hasher {
    private hash = createHash('sha256');
    readonly outputSize = 32;

    update(data: Uint8Array): void {
        this.hash.update(data);
    }

    digest(): Uint8Array {
        const result = this.hash.digest();
        return new Uint8Array(result);
    }
}

const factory = {
    outputSize: 32,
    create: () => new Sha256Hasher()
};

describe('UnorderedMerkleTree', () => {
    it('should create a tree and compute root', () => {
        const leaves = [
            new Uint8Array(Array(32).fill(1)),
            new Uint8Array(Array(32).fill(2)),
            new Uint8Array(Array(32).fill(3)),
        ];

        const tree = UnorderedMerkleTree.new(leaves, factory);
        const root = tree.root();

        expect(root).toBeDefined();
        expect(root.length).toBe(32); // SHA256 output size
    });

    it('should verify leaf existence', () => {
        const leaves = [
            new Uint8Array(Array(32).fill(1)),
            new Uint8Array(Array(32).fill(2)),
        ];

        const tree = UnorderedMerkleTree.new(leaves, factory);

        expect(tree.contains(new Uint8Array(Array(32).fill(1)))).toBe(true);
        expect(tree.contains(new Uint8Array(Array(32).fill(9)))).toBe(false);
    });

    it('should generate valid proof iteration', () => {
        const leaves = [
            new Uint8Array(Array(32).fill(1)),
            new Uint8Array(Array(32).fill(2)),
            new Uint8Array(Array(32).fill(3)),
            new Uint8Array(Array(32).fill(4)),
        ];

        const tree = UnorderedMerkleTree.new(leaves, factory);
        const targetLeaf = new Uint8Array(Array(32).fill(4));

        const proofIter = tree.getProofIter(targetLeaf);
        expect(proofIter).not.toBeNull();

        const steps = Array.from(proofIter!);
        expect(steps.length).toBe(2);

        steps.forEach(step => {
            expect(step.position).toBeDefined();
            expect(step.sibling).toBeInstanceOf(Uint8Array);
        });
    });
});
