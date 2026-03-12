package crypto

import (
	"crypto/sha256"

	"github.com/ethereum/go-ethereum/crypto"
)

const (
	HashSize = 32
)

func SHA256(data []byte) [HashSize]byte {
	var hash [HashSize]byte
	sum := sha256.Sum256(data)
	copy(hash[:], sum[:])
	return hash
}

func Keccak256(data []byte) [HashSize]byte {
	var hash [HashSize]byte
	sum := crypto.Keccak256(data)
	copy(hash[:], sum)
	return hash
}

func SHA256Hash(data []byte) []byte {
	hash := SHA256(data)
	return hash[:]
}

func Keccak256Hash(data []byte) []byte {
	hash := Keccak256(data)
	return hash[:]
}
