package types

import "fmt"

type DigestOp byte

const (
	DigestSHA1      DigestOp = 0x02
	DigestRIPEMD160 DigestOp = 0x03
	DigestSHA256    DigestOp = 0x08
	DigestKECCAK256 DigestOp = 0x67
)

func (op DigestOp) String() string {
	switch op {
	case DigestSHA1:
		return "SHA1"
	case DigestRIPEMD160:
		return "RIPEMD160"
	case DigestSHA256:
		return "SHA256"
	case DigestKECCAK256:
		return "KECCAK256"
	default:
		return "UNKNOWN"
	}
}

func (op DigestOp) Valid() bool {
	switch op {
	case DigestSHA1, DigestRIPEMD160, DigestSHA256, DigestKECCAK256:
		return true
	default:
		return false
	}
}

func (op DigestOp) OutputSize() int {
	switch op {
	case DigestSHA1:
		return 20
	case DigestRIPEMD160:
		return 20
	case DigestSHA256:
		return 32
	case DigestKECCAK256:
		return 32
	default:
		return 0
	}
}

func NewDigestOp(b byte) (DigestOp, bool) {
	op := DigestOp(b)
	return op, op.Valid()
}

type DigestHeader struct {
	kind   DigestOp
	digest []byte
}

func NewDigestHeader(kind DigestOp, digest []byte) (*DigestHeader, error) {
	if len(digest) != kind.OutputSize() {
		return nil, fmt.Errorf("invalid digest length: expected %d, got %d", kind.OutputSize(), len(digest))
	}
	h := &DigestHeader{kind: kind, digest: make([]byte, len(digest))}
	copy(h.digest, digest)
	return h, nil
}

func (h *DigestHeader) Kind() DigestOp {
	return h.kind
}

func (h *DigestHeader) DigestBytes() []byte {
	return h.digest
}

func (h *DigestHeader) String() string {
	return fmt.Sprintf("%s %x", h.kind, h.DigestBytes())
}

type DetachedTimestamp struct {
	Header    *DigestHeader
	Timestamp Timestamp
}

func NewDetachedTimestamp(header *DigestHeader, ts Timestamp) *DetachedTimestamp {
	return &DetachedTimestamp{
		Header:    header,
		Timestamp: ts,
	}
}

func (dt *DetachedTimestamp) String() string {
	return fmt.Sprintf("digest of %s\n%s", dt.Header, dt.Timestamp)
}
