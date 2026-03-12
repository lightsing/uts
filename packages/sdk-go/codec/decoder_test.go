package codec

import (
	"bytes"
	"testing"

	"github.com/lightsing/uts/packages/sdk-go/errors"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

func TestDecoder_Remaining(t *testing.T) {
	data := []byte{0x01, 0x02, 0x03}
	dec := NewDecoder(data)

	if dec.Remaining() != 3 {
		t.Errorf("expected remaining 3, got %d", dec.Remaining())
	}

	dec.ReadByte()
	if dec.Remaining() != 2 {
		t.Errorf("expected remaining 2, got %d", dec.Remaining())
	}
}

func TestDecoder_ReadByte(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		want    byte
		wantErr bool
	}{
		{"read single byte", []byte{0x42}, 0x42, false},
		{"empty data", []byte{}, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadByte()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadByte() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("ReadByte() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_ReadBytes(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		n       int
		want    []byte
		wantErr bool
	}{
		{"read 3 bytes", []byte{0x01, 0x02, 0x03, 0x04}, 3, []byte{0x01, 0x02, 0x03}, false},
		{"read all bytes", []byte{0x01, 0x02}, 2, []byte{0x01, 0x02}, false},
		{"read too many", []byte{0x01}, 2, nil, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadBytes(tt.n)
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadBytes() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !bytes.Equal(got, tt.want) {
				t.Errorf("ReadBytes() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_ReadU32(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		want    uint32
		wantErr bool
	}{
		{"zero", []byte{0x00}, 0, false},
		{"single byte", []byte{0x7f}, 127, false},
		{"two bytes", []byte{0x80, 0x01}, 128, false},
		{"300", []byte{0xac, 0x02}, 300, false},
		{"max u32", []byte{0xff, 0xff, 0xff, 0xff, 0x0f}, 0xffffffff, false},
		{"empty", []byte{}, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadU32()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadU32() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("ReadU32() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_ReadU64(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		want    uint64
		wantErr bool
	}{
		{"zero", []byte{0x00}, 0, false},
		{"single byte", []byte{0x7f}, 127, false},
		{"two bytes", []byte{0x80, 0x01}, 128, false},
		{"large value", []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f}, 0x7fffffffffffffff, false},
		{"max u64", []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01}, 0xffffffffffffffff, false},
		{"empty", []byte{}, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadU64()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadU64() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("ReadU64() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_ReadLengthPrefixed(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		want    []byte
		wantErr bool
	}{
		{"empty", []byte{0x00}, []byte{}, false},
		{"short data", []byte{0x03, 0x01, 0x02, 0x03}, []byte{0x01, 0x02, 0x03}, false},
		{"incomplete", []byte{0x05, 0x01, 0x02}, nil, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadLengthPrefixed()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadLengthPrefixed() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !bytes.Equal(got, tt.want) {
				t.Errorf("ReadLengthPrefixed() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_PeekOp(t *testing.T) {
	tests := []struct {
		name     string
		data     []byte
		want     types.Op
		wantOk   bool
		consumed bool
	}{
		{"peek SHA256", []byte{0x08}, types.OpSHA256, true, false},
		{"peek FORK", []byte{0xff}, types.OpFORK, true, false},
		{"invalid op", []byte{0x99}, 0, false, false},
		{"empty", []byte{}, 0, false, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, ok := dec.PeekOp()
			if ok != tt.wantOk {
				t.Errorf("PeekOp() ok = %v, want %v", ok, tt.wantOk)
				return
			}
			if tt.wantOk && got != tt.want {
				t.Errorf("PeekOp() = %v, want %v", got, tt.want)
			}
			if !tt.consumed && dec.Remaining() != len(tt.data) {
				t.Errorf("PeekOp consumed data, remaining = %d, want %d", dec.Remaining(), len(tt.data))
			}
		})
	}
}

func TestDecoder_ReadOp(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		want    types.Op
		wantErr bool
	}{
		{"SHA256", []byte{0x08}, types.OpSHA256, false},
		{"FORK", []byte{0xff}, types.OpFORK, false},
		{"invalid", []byte{0x99}, 0, true},
		{"empty", []byte{}, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadOp()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadOp() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("ReadOp() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDecoder_ReadMagic(t *testing.T) {
	validMagic := append(append([]byte{}, MagicBytes...), 0x01)

	tests := []struct {
		name       string
		data       []byte
		wantVer    byte
		wantErr    bool
		errMatches func(error) bool
	}{
		{"valid magic", validMagic, 0x01, false, nil},
		{"invalid magic", append([]byte{0x00, 0x01, 0x02, 0x03}, make([]byte, 28)...), 0, true, func(err error) bool {
			return err.(*errors.DecodeError).Code == errors.ErrCodeBadMagic
		}},
		{"empty", []byte{}, 0, true, func(err error) bool {
			return err.(*errors.DecodeError).Code == errors.ErrCodeUnexpectedEof
		}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadMagic()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadMagic() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if tt.wantErr && tt.errMatches != nil && !tt.errMatches(err) {
				t.Errorf("ReadMagic() error = %v, doesn't match expected error type", err)
				return
			}
			if got != tt.wantVer {
				t.Errorf("ReadMagic() = %v, want %v", got, tt.wantVer)
			}
		})
	}
}

func TestDecoder_ReadHeader(t *testing.T) {
	sha256Digest := make([]byte, 32)
	for i := range sha256Digest {
		sha256Digest[i] = byte(i)
	}

	data := append([]byte{byte(types.OpSHA256)}, sha256Digest...)

	tests := []struct {
		name     string
		data     []byte
		wantKind types.DigestOp
		wantLen  int
		wantErr  bool
	}{
		{"SHA256 header", data, types.DigestSHA256, 32, false},
		{"invalid op", []byte{0x99}, 0, 0, true},
		{"empty", []byte{}, 0, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dec := NewDecoder(tt.data)
			got, err := dec.ReadHeader()
			if (err != nil) != tt.wantErr {
				t.Errorf("ReadHeader() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				if got.Kind() != tt.wantKind {
					t.Errorf("ReadHeader() kind = %v, want %v", got.Kind(), tt.wantKind)
				}
				if len(got.DigestBytes()) != tt.wantLen {
					t.Errorf("ReadHeader() digest len = %v, want %v", len(got.DigestBytes()), tt.wantLen)
				}
			}
		})
	}
}

func TestDecoder_ReadExecutionStep(t *testing.T) {
	t.Run("APPEND step", func(t *testing.T) {
		data := []byte{0x03, 'a', 'b', 'c'}
		dec := NewDecoder(data)
		step, err := dec.ReadExecutionStep(types.OpAPPEND)
		if err != nil {
			t.Fatalf("ReadExecutionStep() error = %v", err)
		}
		appendStep, ok := step.(*types.AppendStep)
		if !ok {
			t.Fatalf("expected AppendStep, got %T", step)
		}
		if string(appendStep.Data) != "abc" {
			t.Errorf("APPEND data = %v, want 'abc'", string(appendStep.Data))
		}
	})

	t.Run("PREPEND step", func(t *testing.T) {
		data := []byte{0x02, 'x', 'y'}
		dec := NewDecoder(data)
		step, err := dec.ReadExecutionStep(types.OpPREPEND)
		if err != nil {
			t.Fatalf("ReadExecutionStep() error = %v", err)
		}
		prependStep, ok := step.(*types.PrependStep)
		if !ok {
			t.Fatalf("expected PrependStep, got %T", step)
		}
		if string(prependStep.Data) != "xy" {
			t.Errorf("PREPEND data = %v, want 'xy'", string(prependStep.Data))
		}
	})

	t.Run("REVERSE step", func(t *testing.T) {
		dec := NewDecoder([]byte{})
		step, err := dec.ReadExecutionStep(types.OpREVERSE)
		if err != nil {
			t.Fatalf("ReadExecutionStep() error = %v", err)
		}
		if _, ok := step.(*types.ReverseStep); !ok {
			t.Fatalf("expected ReverseStep, got %T", step)
		}
	})

	t.Run("HEXLIFY step", func(t *testing.T) {
		dec := NewDecoder([]byte{})
		step, err := dec.ReadExecutionStep(types.OpHEXLIFY)
		if err != nil {
			t.Fatalf("ReadExecutionStep() error = %v", err)
		}
		if _, ok := step.(*types.HexlifyStep); !ok {
			t.Fatalf("expected HexlifyStep, got %T", step)
		}
	})
}

func TestDecoder_ReadBitcoinAttestation(t *testing.T) {
	data := []byte{0x80, 0x08} // 1024 in LEB128
	dec := NewDecoder(data)

	att, err := dec.readBitcoinAttestation()
	if err != nil {
		t.Fatalf("readBitcoinAttestation() error = %v", err)
	}
	if att.Height != 1024 {
		t.Errorf("height = %d, want 1024", att.Height)
	}
}

func TestDecoder_ReadPendingAttestation(t *testing.T) {
	t.Run("valid URI", func(t *testing.T) {
		uri := "https://example.com/calendar"
		data := append([]byte{byte(len(uri))}, []byte(uri)...)
		dec := NewDecoder(data)

		att, err := dec.readPendingAttestation()
		if err != nil {
			t.Fatalf("readPendingAttestation() error = %v", err)
		}
		if att.URI != uri {
			t.Errorf("URI = %v, want %v", att.URI, uri)
		}
	})

	t.Run("invalid URI char", func(t *testing.T) {
		uri := "https://example.com/calendar?query=bad char"
		data := append([]byte{byte(len(uri))}, []byte(uri)...)
		dec := NewDecoder(data)

		_, err := dec.readPendingAttestation()
		if err == nil {
			t.Error("expected error for invalid URI char")
		}
	})
}

func TestDecoder_ReadEASAttestation(t *testing.T) {
	chainID := uint64(534352) // Scroll
	chainIDBytes := EncodeU64(chainID)
	uid := [32]byte{}
	for i := range uid {
		uid[i] = byte(i)
	}

	data := append(chainIDBytes, uid[:]...)
	dec := NewDecoder(data)

	att, err := dec.readEASAttestation()
	if err != nil {
		t.Fatalf("readEASAttestation() error = %v", err)
	}
	if att.ChainID != chainID {
		t.Errorf("chainID = %d, want %d", att.ChainID, chainID)
	}
	if att.UID != uid {
		t.Errorf("UID mismatch")
	}
}

func TestDecoder_ReadEASTimestamped(t *testing.T) {
	chainID := uint64(1)
	data := EncodeU64(chainID)
	dec := NewDecoder(data)

	att, err := dec.readEASTimestamped()
	if err != nil {
		t.Fatalf("readEASTimestamped() error = %v", err)
	}
	if att.ChainID != chainID {
		t.Errorf("chainID = %d, want %d", att.ChainID, chainID)
	}
}

func TestDecoder_ReadAttestationStep(t *testing.T) {
	t.Run("Bitcoin attestation", func(t *testing.T) {
		innerData := EncodeU32(800000)
		data := []byte{byte(types.OpAttestation)}
		data = append(data, types.BitcoinTag[:]...)
		data = append(data, byte(len(innerData)))
		data = append(data, innerData...)

		dec := NewDecoder(data)
		step, err := dec.ReadAttestationStep()
		if err != nil {
			t.Fatalf("ReadAttestationStep() error = %v", err)
		}

		btcAtt, ok := step.Attestation.(*types.BitcoinAttestation)
		if !ok {
			t.Fatalf("expected BitcoinAttestation, got %T", step.Attestation)
		}
		if btcAtt.Height != 800000 {
			t.Errorf("height = %d, want 800000", btcAtt.Height)
		}
	})

	t.Run("Unknown attestation", func(t *testing.T) {
		unknownTag := [8]byte{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08}
		innerData := []byte{0xde, 0xad, 0xbe, 0xef}

		data := []byte{byte(types.OpAttestation)}
		data = append(data, unknownTag[:]...)
		data = append(data, byte(len(innerData)))
		data = append(data, innerData...)

		dec := NewDecoder(data)
		step, err := dec.ReadAttestationStep()
		if err != nil {
			t.Fatalf("ReadAttestationStep() error = %v", err)
		}

		unknownAtt, ok := step.Attestation.(*types.UnknownAttestation)
		if !ok {
			t.Fatalf("expected UnknownAttestation, got %T", step.Attestation)
		}
		tag := unknownAtt.Tag()
		if tag != unknownTag {
			t.Errorf("tag mismatch")
		}
	})
}

func TestDecoder_ReadTimestamp(t *testing.T) {
	t.Run("simple timestamp", func(t *testing.T) {
		enc := NewEncoder()
		enc.WriteOp(types.OpREVERSE)
		enc.WriteOp(types.OpHEXLIFY)

		dec := NewDecoder(enc.Bytes())
		ts, err := dec.ReadTimestamp()
		if err != nil {
			t.Fatalf("ReadTimestamp() error = %v", err)
		}
		if len(ts) != 2 {
			t.Errorf("expected 2 steps, got %d", len(ts))
		}
	})

	t.Run("timestamp with attestation", func(t *testing.T) {
		enc := NewEncoder()
		enc.WriteOp(types.OpSHA256)
		attStep := types.NewAttestationStep(&types.BitcoinAttestation{Height: 12345})
		if err := enc.WriteAttestationStep(attStep); err != nil {
			t.Fatalf("encode error: %v", err)
		}

		dec := NewDecoder(enc.Bytes())
		ts, err := dec.ReadTimestamp()
		if err != nil {
			t.Fatalf("ReadTimestamp() error = %v", err)
		}
		if len(ts) != 2 {
			t.Errorf("expected 2 steps, got %d", len(ts))
		}
		if ts[1].Op() != types.OpAttestation {
			t.Errorf("expected ATTESTATION step, got %v", ts[1].Op())
		}
	})
}

func TestDecoder_ReadForkStep(t *testing.T) {
	t.Run("valid fork", func(t *testing.T) {
		branch1 := types.Timestamp{types.NewSHA256Step(nil), types.NewAttestationStep(&types.BitcoinAttestation{Height: 100})}
		branch2 := types.Timestamp{types.NewKeccak256Step(nil), types.NewAttestationStep(&types.BitcoinAttestation{Height: 200})}
		forkStep := types.NewForkStep([]types.Timestamp{branch1, branch2})

		enc := NewEncoder()
		if err := enc.WriteForkStep(forkStep); err != nil {
			t.Fatalf("WriteForkStep error: %v", err)
		}

		dec := NewDecoder(enc.Bytes())
		fork, err := dec.ReadForkStep()
		if err != nil {
			t.Fatalf("ReadForkStep() error = %v", err)
		}
		if len(fork.Branches) != 2 {
			t.Errorf("expected 2 branches, got %d", len(fork.Branches))
		}
	})
}

func TestDecodeDetachedTimestamp(t *testing.T) {
	digest := [32]byte{}
	for i := range digest {
		digest[i] = byte(i)
	}
	header, err := types.NewDigestHeader(types.DigestSHA256, digest[:])
	if err != nil {
		t.Fatalf("NewDigestHeader() error = %v", err)
	}
	ts := types.Timestamp{
		types.NewReverseStep(nil),
		types.NewAttestationStep(&types.BitcoinAttestation{Height: 800000}),
	}
	ots := types.NewDetachedTimestamp(header, ts)

	data, err := EncodeDetachedTimestamp(ots)
	if err != nil {
		t.Fatalf("EncodeDetachedTimestamp() error = %v", err)
	}

	decoded, err := DecodeDetachedTimestamp(data)
	if err != nil {
		t.Fatalf("DecodeDetachedTimestamp() error = %v", err)
	}

	if decoded.Header.Kind() != header.Kind() {
		t.Errorf("header kind mismatch")
	}
	if len(decoded.Timestamp) != len(ts) {
		t.Errorf("timestamp length mismatch: got %d, want %d", len(decoded.Timestamp), len(ts))
	}
}

func TestRoundTrip(t *testing.T) {
	t.Run("simple timestamp", func(t *testing.T) {
		digest := [32]byte{0x01, 0x02, 0x03}
		header, err := types.NewDigestHeader(types.DigestSHA256, digest[:])
		if err != nil {
			t.Fatalf("NewDigestHeader() error = %v", err)
		}
		ts := types.Timestamp{
			types.NewAppendStep([]byte("hello"), nil),
			types.NewReverseStep(nil),
			types.NewAttestationStep(&types.EASAttestation{
				ChainID: 534352,
				UID:     [32]byte{0xaa, 0xbb, 0xcc},
			}),
		}
		original := types.NewDetachedTimestamp(header, ts)

		data, err := EncodeDetachedTimestamp(original)
		if err != nil {
			t.Fatalf("encode error: %v", err)
		}

		decoded, err := DecodeDetachedTimestamp(data)
		if err != nil {
			t.Fatalf("decode error: %v", err)
		}

		if decoded.Header.Kind() != original.Header.Kind() {
			t.Errorf("header kind mismatch")
		}
		if len(decoded.Timestamp) != len(original.Timestamp) {
			t.Errorf("timestamp length mismatch")
		}
	})

	t.Run("fork timestamp", func(t *testing.T) {
		digest := [32]byte{0xff}
		header, err := types.NewDigestHeader(types.DigestSHA256, digest[:])
		if err != nil {
			t.Fatalf("NewDigestHeader() error = %v", err)
		}
		ts := types.Timestamp{
			types.NewForkStep([]types.Timestamp{
				{types.NewReverseStep(nil), types.NewAttestationStep(&types.BitcoinAttestation{Height: 100})},
				{types.NewHexlifyStep(nil), types.NewAttestationStep(&types.BitcoinAttestation{Height: 200})},
			}),
		}
		original := types.NewDetachedTimestamp(header, ts)

		data, err := EncodeDetachedTimestamp(original)
		if err != nil {
			t.Fatalf("encode error: %v", err)
		}

		decoded, err := DecodeDetachedTimestamp(data)
		if err != nil {
			t.Fatalf("decode error: %v", err)
		}

		if len(decoded.Timestamp) != 1 {
			t.Fatalf("expected 1 step, got %d", len(decoded.Timestamp))
		}
		forkStep, ok := decoded.Timestamp[0].(*types.ForkStep)
		if !ok {
			t.Fatalf("expected ForkStep, got %T", decoded.Timestamp[0])
		}
		if len(forkStep.Branches) != 2 {
			t.Errorf("expected 2 branches, got %d", len(forkStep.Branches))
		}
	})

	t.Run("all attestation types", func(t *testing.T) {
		digest := [32]byte{0x99}
		header, err := types.NewDigestHeader(types.DigestKECCAK256, digest[:])
		if err != nil {
			t.Fatalf("NewDigestHeader() error = %v", err)
		}
		uid := [32]byte{}
		for i := range uid {
			uid[i] = byte(i)
		}

		ts := types.Timestamp{
			types.NewForkStep([]types.Timestamp{
				{types.NewAttestationStep(&types.PendingAttestation{URI: "https://calendar.example.com"})},
				{types.NewAttestationStep(&types.BitcoinAttestation{Height: 123456})},
				{types.NewAttestationStep(&types.EASAttestation{ChainID: 1, UID: uid})},
				{types.NewAttestationStep(&types.EASTimestamped{ChainID: 534352})},
			}),
		}
		original := types.NewDetachedTimestamp(header, ts)

		data, err := EncodeDetachedTimestamp(original)
		if err != nil {
			t.Fatalf("encode error: %v", err)
		}

		decoded, err := DecodeDetachedTimestamp(data)
		if err != nil {
			t.Fatalf("decode error: %v", err)
		}

		if len(decoded.Timestamp) != 1 {
			t.Fatalf("expected 1 step (fork), got %d", len(decoded.Timestamp))
		}
		forkStep, ok := decoded.Timestamp[0].(*types.ForkStep)
		if !ok {
			t.Fatalf("expected ForkStep, got %T", decoded.Timestamp[0])
		}
		if len(forkStep.Branches) != 4 {
			t.Errorf("expected 4 branches, got %d", len(forkStep.Branches))
		}

		for i, branch := range forkStep.Branches {
			if len(branch) != 1 {
				t.Errorf("branch %d: expected 1 step, got %d", i, len(branch))
			}
		}
	})
}

func TestDecodeErrors(t *testing.T) {
	t.Run("bad magic", func(t *testing.T) {
		_, err := DecodeDetachedTimestamp([]byte{0x00, 0x01, 0x02})
		if err == nil {
			t.Error("expected error for bad magic")
		}
	})

	t.Run("bad version", func(t *testing.T) {
		data := append(append([]byte{}, MagicBytes...), 0x99)
		_, err := DecodeDetachedTimestamp(data)
		if err == nil {
			t.Error("expected error for bad version")
		}
	})

	t.Run("unexpected EOF", func(t *testing.T) {
		data := append(append([]byte{}, MagicBytes...), 0x01)
		_, err := DecodeDetachedTimestamp(data)
		if err == nil {
			t.Error("expected error for unexpected EOF")
		}
	})
}
