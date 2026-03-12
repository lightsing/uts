package types

import (
	"testing"
)

func TestAttestationKindString(t *testing.T) {
	tests := []struct {
		kind AttestationKind
		want string
	}{
		{KindBitcoin, "bitcoin"},
		{KindPending, "pending"},
		{KindEASAttestation, "eas-attestation"},
		{KindEASTimestamped, "eas-timestamped"},
		{KindUnknown, "unknown"},
		{AttestationKind(99), "unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.kind.String(); got != tt.want {
				t.Errorf("AttestationKind.String() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestBitcoinAttestation(t *testing.T) {
	att := &BitcoinAttestation{Height: 800000}

	if att.Kind() != KindBitcoin {
		t.Errorf("BitcoinAttestation.Kind() = %v, want %v", att.Kind(), KindBitcoin)
	}

	if att.Tag() != BitcoinTag {
		t.Errorf("BitcoinAttestation.Tag() = %v, want %v", att.Tag(), BitcoinTag)
	}

	expectedStr := "Bitcoin at height 800000"
	if att.String() != expectedStr {
		t.Errorf("BitcoinAttestation.String() = %v, want %v", att.String(), expectedStr)
	}

	var _ Attestation = att
}

func TestEASAttestation(t *testing.T) {
	var uid [32]byte
	for i := range uid {
		uid[i] = byte(i)
	}

	att := &EASAttestation{
		ChainID: 1,
		UID:     uid,
	}

	if att.Kind() != KindEASAttestation {
		t.Errorf("EASAttestation.Kind() = %v, want %v", att.Kind(), KindEASAttestation)
	}

	if att.Tag() != EASAttestTag {
		t.Errorf("EASAttestation.Tag() = %v, want %v", att.Tag(), EASAttestTag)
	}

	var _ Attestation = att
}

func TestEASTimestamped(t *testing.T) {
	att := &EASTimestamped{ChainID: 534352}

	if att.Kind() != KindEASTimestamped {
		t.Errorf("EASTimestamped.Kind() = %v, want %v", att.Kind(), KindEASTimestamped)
	}

	if att.Tag() != EASTimestampTag {
		t.Errorf("EASTimestamped.Tag() = %v, want %v", att.Tag(), EASTimestampTag)
	}

	expectedStr := "EAS timestamped on chain 534352"
	if att.String() != expectedStr {
		t.Errorf("EASTimestamped.String() = %v, want %v", att.String(), expectedStr)
	}

	var _ Attestation = att
}

func TestPendingAttestation(t *testing.T) {
	att := &PendingAttestation{URI: "https://example.com/calendar"}

	if att.Kind() != KindPending {
		t.Errorf("PendingAttestation.Kind() = %v, want %v", att.Kind(), KindPending)
	}

	if att.Tag() != PendingTag {
		t.Errorf("PendingAttestation.Tag() = %v, want %v", att.Tag(), PendingTag)
	}

	expectedStr := "Pending at https://example.com/calendar"
	if att.String() != expectedStr {
		t.Errorf("PendingAttestation.String() = %v, want %v", att.String(), expectedStr)
	}

	var _ Attestation = att
}

func TestValidateURI(t *testing.T) {
	maxURIStr := make([]byte, MaxURILen)
	for i := range maxURIStr {
		maxURIStr[i] = 'a'
	}

	tests := []struct {
		name string
		uri  string
		want bool
	}{
		{"valid simple", "https://example.com", true},
		{"valid with path", "https://example.com/calendar", true},
		{"valid with port", "https://example.com:8080/path", true},
		{"alphanumeric", "abc123DEF", true},
		{"with special chars", "a-b_c.d/e:f", true},
		{"empty string", "", true},
		{"space invalid", "has space", false},
		{"at sign invalid", "user@host", false},
		{"percent invalid", "percent%20encoded", false},
		{"unicode invalid", "unicode\u0000null", false},
		{"too long", string(make([]byte, MaxURILen+1)), false},
		{"max length", string(maxURIStr), true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := ValidateURI(tt.uri); got != tt.want {
				t.Errorf("ValidateURI(%q) = %v, want %v", tt.uri, got, tt.want)
			}
		})
	}
}

func TestPendingAttestationValid(t *testing.T) {
	tests := []struct {
		name string
		uri  string
		want bool
	}{
		{"valid URI", "https://example.com", true},
		{"invalid URI", "has space", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			att := &PendingAttestation{URI: tt.uri}
			if got := att.Valid(); got != tt.want {
				t.Errorf("PendingAttestation.Valid() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestUnknownAttestation(t *testing.T) {
	tag := [TagSize]byte{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08}
	data := []byte("somedata")

	att := NewUnknownAttestation(tag, data)

	if att.Kind() != KindUnknown {
		t.Errorf("UnknownAttestation.Kind() = %v, want %v", att.Kind(), KindUnknown)
	}

	if att.Tag() != tag {
		t.Errorf("UnknownAttestation.Tag() = %v, want %v", att.Tag(), tag)
	}

	if string(att.Data) != string(data) {
		t.Errorf("UnknownAttestation.Data = %v, want %v", att.Data, data)
	}
}

func TestAttestationKindFromTag(t *testing.T) {
	tests := []struct {
		name string
		tag  [TagSize]byte
		want AttestationKind
	}{
		{"bitcoin", BitcoinTag, KindBitcoin},
		{"pending", PendingTag, KindPending},
		{"eas attestation", EASAttestTag, KindEASAttestation},
		{"eas timestamped", EASTimestampTag, KindEASTimestamped},
		{"unknown", [TagSize]byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}, KindUnknown},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := AttestationKindFromTag(tt.tag); got != tt.want {
				t.Errorf("AttestationKindFromTag() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParseAttestationKind(t *testing.T) {
	tests := []struct {
		input  string
		want   AttestationKind
		wantOk bool
	}{
		{"bitcoin", KindBitcoin, true},
		{"BITCOIN", KindBitcoin, true},
		{"pending", KindPending, true},
		{"Pending", KindPending, true},
		{"eas-attestation", KindEASAttestation, true},
		{"EAS-ATTESTATION", KindEASAttestation, true},
		{"eas-timestamped", KindEASTimestamped, true},
		{"unknown", KindUnknown, true},
		{"invalid", KindUnknown, false},
		{"", KindUnknown, false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			got, gotOk := ParseAttestationKind(tt.input)
			if got != tt.want || gotOk != tt.wantOk {
				t.Errorf("ParseAttestationKind(%q) = (%v, %v), want (%v, %v)", tt.input, got, gotOk, tt.want, tt.wantOk)
			}
		})
	}
}

func TestTagConstants(t *testing.T) {
	if BitcoinTag == [TagSize]byte{} {
		t.Error("BitcoinTag should not be zero value")
	}
	if PendingTag == [TagSize]byte{} {
		t.Error("PendingTag should not be zero value")
	}
	if EASAttestTag == [TagSize]byte{} {
		t.Error("EASAttestTag should not be zero value")
	}
	if EASTimestampTag == [TagSize]byte{} {
		t.Error("EASTimestampTag should not be zero value")
	}

	if BitcoinTag == PendingTag {
		t.Error("BitcoinTag and PendingTag should be different")
	}
	if EASAttestTag == EASTimestampTag {
		t.Error("EASAttestTag and EASTimestampTag should be different")
	}
}
