package types

import (
	"bytes"
	"testing"
)

func TestDigestOpString(t *testing.T) {
	tests := []struct {
		op   DigestOp
		want string
	}{
		{DigestSHA1, "SHA1"},
		{DigestRIPEMD160, "RIPEMD160"},
		{DigestSHA256, "SHA256"},
		{DigestKECCAK256, "KECCAK256"},
		{DigestOp(0x99), "UNKNOWN"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.op.String(); got != tt.want {
				t.Errorf("DigestOp.String() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDigestOpValid(t *testing.T) {
	tests := []struct {
		op   DigestOp
		want bool
	}{
		{DigestSHA1, true},
		{DigestRIPEMD160, true},
		{DigestSHA256, true},
		{DigestKECCAK256, true},
		{DigestOp(0x00), false},
		{DigestOp(0x99), false},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.Valid(); got != tt.want {
				t.Errorf("DigestOp.Valid() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDigestOpOutputSize(t *testing.T) {
	tests := []struct {
		op   DigestOp
		want int
	}{
		{DigestSHA1, 20},
		{DigestRIPEMD160, 20},
		{DigestSHA256, 32},
		{DigestKECCAK256, 32},
		{DigestOp(0x99), 0},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.OutputSize(); got != tt.want {
				t.Errorf("DigestOp.OutputSize() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestNewDigestOp(t *testing.T) {
	tests := []struct {
		b      byte
		want   DigestOp
		wantOk bool
	}{
		{0x02, DigestSHA1, true},
		{0x03, DigestRIPEMD160, true},
		{0x08, DigestSHA256, true},
		{0x67, DigestKECCAK256, true},
		{0x99, DigestOp(0x99), false},
	}

	for _, tt := range tests {
		t.Run(string(rune(tt.b)), func(t *testing.T) {
			got, gotOk := NewDigestOp(tt.b)
			if got != tt.want || gotOk != tt.wantOk {
				t.Errorf("NewDigestOp() = (%v, %v), want (%v, %v)", got, gotOk, tt.want, tt.wantOk)
			}
		})
	}
}

func TestNewDigestHeader(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	header, err := NewDigestHeader(DigestSHA256, digest)
	if err != nil {
		t.Fatalf("NewDigestHeader() error = %v", err)
	}

	if header.Kind() != DigestSHA256 {
		t.Errorf("DigestHeader.Kind = %v, want %v", header.Kind(), DigestSHA256)
	}

	if !bytes.Equal(header.DigestBytes(), digest) {
		t.Errorf("DigestHeader.Digest not set correctly")
	}
}

func TestDigestHeaderDigestBytes(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	tests := []struct {
		name      string
		kind      DigestOp
		wantLen   int
		wantBytes []byte
	}{
		{"SHA256", DigestSHA256, 32, digest},
		{"SHA1", DigestSHA1, 20, digest[:20]},
		{"RIPEMD160", DigestRIPEMD160, 20, digest[:20]},
		{"KECCAK256", DigestKECCAK256, 32, digest},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			header, err := NewDigestHeader(tt.kind, digest)
			if err != nil {
				t.Fatalf("NewDigestHeader() error = %v", err)
			}
			got := header.DigestBytes()
			if len(got) != tt.wantLen {
				t.Errorf("DigestBytes() length = %v, want %v", len(got), tt.wantLen)
			}
			if !bytes.Equal(got, tt.wantBytes[:tt.wantLen]) {
				t.Errorf("DigestBytes() content mismatch")
			}
		})
	}
}

func TestDigestHeaderString(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	header, err := NewDigestHeader(DigestSHA256, digest)
	if err != nil {
		t.Fatalf("NewDigestHeader() error = %v", err)
	}
	str := header.String()

	if str == "" {
		t.Error("DigestHeader.String() should not be empty")
	}
}

func TestDetachedTimestamp(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	header, err := NewDigestHeader(DigestSHA256, digest)
	if err != nil {
		t.Fatalf("NewDigestHeader() error = %v", err)
	}
	ts := Timestamp{
		NewSHA256Step(Timestamp{
			NewAttestationStep(&BitcoinAttestation{Height: 123}),
		}),
	}

	dt := NewDetachedTimestamp(header, ts)

	if dt.Header != header {
		t.Error("DetachedTimestamp.Header not set correctly")
	}

	if len(dt.Timestamp) != 1 {
		t.Error("DetachedTimestamp.Timestamp not set correctly")
	}
}

func TestDetachedTimestampString(t *testing.T) {
	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	header, err := NewDigestHeader(DigestSHA256, digest)
	if err != nil {
		t.Fatalf("NewDigestHeader() error = %v", err)
	}
	ts := Timestamp{}
	dt := NewDetachedTimestamp(header, ts)

	str := dt.String()
	if str == "" {
		t.Error("DetachedTimestamp.String() should not be empty")
	}
}

func TestDigestOpConstants(t *testing.T) {
	if DigestSHA1 != DigestOp(0x02) {
		t.Errorf("DigestSHA1 = %v, want 0x02", DigestSHA1)
	}
	if DigestRIPEMD160 != DigestOp(0x03) {
		t.Errorf("DigestRIPEMD160 = %v, want 0x03", DigestRIPEMD160)
	}
	if DigestSHA256 != DigestOp(0x08) {
		t.Errorf("DigestSHA256 = %v, want 0x08", DigestSHA256)
	}
	if DigestKECCAK256 != DigestOp(0x67) {
		t.Errorf("DigestKECCAK256 = %v, want 0x67", DigestKECCAK256)
	}
}
