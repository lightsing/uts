package codec

import (
	"strings"

	"github.com/uts-dot/sdk-go"
	"github.com/uts-dot/sdk-go/types"
)

type Encoder struct {
	buf []byte
}

func NewEncoder() *Encoder {
	return &Encoder{
		buf: make([]byte, 0, 1024),
	}
}

func NewEncoderWithSize(size int) *Encoder {
	return &Encoder{
		buf: make([]byte, 0, size),
	}
}

func (e *Encoder) Bytes() []byte {
	return e.buf
}

func (e *Encoder) ensureCapacity(n int) {
	if cap(e.buf)-len(e.buf) < n {
		newCap := cap(e.buf) * 2
		if newCap < len(e.buf)+n {
			newCap = len(e.buf) + n
		}
		newBuf := make([]byte, len(e.buf), newCap)
		copy(newBuf, e.buf)
		e.buf = newBuf
	}
}

func (e *Encoder) WriteByte(b byte) *Encoder {
	e.ensureCapacity(1)
	e.buf = append(e.buf, b)
	return e
}

func (e *Encoder) WriteBytes(data []byte) *Encoder {
	e.ensureCapacity(len(data))
	e.buf = append(e.buf, data...)
	return e
}

func (e *Encoder) WriteU32(n uint32) *Encoder {
	e.buf = append(e.buf, EncodeU32(n)...)
	return e
}

func (e *Encoder) WriteU64(n uint64) *Encoder {
	e.buf = append(e.buf, EncodeU64(n)...)
	return e
}

func (e *Encoder) WriteLengthPrefixed(data []byte) *Encoder {
	e.WriteU32(uint32(len(data)))
	e.WriteBytes(data)
	return e
}

func (e *Encoder) WriteOp(op types.Op) *Encoder {
	return e.WriteByte(byte(op))
}

func (e *Encoder) WriteVersionedMagic(version byte) *Encoder {
	e.WriteBytes(MagicBytes)
	e.WriteByte(version)
	return e
}

func (e *Encoder) WriteHeader(header *types.DigestHeader) *Encoder {
	e.WriteOp(types.Op(header.Kind))
	e.WriteBytes(header.DigestBytes())
	return e
}

func (e *Encoder) WriteExecutionStep(step types.Step) *Encoder {
	e.WriteOp(step.Op())
	switch s := step.(type) {
	case *types.AppendStep:
		e.WriteLengthPrefixed(s.Data)
	case *types.PrependStep:
		e.WriteLengthPrefixed(s.Data)
	}
	return e
}

func (e *Encoder) WriteForkStep(step *types.ForkStep) error {
	if len(step.Branches) < 2 {
		return uts.NewEncodeError(uts.ErrCodeInvalidData, "FORK step must have at least 2 branches", nil)
	}
	for _, branch := range step.Branches[:len(step.Branches)-1] {
		e.WriteOp(types.OpFORK)
		if err := e.WriteTimestamp(branch); err != nil {
			return err
		}
	}
	return e.WriteTimestamp(step.Branches[len(step.Branches)-1])
}

func (e *Encoder) WritePendingAttestation(att *types.PendingAttestation) error {
	uri := strings.TrimSuffix(att.URI, "/")
	if len(uri) > types.MaxURILen {
		return uts.NewEncodeError(uts.ErrCodeUriTooLong, "URI exceeds maximum length", nil)
	}
	if !types.ValidateURI(uri) {
		return uts.NewEncodeError(uts.ErrCodeInvalidUriChar, "invalid character in URI", nil)
	}
	e.WriteLengthPrefixed([]byte(uri))
	return nil
}

func (e *Encoder) WriteBitcoinAttestation(att *types.BitcoinAttestation) *Encoder {
	e.WriteU32(att.Height)
	return e
}

func (e *Encoder) WriteEASAttestation(att *types.EASAttestation) *Encoder {
	e.WriteU64(att.ChainID)
	e.WriteBytes(att.UID[:])
	return e
}

func (e *Encoder) WriteEASTimestamped(att *types.EASTimestamped) *Encoder {
	e.WriteU64(att.ChainID)
	return e
}

func (e *Encoder) WriteUnknownAttestation(att *types.UnknownAttestation) *Encoder {
	e.WriteBytes(att.Tag[:])
	e.WriteLengthPrefixed(att.Data)
	return e
}

func (e *Encoder) WriteAttestationStep(step *types.AttestationStep) error {
	e.WriteOp(types.OpAttestation)
	inner := NewEncoder()

	switch att := step.Attestation.(type) {
	case *types.PendingAttestation:
		e.WriteBytes(types.PendingTag[:])
		if err := inner.WritePendingAttestation(att); err != nil {
			return err
		}
		e.WriteLengthPrefixed(inner.Bytes())
	case *types.BitcoinAttestation:
		e.WriteBytes(types.BitcoinTag[:])
		inner.WriteBitcoinAttestation(att)
		e.WriteLengthPrefixed(inner.Bytes())
	case *types.EASAttestation:
		e.WriteBytes(types.EASAttestTag[:])
		inner.WriteEASAttestation(att)
		e.WriteLengthPrefixed(inner.Bytes())
	case *types.EASTimestamped:
		e.WriteBytes(types.EASTimestampTag[:])
		inner.WriteEASTimestamped(att)
		e.WriteLengthPrefixed(inner.Bytes())
	default:
		tag := step.Attestation.Tag()
		e.WriteBytes(tag[:])
		e.WriteLengthPrefixed([]byte{})
	}
	return nil
}

func (e *Encoder) WriteStep(step types.Step) error {
	switch s := step.(type) {
	case *types.ForkStep:
		return e.WriteForkStep(s)
	case *types.AttestationStep:
		return e.WriteAttestationStep(s)
	default:
		e.WriteExecutionStep(s)
		return nil
	}
}

func (e *Encoder) WriteTimestamp(ts types.Timestamp) error {
	for _, step := range ts {
		if err := e.WriteStep(step); err != nil {
			return err
		}
	}
	return nil
}

func EncodeDetachedTimestamp(ots *types.DetachedTimestamp) ([]byte, error) {
	enc := NewEncoder()
	enc.WriteVersionedMagic(0x01)
	enc.WriteHeader(ots.Header)
	if err := enc.WriteTimestamp(ots.Timestamp); err != nil {
		return nil, err
	}
	return enc.Bytes(), nil
}
