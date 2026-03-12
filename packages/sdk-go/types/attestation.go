package types

import (
	"encoding/hex"
	"fmt"
	"strings"
)

const TagSize = 8

var (
	BitcoinTag      = [TagSize]byte{0x05, 0x88, 0x96, 0x0d, 0x73, 0xd7, 0x19, 0x01}
	PendingTag      = [TagSize]byte{0x83, 0xdf, 0xe3, 0x0d, 0x2e, 0xf9, 0x0c, 0x8e}
	EASAttestTag    = [TagSize]byte{0x8b, 0xf4, 0x6b, 0xf4, 0xcf, 0xd6, 0x74, 0xfa}
	EASTimestampTag = [TagSize]byte{0x5a, 0xaf, 0xce, 0xeb, 0x1c, 0x7a, 0xd5, 0x8e}
)

type AttestationKind int

const (
	KindUnknown AttestationKind = iota
	KindBitcoin
	KindPending
	KindEASAttestation
	KindEASTimestamped
)

func (k AttestationKind) String() string {
	switch k {
	case KindBitcoin:
		return "bitcoin"
	case KindPending:
		return "pending"
	case KindEASAttestation:
		return "eas-attestation"
	case KindEASTimestamped:
		return "eas-timestamped"
	default:
		return "unknown"
	}
}

type BitcoinAttestation struct {
	Height uint32
}

func (a *BitcoinAttestation) Kind() AttestationKind { return KindBitcoin }
func (a *BitcoinAttestation) Tag() [TagSize]byte    { return BitcoinTag }

func (a *BitcoinAttestation) String() string {
	return fmt.Sprintf("Bitcoin at height %d", a.Height)
}

type EASAttestation struct {
	ChainID uint64
	UID     [32]byte
}

func (a *EASAttestation) Kind() AttestationKind { return KindEASAttestation }
func (a *EASAttestation) Tag() [TagSize]byte    { return EASAttestTag }

func (a *EASAttestation) String() string {
	return fmt.Sprintf("EAS attestation %s on chain %d", hex.EncodeToString(a.UID[:]), a.ChainID)
}

type EASTimestamped struct {
	ChainID uint64
}

func (a *EASTimestamped) Kind() AttestationKind { return KindEASTimestamped }
func (a *EASTimestamped) Tag() [TagSize]byte    { return EASTimestampTag }

func (a *EASTimestamped) String() string {
	return fmt.Sprintf("EAS timestamped on chain %d", a.ChainID)
}

type PendingAttestation struct {
	URI string
}

const MaxURILen = 1000

func ValidateURI(uri string) bool {
	if len(uri) > MaxURILen {
		return false
	}
	for _, ch := range uri {
		if !((ch >= 'a' && ch <= 'z') ||
			(ch >= 'A' && ch <= 'Z') ||
			(ch >= '0' && ch <= '9') ||
			ch == '.' || ch == '-' || ch == '_' || ch == '/' || ch == ':') {
			return false
		}
	}
	return true
}

func (a *PendingAttestation) Kind() AttestationKind { return KindPending }
func (a *PendingAttestation) Tag() [TagSize]byte    { return PendingTag }

func (a *PendingAttestation) String() string {
	return fmt.Sprintf("Pending at %s", a.URI)
}

func (a *PendingAttestation) Valid() bool {
	return ValidateURI(a.URI)
}

type UnknownAttestation struct {
	tag  [TagSize]byte
	Data []byte
}

func (a *UnknownAttestation) Kind() AttestationKind { return KindUnknown }
func (a *UnknownAttestation) Tag() [TagSize]byte    { return a.tag }

func NewUnknownAttestation(tag [TagSize]byte, data []byte) *UnknownAttestation {
	return &UnknownAttestation{tag: tag, Data: data}
}

func (a *UnknownAttestation) String() string {
	tag := a.Tag()
	return fmt.Sprintf("Unknown Attestation with tag %s", hex.EncodeToString(tag[:]))
}

func AttestationKindFromTag(tag [TagSize]byte) AttestationKind {
	switch tag {
	case BitcoinTag:
		return KindBitcoin
	case PendingTag:
		return KindPending
	case EASAttestTag:
		return KindEASAttestation
	case EASTimestampTag:
		return KindEASTimestamped
	default:
		return KindUnknown
	}
}

func ParseAttestationKind(s string) (AttestationKind, bool) {
	switch strings.ToLower(s) {
	case "bitcoin":
		return KindBitcoin, true
	case "pending":
		return KindPending, true
	case "eas-attestation":
		return KindEASAttestation, true
	case "eas-timestamped":
		return KindEASTimestamped, true
	case "unknown":
		return KindUnknown, true
	default:
		return KindUnknown, false
	}
}
