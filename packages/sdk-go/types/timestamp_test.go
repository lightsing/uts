package types

import (
	"testing"
)

func TestOpString(t *testing.T) {
	tests := []struct {
		op   Op
		want string
	}{
		{OpAttestation, "ATTESTATION"},
		{OpSHA1, "SHA1"},
		{OpRIPEMD160, "RIPEMD160"},
		{OpSHA256, "SHA256"},
		{OpKECCAK256, "KECCAK256"},
		{OpAPPEND, "APPEND"},
		{OpPREPEND, "PREPEND"},
		{OpREVERSE, "REVERSE"},
		{OpHEXLIFY, "HEXLIFY"},
		{OpFORK, "FORK"},
		{Op(0x99), "UNKNOWN"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.op.String(); got != tt.want {
				t.Errorf("Op.String() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestOpHasImmediate(t *testing.T) {
	tests := []struct {
		op   Op
		want bool
	}{
		{OpAPPEND, true},
		{OpPREPEND, true},
		{OpSHA256, false},
		{OpAttestation, false},
		{OpFORK, false},
		{OpREVERSE, false},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.HasImmediate(); got != tt.want {
				t.Errorf("Op.HasImmediate() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestOpIsControl(t *testing.T) {
	tests := []struct {
		op   Op
		want bool
	}{
		{OpAttestation, true},
		{OpFORK, true},
		{OpSHA256, false},
		{OpAPPEND, false},
		{OpPREPEND, false},
		{OpREVERSE, false},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.IsControl(); got != tt.want {
				t.Errorf("Op.IsControl() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestOpIsDigest(t *testing.T) {
	tests := []struct {
		op   Op
		want bool
	}{
		{OpSHA1, true},
		{OpRIPEMD160, true},
		{OpSHA256, true},
		{OpKECCAK256, true},
		{OpAPPEND, false},
		{OpPREPEND, false},
		{OpAttestation, false},
		{OpFORK, false},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.IsDigest(); got != tt.want {
				t.Errorf("Op.IsDigest() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestOpValid(t *testing.T) {
	tests := []struct {
		op   Op
		want bool
	}{
		{OpAttestation, true},
		{OpSHA1, true},
		{OpRIPEMD160, true},
		{OpSHA256, true},
		{OpKECCAK256, true},
		{OpAPPEND, true},
		{OpPREPEND, true},
		{OpREVERSE, true},
		{OpHEXLIFY, true},
		{OpFORK, true},
		{Op(0x00), true},
		{Op(0x99), false},
		{Op(0xfe), false},
	}

	for _, tt := range tests {
		name := tt.op.String()
		t.Run(name, func(t *testing.T) {
			if got := tt.op.Valid(); got != tt.want {
				t.Errorf("Op.Valid() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestNewOp(t *testing.T) {
	tests := []struct {
		b      byte
		want   Op
		wantOk bool
	}{
		{0x00, OpAttestation, true},
		{0x02, OpSHA1, true},
		{0x08, OpSHA256, true},
		{0x67, OpKECCAK256, true},
		{0xf0, OpAPPEND, true},
		{0xff, OpFORK, true},
		{0x99, Op(0x99), false},
	}

	for _, tt := range tests {
		name := string(rune(tt.b))
		t.Run(name, func(t *testing.T) {
			got, gotOk := NewOp(tt.b)
			if got != tt.want || gotOk != tt.wantOk {
				t.Errorf("NewOp() = (%v, %v), want (%v, %v)", got, gotOk, tt.want, tt.wantOk)
			}
		})
	}
}

func TestStepConstructors(t *testing.T) {
	dummyTS := Timestamp{}

	t.Run("AppendStep", func(t *testing.T) {
		data := []byte("test")
		step := NewAppendStep(data, dummyTS)
		if step.Op() != OpAPPEND {
			t.Errorf("AppendStep.Op() = %v, want %v", step.Op(), OpAPPEND)
		}
		if string(step.Data) != string(data) {
			t.Errorf("AppendStep.Data = %v, want %v", step.Data, data)
		}
	})

	t.Run("PrependStep", func(t *testing.T) {
		data := []byte("test")
		step := NewPrependStep(data, dummyTS)
		if step.Op() != OpPREPEND {
			t.Errorf("PrependStep.Op() = %v, want %v", step.Op(), OpPREPEND)
		}
		if string(step.Data) != string(data) {
			t.Errorf("PrependStep.Data = %v, want %v", step.Data, data)
		}
	})

	t.Run("ReverseStep", func(t *testing.T) {
		step := NewReverseStep(dummyTS)
		if step.Op() != OpREVERSE {
			t.Errorf("ReverseStep.Op() = %v, want %v", step.Op(), OpREVERSE)
		}
	})

	t.Run("HexlifyStep", func(t *testing.T) {
		step := NewHexlifyStep(dummyTS)
		if step.Op() != OpHEXLIFY {
			t.Errorf("HexlifyStep.Op() = %v, want %v", step.Op(), OpHEXLIFY)
		}
	})

	t.Run("SHA256Step", func(t *testing.T) {
		step := NewSHA256Step(dummyTS)
		if step.Op() != OpSHA256 {
			t.Errorf("SHA256Step.Op() = %v, want %v", step.Op(), OpSHA256)
		}
	})

	t.Run("Keccak256Step", func(t *testing.T) {
		step := NewKeccak256Step(dummyTS)
		if step.Op() != OpKECCAK256 {
			t.Errorf("Keccak256Step.Op() = %v, want %v", step.Op(), OpKECCAK256)
		}
	})

	t.Run("SHA1Step", func(t *testing.T) {
		step := NewSHA1Step(dummyTS)
		if step.Op() != OpSHA1 {
			t.Errorf("SHA1Step.Op() = %v, want %v", step.Op(), OpSHA1)
		}
	})

	t.Run("RIPEMD160Step", func(t *testing.T) {
		step := NewRIPEMD160Step(dummyTS)
		if step.Op() != OpRIPEMD160 {
			t.Errorf("RIPEMD160Step.Op() = %v, want %v", step.Op(), OpRIPEMD160)
		}
	})

	t.Run("ForkStep", func(t *testing.T) {
		branches := []Timestamp{dummyTS, dummyTS}
		step := NewForkStep(branches)
		if step.Op() != OpFORK {
			t.Errorf("ForkStep.Op() = %v, want %v", step.Op(), OpFORK)
		}
		if len(step.Branches) != 2 {
			t.Errorf("ForkStep.Branches length = %v, want 2", len(step.Branches))
		}
	})

	t.Run("AttestationStep", func(t *testing.T) {
		att := &BitcoinAttestation{Height: 123}
		step := NewAttestationStep(att)
		if step.Op() != OpAttestation {
			t.Errorf("AttestationStep.Op() = %v, want %v", step.Op(), OpAttestation)
		}
	})
}

func TestStepInterface(t *testing.T) {
	dummyTS := Timestamp{}
	var _ Step = NewAppendStep([]byte("test"), dummyTS)
	var _ Step = NewPrependStep([]byte("test"), dummyTS)
	var _ Step = NewReverseStep(dummyTS)
	var _ Step = NewHexlifyStep(dummyTS)
	var _ Step = NewSHA256Step(dummyTS)
	var _ Step = NewKeccak256Step(dummyTS)
	var _ Step = NewSHA1Step(dummyTS)
	var _ Step = NewRIPEMD160Step(dummyTS)
	var _ Step = NewForkStep(nil)
	var _ Step = NewAttestationStep(&BitcoinAttestation{})
}
