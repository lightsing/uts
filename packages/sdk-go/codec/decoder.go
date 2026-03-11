package codec

import (
	"bytes"

	"github.com/uts-dot/sdk-go/errors"
	"github.com/uts-dot/sdk-go/types"
)

type Decoder struct {
	data []byte
	pos  int
}

func NewDecoder(data []byte) *Decoder {
	return &Decoder{data: data, pos: 0}
}

func (d *Decoder) Remaining() int {
	return len(d.data) - d.pos
}

func (d *Decoder) checkBounds(n int) error {
	if d.pos+n > len(d.data) {
		return errors.ErrUnexpectedEof()
	}
	return nil
}

func (d *Decoder) CheckEOF() error {
	if d.Remaining() > 0 {
		return errors.NewDecodeError(errors.ErrCodeInvalidData, "expected end of stream but bytes remain", nil)
	}
	return nil
}

func (d *Decoder) ReadByte() (byte, error) {
	if err := d.checkBounds(1); err != nil {
		return 0, err
	}
	b := d.data[d.pos]
	d.pos++
	return b, nil
}

func (d *Decoder) ReadBytes(n int) ([]byte, error) {
	if err := d.checkBounds(n); err != nil {
		return nil, err
	}
	result := d.data[d.pos : d.pos+n]
	d.pos += n
	return result, nil
}

func (d *Decoder) ReadU32() (uint32, error) {
	var result uint32
	var shift uint

	for {
		b, err := d.ReadByte()
		if err != nil {
			return 0, err
		}

		val := uint32(b & 0x7f)
		if shift >= 28 {
			if shift == 28 && val > 0x0f {
				return 0, errors.ErrLEB128Overflow(32)
			}
			if shift > 28 {
				return 0, errors.ErrLEB128Overflow(32)
			}
		}

		result |= val << shift

		if b&0x80 == 0 {
			break
		}

		shift += 7
	}

	return result, nil
}

func (d *Decoder) ReadU64() (uint64, error) {
	var result uint64
	var shift uint

	for {
		b, err := d.ReadByte()
		if err != nil {
			return 0, err
		}

		val := uint64(b & 0x7f)
		if shift >= 63 {
			if shift == 63 && val > 0x01 {
				return 0, errors.ErrLEB128Overflow(64)
			}
			if shift > 63 {
				return 0, errors.ErrLEB128Overflow(64)
			}
		}

		result |= val << shift

		if b&0x80 == 0 {
			break
		}

		shift += 7
	}

	return result, nil
}

func (d *Decoder) ReadLengthPrefixed() ([]byte, error) {
	length, err := d.ReadU32()
	if err != nil {
		return nil, err
	}
	return d.ReadBytes(int(length))
}

func (d *Decoder) PeekOp() (types.Op, bool) {
	if d.Remaining() == 0 {
		return 0, false
	}
	return types.Op(d.data[d.pos]), true
}

func (d *Decoder) ReadOp() (types.Op, error) {
	b, err := d.ReadByte()
	if err != nil {
		return 0, err
	}
	op, ok := types.NewOp(b)
	if !ok {
		return 0, errors.ErrBadOpCode(b)
	}
	return op, nil
}

func (d *Decoder) ReadMagic() (byte, error) {
	magic, err := d.ReadBytes(len(MagicBytes))
	if err != nil {
		return 0, err
	}
	if !bytes.Equal(magic, MagicBytes) {
		return 0, errors.ErrBadMagic()
	}
	return d.ReadByte()
}

func (d *Decoder) ReadHeader() (*types.DigestHeader, error) {
	op, err := d.ReadOp()
	if err != nil {
		return nil, err
	}

	digestOp := types.DigestOp(op)
	if !digestOp.Valid() {
		return nil, errors.ErrExpectedDigestOp(op.String())
	}

	digestLen := digestOp.OutputSize()
	digest, err := d.ReadBytes(digestLen)
	if err != nil {
		return nil, err
	}

	return types.NewDigestHeader(digestOp, digest), nil
}

func (d *Decoder) readPendingAttestation() (*types.PendingAttestation, error) {
	uriBytes, err := d.ReadLengthPrefixed()
	if err != nil {
		return nil, err
	}

	uri := string(uriBytes)
	if len(uri) > types.MaxURILen {
		return nil, errors.ErrUriTooLong()
	}
	if !types.ValidateURI(uri) {
		return nil, errors.ErrInvalidUriChar()
	}

	return &types.PendingAttestation{URI: uri}, nil
}

func (d *Decoder) readBitcoinAttestation() (*types.BitcoinAttestation, error) {
	height, err := d.ReadU32()
	if err != nil {
		return nil, err
	}
	return &types.BitcoinAttestation{Height: height}, nil
}

func (d *Decoder) readEASAttestation() (*types.EASAttestation, error) {
	chainID, err := d.ReadU64()
	if err != nil {
		return nil, err
	}

	uidBytes, err := d.ReadBytes(32)
	if err != nil {
		return nil, err
	}

	var uid [32]byte
	copy(uid[:], uidBytes)

	return &types.EASAttestation{ChainID: chainID, UID: uid}, nil
}

