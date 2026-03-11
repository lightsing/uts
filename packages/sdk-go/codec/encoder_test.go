package codec

import (
	"bytes"
	"testing"

	"github.com/uts-dot/sdk-go/types"
)

func TestNewEncoder(t *testing.T) {
	enc := NewEncoder()
	if enc == nil {
		t.Fatal("NewEncoder returned nil")
	}
	if len(enc.Bytes()) != 0 {
		t.Errorf("new encoder should have empty buffer, got %d bytes", len(enc.Bytes()))
	}
}

func TestEncoderWriteByte(t *testing.T) {
	enc := NewEncoder()
	enc.WriteByte(0x00).WriteByte(0xff).WriteByte(0x08)

	got := enc.Bytes()
	want := []byte{0x00, 0xff, 0x08}
	if !bytes.Equal(got, want) {
		t.Errorf("WriteByte() = %v, want %v", got, want)
	}
}

func TestEncoderWriteBytes(t *testing.T) {
	enc := NewEncoder()
	data := []byte{0x01, 0x02, 0x03, 0x04}
	enc.WriteBytes(data)

	if !bytes.Equal(enc.Bytes(), data) {
		t.Errorf("WriteBytes() = %v, want %v", enc.Bytes(), data)
	}
}

func TestEncoderWriteU32(t *testing.T) {
	tests := []struct {
		name  string
		input uint32
		want  []byte
	}{
		{"zero", 0, []byte{0x00}},
		{"one", 1, []byte{0x01}},
		{"127", 127, []byte{0x7f}},
		{"128", 128, []byte{0x80, 0x01}},
		{"300", 300, []byte{0xac, 0x02}},
		{"max", 0xffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0x0f}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			enc := NewEncoder()
			enc.WriteU32(tt.input)
			if !bytes.Equal(enc.Bytes(), tt.want) {
				t.Errorf("WriteU32(%d) = %v, want %v", tt.input, enc.Bytes(), tt.want)
			}
		})
	}
}

func TestEncoderWriteU64(t *testing.T) {
	tests := []struct {
		name  string
		input uint64
		want  []byte
	}{
		{"zero", 0, []byte{0x00}},
		{"one", 1, []byte{0x01}},
		{"127", 127, []byte{0x7f}},
		{"128", 128, []byte{0x80, 0x01}},
		{"max uint32", 0xffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0x0f}},
		{"large", 0x0fffffffffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			enc := NewEncoder()
			enc.WriteU64(tt.input)
			if !bytes.Equal(enc.Bytes(), tt.want) {
				t.Errorf("WriteU64(%d) = %v, want %v", tt.input, enc.Bytes(), tt.want)
			}
		})
	}
}

