package types

import "encoding/hex"

type Op byte

const (
	OpAttestation Op = 0x00
	OpSHA1        Op = 0x02
	OpRIPEMD160   Op = 0x03
	OpSHA256      Op = 0x08
	OpKECCAK256   Op = 0x67
	OpAPPEND      Op = 0xf0
	OpPREPEND     Op = 0xf1
	OpREVERSE     Op = 0xf2
	OpHEXLIFY     Op = 0xf3
	OpFORK        Op = 0xff
)

func (op Op) String() string {
	switch op {
	case OpAttestation:
		return "ATTESTATION"
	case OpSHA1:
		return "SHA1"
	case OpRIPEMD160:
		return "RIPEMD160"
	case OpSHA256:
		return "SHA256"
	case OpKECCAK256:
		return "KECCAK256"
	case OpAPPEND:
		return "APPEND"
	case OpPREPEND:
		return "PREPEND"
	case OpREVERSE:
		return "REVERSE"
	case OpHEXLIFY:
		return "HEXLIFY"
	case OpFORK:
		return "FORK"
	default:
		return "UNKNOWN"
	}
}

func (op Op) HasImmediate() bool {
	return op == OpAPPEND || op == OpPREPEND
}

func (op Op) IsControl() bool {
	return op == OpAttestation || op == OpFORK
}

func (op Op) IsDigest() bool {
	return op == OpSHA1 || op == OpRIPEMD160 || op == OpSHA256 || op == OpKECCAK256
}

func (op Op) Valid() bool {
	switch op {
	case OpAttestation, OpSHA1, OpRIPEMD160, OpSHA256, OpKECCAK256,
		OpAPPEND, OpPREPEND, OpREVERSE, OpHEXLIFY, OpFORK:
		return true
	default:
		return false
	}
}

func NewOp(b byte) (Op, bool) {
	op := Op(b)
	return op, op.Valid()
}

type Step interface {
	Op() Op
}

type baseStep struct {
	op Op
}

func (s *baseStep) Op() Op {
	return s.op
}

type AppendStep struct {
	baseStep
	Data []byte
	Next Timestamp
}

func NewAppendStep(data []byte, next Timestamp) *AppendStep {
	return &AppendStep{
		baseStep: baseStep{op: OpAPPEND},
		Data:     data,
		Next:     next,
	}
}

type PrependStep struct {
	baseStep
	Data []byte
	Next Timestamp
}

func NewPrependStep(data []byte, next Timestamp) *PrependStep {
	return &PrependStep{
		baseStep: baseStep{op: OpPREPEND},
		Data:     data,
		Next:     next,
	}
}

type ReverseStep struct {
	baseStep
	Next Timestamp
}

func NewReverseStep(next Timestamp) *ReverseStep {
	return &ReverseStep{
		baseStep: baseStep{op: OpREVERSE},
		Next:     next,
	}
}

type HexlifyStep struct {
	baseStep
	Next Timestamp
}

func NewHexlifyStep(next Timestamp) *HexlifyStep {
	return &HexlifyStep{
		baseStep: baseStep{op: OpHEXLIFY},
		Next:     next,
	}
}

type SHA256Step struct {
	baseStep
	Next Timestamp
}

func NewSHA256Step(next Timestamp) *SHA256Step {
	return &SHA256Step{
		baseStep: baseStep{op: OpSHA256},
		Next:     next,
	}
}

type Keccak256Step struct {
	baseStep
	Next Timestamp
}

func NewKeccak256Step(next Timestamp) *Keccak256Step {
	return &Keccak256Step{
		baseStep: baseStep{op: OpKECCAK256},
		Next:     next,
	}
}

type SHA1Step struct {
	baseStep
	Next Timestamp
}

func NewSHA1Step(next Timestamp) *SHA1Step {
	return &SHA1Step{
		baseStep: baseStep{op: OpSHA1},
		Next:     next,
	}
}

type RIPEMD160Step struct {
	baseStep
	Next Timestamp
}

func NewRIPEMD160Step(next Timestamp) *RIPEMD160Step {
	return &RIPEMD160Step{
		baseStep: baseStep{op: OpRIPEMD160},
		Next:     next,
	}
}

type ForkStep struct {
	baseStep
	Branches []Timestamp
}

func NewForkStep(branches []Timestamp) *ForkStep {
	return &ForkStep{
		baseStep: baseStep{op: OpFORK},
		Branches: branches,
	}
}

type Attestation interface {
	Tag() [8]byte
}

type AttestationStep struct {
	baseStep
	Attestation Attestation
}

func NewAttestationStep(att Attestation) *AttestationStep {
	return &AttestationStep{
		baseStep:    baseStep{op: OpAttestation},
		Attestation: att,
	}
}

type Timestamp []Step

type rawAttestationStep struct {
	baseStep
	tag  [8]byte
	data []byte
}

func (s *rawAttestationStep) Tag() [8]byte {
	return s.tag
}

func (s *rawAttestationStep) Data() []byte {
	return s.data
}

func newRawAttestationStep(tag [8]byte, data []byte) *rawAttestationStep {
	return &rawAttestationStep{
		baseStep: baseStep{op: OpAttestation},
		tag:      tag,
		data:     data,
	}
}

func (s *baseStep) String() string {
	return s.op.String()
}

func (s *AppendStep) String() string {
	return "APPEND " + hex.EncodeToString(s.Data)
}

func (s *PrependStep) String() string {
	return "PREPEND " + hex.EncodeToString(s.Data)
}

func (s *ReverseStep) String() string {
	return "REVERSE"
}

func (s *HexlifyStep) String() string {
	return "HEXLIFY"
}

func (s *SHA256Step) String() string {
	return "SHA256"
}

func (s *Keccak256Step) String() string {
	return "KECCAK256"
}

func (s *SHA1Step) String() string {
	return "SHA1"
}

func (s *RIPEMD160Step) String() string {
	return "RIPEMD160"
}

func (s *ForkStep) String() string {
	return "FORK"
}

func (s *AttestationStep) String() string {
	return "ATTESTATION"
}
