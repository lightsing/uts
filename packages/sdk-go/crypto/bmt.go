package crypto

import "errors"

const InnerNodePrefix = 0x01

type NodePosition int

const (
	PositionLeft NodePosition = iota
	PositionRight
)

type ProofStep struct {
	Position NodePosition
	Sibling  [HashSize]byte
}

type MerkleTree struct {
	nodes [][HashSize]byte
	len   int
}

func NewMerkleTree(leaves [][HashSize]byte) *MerkleTree {
	if len(leaves) == 0 {
		panic("Cannot create Merkle tree with zero leaves")
	}

	rawLen := len(leaves)
	treeLen := nextPowerOfTwo(rawLen)
	nodes := make([][HashSize]byte, 2*treeLen)

	for i := 0; i < rawLen; i++ {
		nodes[treeLen+i] = leaves[i]
	}

	for i := treeLen - 1; i >= 1; i-- {
		left := nodes[2*i]
		right := nodes[2*i+1]
		nodes[i] = hashInnerNode(left, right)
	}

	return &MerkleTree{
		nodes: nodes,
		len:   treeLen,
	}
}

func hashInnerNode(left, right [HashSize]byte) [HashSize]byte {
	data := make([]byte, 0, 1+HashSize*2)
	data = append(data, InnerNodePrefix)
	data = append(data, left[:]...)
	data = append(data, right[:]...)
	return SHA256(data)
}

func (t *MerkleTree) Root() [HashSize]byte {
	return t.nodes[1]
}

func (t *MerkleTree) Leaves() [][HashSize]byte {
	start := t.len
	end := t.len + t.len
	return t.nodes[start:end]
}

func (t *MerkleTree) Contains(leaf [HashSize]byte) bool {
	for _, l := range t.Leaves() {
		if l == leaf {
			return true
		}
	}
	return false
}

func (t *MerkleTree) GetProof(leaf [HashSize]byte) ([]ProofStep, error) {
	leaves := t.Leaves()
	leafIndex := -1
	for i, l := range leaves {
		if l == leaf {
			leafIndex = i
			break
		}
	}
	if leafIndex == -1 {
		return nil, errors.New("leaf not found in tree")
	}

	current := t.len + leafIndex
	var proof []ProofStep

	for current > 1 {
		var position NodePosition
		var siblingIndex int

		if current%2 == 0 {
			position = PositionLeft
			siblingIndex = current + 1
		} else {
			position = PositionRight
			siblingIndex = current - 1
		}

		proof = append(proof, ProofStep{
			Position: position,
			Sibling:  t.nodes[siblingIndex],
		})

		current = current / 2
	}

	return proof, nil
}

func VerifyProof(leaf [HashSize]byte, proof []ProofStep, root [HashSize]byte) bool {
	current := leaf

	for _, step := range proof {
		var left, right [HashSize]byte
		switch step.Position {
		case PositionLeft:
			left = current
			right = step.Sibling
		case PositionRight:
			left = step.Sibling
			right = current
		}
		current = hashInnerNode(left, right)
	}

	return current == root
}

func nextPowerOfTwo(n int) int {
	if n <= 0 {
		return 1
	}
	n--
	n |= n >> 1
	n |= n >> 2
	n |= n >> 4
	n |= n >> 8
	n |= n >> 16
	n |= n >> 32
	return n + 1
}

func (t *MerkleTree) AsRawBytes() []byte {
	result := make([]byte, len(t.nodes)*HashSize)
	for i, node := range t.nodes {
		copy(result[i*HashSize:], node[:])
	}
	return result
}

func MerkleTreeFromRawBytes(data []byte) *MerkleTree {
	if len(data)%HashSize != 0 {
		panic("data length must be a multiple of hash size")
	}
	numNodes := len(data) / HashSize
	if numNodes%2 != 0 {
		panic("number of nodes must be even")
	}
	nodes := make([][HashSize]byte, numNodes)
	for i := 0; i < numNodes; i++ {
		copy(nodes[i][:], data[i*HashSize:(i+1)*HashSize])
	}
	return &MerkleTree{
		nodes: nodes,
		len:   numNodes / 2,
	}
}
