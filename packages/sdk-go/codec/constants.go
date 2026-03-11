package codec

import "github.com/lightsing/uts/packages/sdk-go/types"

const (
	OpSHA1        byte = 0x02
	OpRIPEMD160   byte = 0x03
	OpSHA256      byte = 0x08
	OpKECCAK256   byte = 0x67
	OpAPPEND      byte = 0xf0
	OpPREPEND     byte = 0xf1
	OpREVERSE     byte = 0xf2
	OpHEXLIFY     byte = 0xf3
	OpATTESTATION byte = 0x00
	OpFORK        byte = 0xff
)

var OpCodeMap = map[string]byte{
	"SHA1":        OpSHA1,
	"RIPEMD160":   OpRIPEMD160,
	"SHA256":      OpSHA256,
	"KECCAK256":   OpKECCAK256,
	"APPEND":      OpAPPEND,
	"PREPEND":     OpPREPEND,
	"REVERSE":     OpREVERSE,
	"HEXLIFY":     OpHEXLIFY,
	"ATTESTATION": OpATTESTATION,
	"FORK":        OpFORK,
}

var OpCodeName = map[byte]string{
	OpSHA1:        "SHA1",
	OpRIPEMD160:   "RIPEMD160",
	OpSHA256:      "SHA256",
	OpKECCAK256:   "KECCAK256",
	OpAPPEND:      "APPEND",
	OpPREPEND:     "PREPEND",
	OpREVERSE:     "REVERSE",
	OpHEXLIFY:     "HEXLIFY",
	OpATTESTATION: "ATTESTATION",
	OpFORK:        "FORK",
}

var DigestLengths = map[string]int{
	"SHA1":      20,
	"RIPEMD160": 20,
	"SHA256":    32,
	"KECCAK256": 32,
}

var MagicBytes = []byte{
	0x00, 0x4f, 0x70, 0x65, 0x6e, 0x54, 0x69, 0x6d,
	0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x73, 0x00,
	0x00, 0x50, 0x72, 0x6f, 0x6f, 0x66, 0x00, 0xbf,
	0x89, 0xe2, 0xe8, 0x84, 0xe8, 0x92, 0x94,
}

var (
	BitcoinAttestationTag = types.BitcoinTag
	PendingAttestationTag = types.PendingTag
	EASAttestTag          = types.EASAttestTag
	EASTimestampTag       = types.EASTimestampTag
)

var SchemaID = [32]byte{
	0x5c, 0x5b, 0x8b, 0x29, 0x5f, 0xf4, 0x3c, 0x8e,
	0x44, 0x2b, 0xe1, 0x1d, 0x56, 0x9e, 0x94, 0xa4,
	0xcd, 0x54, 0x76, 0xf5, 0xe2, 0x3d, 0xf0, 0xf7,
	0x1b, 0xdd, 0x40, 0x8d, 0xf6, 0xb9, 0x64, 0x9c,
}

const (
	MaxURILen    = 1000
	NoExpiration = uint64(0)
	TagSize      = 8
)

func GetOpCode(name string) (byte, bool) {
	code, ok := OpCodeMap[name]
	return code, ok
}

func GetOpName(code byte) (string, bool) {
	name, ok := OpCodeName[code]
	return name, ok
}

func GetDigestLength(name string) (int, bool) {
	length, ok := DigestLengths[name]
	return length, ok
}

func IsValidOpCode(code byte) bool {
	_, ok := OpCodeName[code]
	return ok
}

func IsDigestOp(code byte) bool {
	return code == OpSHA1 || code == OpRIPEMD160 || code == OpSHA256 || code == OpKECCAK256
}

func IsControlOp(code byte) bool {
	return code == OpATTESTATION || code == OpFORK
}

func HasImmediate(code byte) bool {
	return code == OpAPPEND || code == OpPREPEND
}