func (d *Decoder) readEASTimestamped() (*types.EASTimestamped, error) {
	chainID, err := d.ReadU64()
	if err != nil {
		return nil, err
	}
	return &types.EASTimestamped{ChainID: chainID}, nil
}

func (d *Decoder) readAttestationFromData(data []byte, tag [8]byte) (types.Attestation, error) {
	inner := NewDecoder(data)

	switch tag {
	case types.BitcoinTag:
		return inner.readBitcoinAttestation()
	case types.PendingTag:
		return inner.readPendingAttestation()
	case types.EASAttestTag:
		return inner.readEASAttestation()
	case types.EASTimestampTag:
		return inner.readEASTimestamped()
	default:
		unknown := types.NewUnknownAttestation(tag, data)
		return unknown, nil
	}
}

func (d *Decoder) ReadAttestationStep() (*types.AttestationStep, error) {
	op, err := d.ReadOp()
	if err != nil {
		return nil, err
	}
	if op != types.OpAttestation {
		return nil, errors.NewDecodeError(errors.ErrCodeInvalidData,
			"expected ATTESTATION op", map[string]interface{}{"op": op.String()})
	}

	tagBytes, err := d.ReadBytes(TagSize)
	if err != nil {
		return nil, err
	}
	var tag [8]byte
	copy(tag[:], tagBytes)

	data, err := d.ReadLengthPrefixed()
	if err != nil {
		return nil, err
	}

	att, err := d.readAttestationFromData(data, tag)
	if err != nil {
		return nil, err
	}

	return types.NewAttestationStep(att), nil
}

func (d *Decoder) ReadExecutionStep(op types.Op) (types.Step, error) {
	switch op {
	case types.OpAPPEND:
		data, err := d.ReadLengthPrefixed()
		if err != nil {
			return nil, err
		}
		return types.NewAppendStep(data, nil), nil
	case types.OpPREPEND:
		data, err := d.ReadLengthPrefixed()
		if err != nil {
			return nil, err
		}
		return types.NewPrependStep(data, nil), nil
	case types.OpREVERSE:
		return types.NewReverseStep(nil), nil
	case types.OpHEXLIFY:
		return types.NewHexlifyStep(nil), nil
	case types.OpSHA256:
		return types.NewSHA256Step(nil), nil
	case types.OpKECCAK256:
		return types.NewKeccak256Step(nil), nil
	case types.OpSHA1:
		return types.NewSHA1Step(nil), nil
	case types.OpRIPEMD160:
		return types.NewRIPEMD160Step(nil), nil
	default:
		return nil, errors.ErrBadOpCode(byte(op))
	}
}

func (d *Decoder) ReadForkStep() (*types.ForkStep, error) {
	branches := make([]types.Timestamp, 0)

	for {
		op, ok := d.PeekOp()
		if !ok {
			return nil, errors.ErrUnexpectedEof()
		}

		if op == types.OpFORK {
			if _, err := d.ReadOp(); err != nil {
				return nil, err
			}
			ts, err := d.ReadTimestamp()
			if err != nil {
				return nil, err
			}
			branches = append(branches, ts)
		} else {
			ts, err := d.ReadTimestamp()
			if err != nil {
				return nil, err
			}
			branches = append(branches, ts)
			break
		}
	}

	if len(branches) < 2 {
		return nil, errors.NewDecodeError(errors.ErrCodeInvalidData,
			"fork step must have at least 2 branches", nil)
	}

	return types.NewForkStep(branches), nil
}

func (d *Decoder) ReadStep() (types.Step, error) {
	op, ok := d.PeekOp()
	if !ok {
		// Distinguish between true EOF and invalid opcode.
		if d.Remaining() == 0 {
			return nil, errors.ErrUnexpectedEof()
		}
		// There is data remaining but PeekOp could not decode a valid opcode.
		// Report a bad opcode error for the offending byte.
		return nil, errors.ErrBadOpCode(d.data[d.pos])
	}

	switch op {
	case types.OpFORK:
		return d.ReadForkStep()
	case types.OpAttestation:
		return d.ReadAttestationStep()
	default:
		_, err := d.ReadOp()
		if err != nil {
			return nil, err
		}
		return d.ReadExecutionStep(op)
	}
}

func (d *Decoder) ReadTimestamp() (types.Timestamp, error) {
	steps := make(types.Timestamp, 0)

	for d.Remaining() > 0 {
		step, err := d.ReadStep()
		if err != nil {
			return nil, err
		}
		steps = append(steps, step)

		if step.Op() == types.OpFORK || step.Op() == types.OpAttestation {
			break
		}
	}

	return steps, nil
}

func DecodeDetachedTimestamp(data []byte) (*types.DetachedTimestamp, error) {
	dec := NewDecoder(data)

	version, err := dec.ReadMagic()
	if err != nil {
		return nil, err
	}
	if version != 0x01 {
		return nil, errors.ErrBadVersion()
	}

	header, err := dec.ReadHeader()
	if err != nil {
		return nil, err
	}

	timestamp, err := dec.ReadTimestamp()
	if err != nil {
		return nil, err
	}

	return types.NewDetachedTimestamp(header, timestamp), nil
}
