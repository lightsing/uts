package crypto

import (
	"testing"
)

func TestNewMerkleTree(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
		SHA256([]byte("leaf4")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if tree.len != 4 {
		t.Errorf("expected len 4, got %d", tree.len)
	}

	if len(tree.nodes) != 8 {
		t.Errorf("expected 8 nodes, got %d", len(tree.nodes))
	}

	leavesSlice := tree.Leaves()
	if len(leavesSlice) != 4 {
		t.Errorf("expected 4 leaves, got %d", len(leavesSlice))
	}
}

func TestNewMerkleTreeNonPowerOfTwo(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if tree.len != 4 {
		t.Errorf("expected len 4 (next power of two), got %d", tree.len)
	}

	if len(tree.nodes) != 8 {
		t.Errorf("expected 8 nodes, got %d", len(tree.nodes))
	}
}

func TestMerkleTreeRoot(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
		SHA256([]byte("leaf4")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	leftHash := hashInnerNode(leaves[0], leaves[1])
	rightHash := hashInnerNode(leaves[2], leaves[3])
	expectedRoot := hashInnerNode(leftHash, rightHash)

	root := tree.Root()
	if root != expectedRoot {
		t.Errorf("root mismatch: expected %x, got %x", expectedRoot, root)
	}
}

func TestMerkleTreeContains(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
		SHA256([]byte("leaf4")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if !tree.Contains(leaves[0]) {
		t.Error("expected tree to contain leaves[0]")
	}

	if !tree.Contains(leaves[3]) {
		t.Error("expected tree to contain leaves[3]")
	}

	notInTree := SHA256([]byte("not_in_tree"))
	if tree.Contains(notInTree) {
		t.Error("expected tree to not contain notInTree")
	}
}

func TestMerkleTreeGetProof(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("apple")),
		SHA256([]byte("banana")),
		SHA256([]byte("cherry")),
		SHA256([]byte("date")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	for _, leaf := range leaves {
		proof, err := tree.GetProof(leaf)
		if err != nil {
			t.Errorf("failed to get proof: %v", err)
			continue
		}

		if !VerifyProof(leaf, proof, tree.Root()) {
			t.Errorf("proof verification failed for leaf")
		}
	}
}

func TestMerkleTreeGetProofNotFound(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	notInTree := SHA256([]byte("not_in_tree"))
	_, err = tree.GetProof(notInTree)
	if err == nil {
		t.Error("expected error for leaf not in tree")
	}
}

func TestVerifyProof(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("apple")),
		SHA256([]byte("banana")),
		SHA256([]byte("cherry")),
		SHA256([]byte("date")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	for _, leaf := range leaves {
		proof, _ := tree.GetProof(leaf)
		if !VerifyProof(leaf, proof, tree.Root()) {
			t.Errorf("proof verification failed for leaf")
		}
	}

	wrongRoot := SHA256([]byte("wrong_root"))
	for _, leaf := range leaves {
		proof, _ := tree.GetProof(leaf)
		if VerifyProof(leaf, proof, wrongRoot) {
			t.Error("proof should not verify against wrong root")
		}
	}
}

func TestProofStructure(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf0")),
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	proof, _ := tree.GetProof(leaves[0])
	if len(proof) != 2 {
		t.Errorf("expected 2 proof steps for 4 leaves, got %d", len(proof))
	}

	proof, _ = tree.GetProof(leaves[3])
	if len(proof) != 2 {
		t.Errorf("expected 2 proof steps for 4 leaves, got %d", len(proof))
	}
}

func TestSingleNodeTree(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("single")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if tree.len != 1 {
		t.Errorf("expected len 1, got %d", tree.len)
	}

	if tree.Root() != leaves[0] {
		t.Error("root should equal the single leaf")
	}

	proof, err := tree.GetProof(leaves[0])
	if err != nil {
		t.Errorf("failed to get proof: %v", err)
	}

	if len(proof) != 0 {
		t.Errorf("expected 0 proof steps for single node, got %d", len(proof))
	}

	if !VerifyProof(leaves[0], proof, tree.Root()) {
		t.Error("proof verification failed for single node")
	}
}

func TestNextPowerOfTwo(t *testing.T) {
	tests := []struct {
		input    int
		expected int
	}{
		{1, 1},
		{2, 2},
		{3, 4},
		{4, 4},
		{5, 8},
		{7, 8},
		{8, 8},
		{9, 16},
		{15, 16},
		{16, 16},
		{17, 32},
		{1023, 1024},
		{1024, 1024},
	}

	for _, tt := range tests {
		result := nextPowerOfTwo(tt.input)
		if result != tt.expected {
			t.Errorf("nextPowerOfTwo(%d) = %d, expected %d", tt.input, result, tt.expected)
		}
	}
}

func TestAsRawBytes(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
		SHA256([]byte("leaf4")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}
	raw := tree.AsRawBytes()

	expectedLen := len(tree.nodes) * HashSize
	if len(raw) != expectedLen {
		t.Errorf("expected raw bytes length %d, got %d", expectedLen, len(raw))
	}
}

func TestMerkleTreeFromRawBytes(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
		SHA256([]byte("leaf4")),
	}

	original, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}
	raw := original.AsRawBytes()

	reconstructed := MerkleTreeFromRawBytes(raw)

	if reconstructed.Root() != original.Root() {
		t.Error("reconstructed tree has different root")
	}

	if reconstructed.len != original.len {
		t.Errorf("reconstructed tree has different len: %d vs %d", reconstructed.len, original.len)
	}

	for i, leaf := range leaves {
		reconstructedProof, _ := reconstructed.GetProof(leaf)

		if !VerifyProof(leaf, reconstructedProof, reconstructed.Root()) {
			t.Errorf("proof verification failed for leaf %d in reconstructed tree", i)
		}
	}
}

func TestLargeTree(t *testing.T) {
	numLeaves := 1024
	leaves := make([][HashSize]byte, numLeaves)
	for i := 0; i < numLeaves; i++ {
		leaves[i] = SHA256([]byte{byte(i >> 8), byte(i)})
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if tree.len != 1024 {
		t.Errorf("expected len 1024, got %d", tree.len)
	}

	for i := 0; i < numLeaves; i++ {
		proof, err := tree.GetProof(leaves[i])
		if err != nil {
			t.Errorf("failed to get proof for leaf %d: %v", i, err)
			continue
		}
		if !VerifyProof(leaves[i], proof, tree.Root()) {
			t.Errorf("proof verification failed for leaf %d", i)
		}
	}
}

func TestNonPowerOfTwoPadding(t *testing.T) {
	leaves := [][HashSize]byte{
		SHA256([]byte("leaf1")),
		SHA256([]byte("leaf2")),
		SHA256([]byte("leaf3")),
	}

	tree, err := NewMerkleTree(leaves)
	if err != nil {
		t.Fatalf("failed to create Merkle tree: %v", err)
	}

	if tree.len != 4 {
		t.Errorf("expected len 4 (padded), got %d", tree.len)
	}

	treeLeaves := tree.Leaves()
	if len(treeLeaves) != 4 {
		t.Errorf("expected 4 leaves (padded), got %d", len(treeLeaves))
	}

	var zeroHash [HashSize]byte
	if treeLeaves[3] != zeroHash {
		t.Error("expected padding leaf to be zero hash")
	}

	for _, leaf := range leaves {
		proof, _ := tree.GetProof(leaf)
		if !VerifyProof(leaf, proof, tree.Root()) {
			t.Error("proof verification failed with non-power-of-two tree")
		}
	}
}
