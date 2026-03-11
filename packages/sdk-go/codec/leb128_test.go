package codec

import (
	"bytes"
	"testing"
)

func TestEncodeU32(t *testing.T) {
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
		{"16383", 16383, []byte{0xff, 0x7f}},
		{"16384", 16384, []byte{0x80, 0x80, 0x01}},
		{"max uint32", 0xffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0x0f}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := EncodeU32(tt.input)
			if !bytes.Equal(got, tt.want) {
				t.Errorf("EncodeU32(%d) = %v, want %v", tt.input, got, tt.want)
			}
		})
	}
}

func TestDecodeU32(t *testing.T) {
	tests := []struct {
		name  string
		input []byte
		want  uint32
	}{
		{"zero", []byte{0x00}, 0},
		{"one", []byte{0x01}, 1},
		{"127", []byte{0x7f}, 127},
		{"128", []byte{0x80, 0x01}, 128},
		{"300", []byte{0xac, 0x02}, 300},
		{"16383", []byte{0xff, 0x7f}, 16383},
		{"16384", []byte{0x80, 0x80, 0x01}, 16384},
		{"max uint32", []byte{0xff, 0xff, 0xff, 0xff, 0x0f}, 0xffffffff},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := DecodeU32(bytes.NewReader(tt.input))
			if err != nil {
				t.Errorf("DecodeU32(%v) error: %v", tt.input, err)
				return
			}
			if got != tt.want {
				t.Errorf("DecodeU32(%v) = %d, want %d", tt.input, got, tt.want)
			}
		})
	}
}

func TestEncodeDecodeU32RoundTrip(t *testing.T) {
	values := []uint32{
		0,
		1,
		127,
		128,
		300,
		16383,
		16384,
		624352,
		0x00ffffff,
		0xffffffff,
	}

	for _, v := range values {
		t.Run("", func(t *testing.T) {
			encoded := EncodeU32(v)
			decoded, err := DecodeU32(bytes.NewReader(encoded))
			if err != nil {
				t.Errorf("DecodeU32 error: %v", err)
				return
			}
			if decoded != v {
				t.Errorf("round trip: got %d, want %d", decoded, v)
			}
		})
	}
}

func TestEncodeU64(t *testing.T) {
	tests := []struct {
		name  string
		input uint64
		want  []byte
	}{
		{"zero", 0, []byte{0x00}},
		{"one", 1, []byte{0x01}},
		{"127", 127, []byte{0x7f}},
		{"128", 128, []byte{0x80, 0x01}},
		{"300", 300, []byte{0xac, 0x02}},
		{"16383", 16383, []byte{0xff, 0x7f}},
		{"16384", 16384, []byte{0x80, 0x80, 0x01}},
		{"max uint32", 0xffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0x0f}},
		{"large", 0x0fffffffffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f}},
		{"max uint64", 0xffffffffffffffff, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := EncodeU64(tt.input)
			if !bytes.Equal(got, tt.want) {
				t.Errorf("EncodeU64(%d) = %v, want %v", tt.input, got, tt.want)
			}
		})
	}
}

func TestDecodeU64(t *testing.T) {
	tests := []struct {
		name  string
		input []byte
		want  uint64
	}{
		{"zero", []byte{0x00}, 0},
		{"one", []byte{0x01}, 1},
		{"127", []byte{0x7f}, 127},
		{"128", []byte{0x80, 0x01}, 128},
		{"300", []byte{0xac, 0x02}, 300},
		{"16383", []byte{0xff, 0x7f}, 16383},
		{"16384", []byte{0x80, 0x80, 0x01}, 16384},
		{"max uint32", []byte{0xff, 0xff, 0xff, 0xff, 0x0f}, 0xffffffff},
		{"large", []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f}, 0x0fffffffffffffff},
		{"max uint64", []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01}, 0xffffffffffffffff},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := DecodeU64(bytes.NewReader(tt.input))
			if err != nil {
				t.Errorf("DecodeU64(%v) error: %v", tt.input, err)
				return
			}
			if got != tt.want {
				t.Errorf("DecodeU64(%v) = %d, want %d", tt.input, got, tt.want)
			}
		})
	}
}

func TestEncodeDecodeU64RoundTrip(t *testing.T) {
	values := []uint64{
		0,
		1,
		127,
		128,
		300,
		16383,
		16384,
		0xffffffff,
		0x0fffffffffffffff,
		0xffffffffffffffff,
	}

	for _, v := range values {
		t.Run("", func(t *testing.T) {
			encoded := EncodeU64(v)
			decoded, err := DecodeU64(bytes.NewReader(encoded))
			if err != nil {
				t.Errorf("DecodeU64 error: %v", err)
				return
			}
			if decoded != v {
				t.Errorf("round trip: got %d, want %d", decoded, v)
			}
		})
	}
}

func TestDecodeU32Overflow(t *testing.T) {
	input := []byte{0xff, 0xff, 0xff, 0xff, 0x1f}
	_, err := DecodeU32(bytes.NewReader(input))
	if err == nil {
		t.Error("expected overflow error, got nil")
	}
}

func TestDecodeU64Overflow(t *testing.T) {
	input := []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f}
	_, err := DecodeU64(bytes.NewReader(input))
	if err == nil {
		t.Error("expected overflow error, got nil")
	}
}

func TestDecodeU32UnexpectedEOF(t *testing.T) {
	input := []byte{0x80}
	_, err := DecodeU32(bytes.NewReader(input))
	if err == nil {
		t.Error("expected error for truncated input, got nil")
	}
}

func TestDecodeU64UnexpectedEOF(t *testing.T) {
	input := []byte{0x80}
	_, err := DecodeU64(bytes.NewReader(input))
	if err == nil {
		t.Error("expected error for truncated input, got nil")
	}
}