func TestEncoderWriteLengthPrefixed(t *testing.T) {
	enc := NewEncoder()
	data := []byte("hello")
	enc.WriteLengthPrefixed(data)

	want := append([]byte{0x05}, data...)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteLengthPrefixed() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteHeader(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	header := types.NewDigestHeader(types.DigestSHA256, digest)
	enc := NewEncoder()
	enc.WriteHeader(header)

	want := append([]byte{0x08}, digest...)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteHeader() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteHeaderSHA1(t *testing.T) {
	digest := make([]byte, 20)
	for i := range digest {
		digest[i] = byte(i)
	}

	header := types.NewDigestHeader(types.DigestSHA1, digest)
	enc := NewEncoder()
	enc.WriteHeader(header)

	want := append([]byte{0x02}, digest...)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteHeader() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteVersionedMagic(t *testing.T) {
	enc := NewEncoder()
	enc.WriteVersionedMagic(0x01)

	want := append(MagicBytes, 0x01)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteVersionedMagic() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteAppendStep(t *testing.T) {
	enc := NewEncoder()
	step := types.NewAppendStep([]byte("test"), nil)
	enc.WriteExecutionStep(step)

	want := []byte{0xf0, 0x04, 't', 'e', 's', 't'}
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteAppendStep() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWritePrependStep(t *testing.T) {
	enc := NewEncoder()
	step := types.NewPrependStep([]byte("test"), nil)
	enc.WriteExecutionStep(step)

	want := []byte{0xf1, 0x04, 't', 'e', 's', 't'}
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WritePrependStep() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteHashSteps(t *testing.T) {
	tests := []struct {
		name string
		op   byte
		step types.Step
	}{
		{"SHA256", 0x08, types.NewSHA256Step(nil)},
		{"KECCAK256", 0x67, types.NewKeccak256Step(nil)},
		{"SHA1", 0x02, types.NewSHA1Step(nil)},
		{"RIPEMD160", 0x03, types.NewRIPEMD160Step(nil)},
		{"REVERSE", 0xf2, types.NewReverseStep(nil)},
		{"HEXLIFY", 0xf3, types.NewHexlifyStep(nil)},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			enc := NewEncoder()
			enc.WriteExecutionStep(tt.step)
			if !bytes.Equal(enc.Bytes(), []byte{tt.op}) {
				t.Errorf("WriteExecutionStep() = %v, want %v", enc.Bytes(), []byte{tt.op})
			}
		})
	}
}

func TestEncoderWriteForkStep(t *testing.T) {
	enc := NewEncoder()
	branches := []types.Timestamp{
		{types.NewSHA256Step(nil)},
		{types.NewSHA256Step(nil)},
	}
	step := types.NewForkStep(branches)
	err := enc.WriteForkStep(step)
	if err != nil {
		t.Fatalf("WriteForkStep error: %v", err)
	}

	want := []byte{0xff, 0x08, 0x08}
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteForkStep() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteForkStepError(t *testing.T) {
	enc := NewEncoder()
	step := types.NewForkStep([]types.Timestamp{})
	err := enc.WriteForkStep(step)
	if err == nil {
		t.Error("expected error for empty fork step")
	}

	step = types.NewForkStep([]types.Timestamp{{types.NewSHA256Step(nil)}})
	err = enc.WriteForkStep(step)
	if err == nil {
		t.Error("expected error for single branch fork step")
	}
}

func TestEncoderWritePendingAttestation(t *testing.T) {
	enc := NewEncoder()
	att := &types.PendingAttestation{URI: "https://example.com"}
	err := enc.WritePendingAttestation(att)
	if err != nil {
		t.Fatalf("WritePendingAttestation error: %v", err)
	}

	uri := "https://example.com"
	want := append([]byte{byte(len(uri))}, uri...)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WritePendingAttestation() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWritePendingAttestationTrimSlash(t *testing.T) {
	enc := NewEncoder()
	att := &types.PendingAttestation{URI: "https://example.com/"}
	err := enc.WritePendingAttestation(att)
	if err != nil {
		t.Fatalf("WritePendingAttestation error: %v", err)
	}

	uri := "https://example.com"
	want := append([]byte{byte(len(uri))}, uri...)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WritePendingAttestation() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWritePendingAttestationInvalidURI(t *testing.T) {
	enc := NewEncoder()
	att := &types.PendingAttestation{URI: "https://example.com/invalid!@#$"}
	err := enc.WritePendingAttestation(att)
	if err == nil {
		t.Error("expected error for invalid URI")
	}
}

func TestEncoderWritePendingAttestationURITooLong(t *testing.T) {
	enc := NewEncoder()
	uri := make([]byte, types.MaxURILen+1)
	for i := range uri {
		uri[i] = 'a'
	}
	att := &types.PendingAttestation{URI: string(uri)}
	err := enc.WritePendingAttestation(att)
	if err == nil {
		t.Error("expected error for URI too long")
	}
}

func TestEncoderWriteBitcoinAttestation(t *testing.T) {
	enc := NewEncoder()
	att := &types.BitcoinAttestation{Height: 500000}
	enc.WriteBitcoinAttestation(att)

	want := EncodeU32(500000)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteBitcoinAttestation() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteEASAttestation(t *testing.T) {
	enc := NewEncoder()
	var uid [32]byte
	for i := range uid {
		uid[i] = byte(i)
	}
	att := &types.EASAttestation{ChainID: 1, UID: uid}
	enc.WriteEASAttestation(att)

	enc2 := NewEncoder()
	enc2.WriteU64(1)
	enc2.WriteBytes(uid[:])
	if !bytes.Equal(enc.Bytes(), enc2.Bytes()) {
		t.Errorf("WriteEASAttestation() = %v, want %v", enc.Bytes(), enc2.Bytes())
	}
}

func TestEncoderWriteEASTimestamped(t *testing.T) {
	enc := NewEncoder()
	att := &types.EASTimestamped{ChainID: 534352}
	enc.WriteEASTimestamped(att)

	want := EncodeU64(534352)
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteEASTimestamped() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncoderWriteAttestationStepPending(t *testing.T) {
	enc := NewEncoder()
	att := &types.PendingAttestation{URI: "https://example.com"}
	step := types.NewAttestationStep(att)
	err := enc.WriteAttestationStep(step)
	if err != nil {
		t.Fatalf("WriteAttestationStep error: %v", err)
	}

	if enc.Bytes()[0] != 0x00 {
		t.Errorf("expected attestation opcode 0x00, got 0x%02x", enc.Bytes()[0])
	}
	tag := enc.Bytes()[1:9]
	if !bytes.Equal(tag, types.PendingTag[:]) {
		t.Errorf("tag = %v, want %v", tag, types.PendingTag[:])
	}
}

func TestEncoderWriteAttestationStepBitcoin(t *testing.T) {
	enc := NewEncoder()
	att := &types.BitcoinAttestation{Height: 123}
	step := types.NewAttestationStep(att)
	err := enc.WriteAttestationStep(step)
	if err != nil {
		t.Fatalf("WriteAttestationStep error: %v", err)
	}

	if enc.Bytes()[0] != 0x00 {
		t.Errorf("expected attestation opcode 0x00, got 0x%02x", enc.Bytes()[0])
	}
	tag := enc.Bytes()[1:9]
	if !bytes.Equal(tag, types.BitcoinTag[:]) {
		t.Errorf("tag = %v, want %v", tag, types.BitcoinTag[:])
	}
}

func TestEncoderWriteAttestationStepEASAttestation(t *testing.T) {
	enc := NewEncoder()
	var uid [32]byte
	att := &types.EASAttestation{ChainID: 1, UID: uid}
	step := types.NewAttestationStep(att)
	err := enc.WriteAttestationStep(step)
	if err != nil {
		t.Fatalf("WriteAttestationStep error: %v", err)
	}

	if enc.Bytes()[0] != 0x00 {
		t.Errorf("expected attestation opcode 0x00, got 0x%02x", enc.Bytes()[0])
	}
	tag := enc.Bytes()[1:9]
	if !bytes.Equal(tag, types.EASAttestTag[:]) {
		t.Errorf("tag = %v, want %v", tag, types.EASAttestTag[:])
	}
}

func TestEncoderWriteAttestationStepEASTimestamped(t *testing.T) {
	enc := NewEncoder()
	att := &types.EASTimestamped{ChainID: 534352}
	step := types.NewAttestationStep(att)
	err := enc.WriteAttestationStep(step)
	if err != nil {
		t.Fatalf("WriteAttestationStep error: %v", err)
	}

	if enc.Bytes()[0] != 0x00 {
		t.Errorf("expected attestation opcode 0x00, got 0x%02x", enc.Bytes()[0])
	}
	tag := enc.Bytes()[1:9]
	if !bytes.Equal(tag, types.EASTimestampTag[:]) {
		t.Errorf("tag = %v, want %v", tag, types.EASTimestampTag[:])
	}
}

func TestEncoderWriteTimestamp(t *testing.T) {
	enc := NewEncoder()
	ts := types.Timestamp{
		types.NewSHA256Step(nil),
		types.NewAppendStep([]byte("test"), nil),
	}
	err := enc.WriteTimestamp(ts)
	if err != nil {
		t.Fatalf("WriteTimestamp error: %v", err)
	}

	want := []byte{0x08, 0xf0, 0x04, 't', 'e', 's', 't'}
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("WriteTimestamp() = %v, want %v", enc.Bytes(), want)
	}
}

func TestEncodeDetachedTimestamp(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}
	header := types.NewDigestHeader(types.DigestSHA256, digest)
	ts := types.Timestamp{
		types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
	}
	ots := types.NewDetachedTimestamp(header, ts)

	encoded, err := EncodeDetachedTimestamp(ots)
	if err != nil {
		t.Fatalf("EncodeDetachedTimestamp error: %v", err)
	}

	if !bytes.HasPrefix(encoded, MagicBytes) {
		t.Error("encoded timestamp should start with magic bytes")
	}
	if encoded[len(MagicBytes)] != 0x01 {
		t.Error("encoded timestamp should have version 0x01")
	}
}

func TestEncoderCapacityGrowth(t *testing.T) {
	enc := NewEncoderWithSize(10)
	for i := 0; i < 100; i++ {
		enc.WriteByte(byte(i))
	}
	if len(enc.Bytes()) != 100 {
		t.Errorf("expected 100 bytes, got %d", len(enc.Bytes()))
	}
}

func TestEncoderChaining(t *testing.T) {
	enc := NewEncoder()
	enc.WriteByte(0x01).WriteByte(0x02).WriteByte(0x03)

	want := []byte{0x01, 0x02, 0x03}
	if !bytes.Equal(enc.Bytes(), want) {
		t.Errorf("chaining = %v, want %v", enc.Bytes(), want)
	}
}
