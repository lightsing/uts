package crypto

import (
	"bytes"
	"encoding/hex"
	"testing"
)

func TestSHA256(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "empty string",
			input:    "",
			expected: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
		},
		{
			name:     "hello world",
			input:    "hello world",
			expected: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
		},
		{
			name:     "abc",
			input:    "abc",
			expected: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
		},
		{
			name:     "long input",
			input:    "The quick brown fox jumps over the lazy dog",
			expected: "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := []byte(tt.input)
			hash := SHA256(input)

			if len(hash) != HashSize {
				t.Errorf("SHA256: expected hash length %d, got %d", HashSize, len(hash))
			}

			expectedBytes, err := hex.DecodeString(tt.expected)
			if err != nil {
				t.Fatalf("failed to decode expected hex: %v", err)
			}

			if !bytes.Equal(hash[:], expectedBytes) {
				t.Errorf("SHA256: expected %x, got %x", expectedBytes, hash)
			}
		})
	}
}

func TestKeccak256(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "empty string",
			input:    "",
			expected: "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
		},
		{
			name:     "hello world",
			input:    "hello world",
			expected: "47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad",
		},
		{
			name:     "abc",
			input:    "abc",
			expected: "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := []byte(tt.input)
			hash := Keccak256(input)

			if len(hash) != HashSize {
				t.Errorf("Keccak256: expected hash length %d, got %d", HashSize, len(hash))
			}

			expectedBytes, err := hex.DecodeString(tt.expected)
			if err != nil {
				t.Fatalf("failed to decode expected hex: %v", err)
			}

			if !bytes.Equal(hash[:], expectedBytes) {
				t.Errorf("Keccak256: expected %x, got %x", expectedBytes, hash)
			}
		})
	}
}

func TestSHA256Hash(t *testing.T) {
	input := []byte("test")
	hash := SHA256Hash(input)

	if len(hash) != HashSize {
		t.Errorf("SHA256Hash: expected length %d, got %d", HashSize, len(hash))
	}

	expected := SHA256(input)
	if !bytes.Equal(hash, expected[:]) {
		t.Errorf("SHA256Hash: result doesn't match SHA256")
	}
}

func TestKeccak256Hash(t *testing.T) {
	input := []byte("test")
	hash := Keccak256Hash(input)

	if len(hash) != HashSize {
		t.Errorf("Keccak256Hash: expected length %d, got %d", HashSize, len(hash))
	}

	expected := Keccak256(input)
	if !bytes.Equal(hash, expected[:]) {
		t.Errorf("Keccak256Hash: result doesn't match Keccak256")
	}
}

func TestHashConsistency(t *testing.T) {
	data := []byte("consistency test")

	hash1 := SHA256(data)
	hash2 := SHA256(data)
	if !bytes.Equal(hash1[:], hash2[:]) {
		t.Error("SHA256: same input should produce same hash")
	}

	khash1 := Keccak256(data)
	khash2 := Keccak256(data)
	if !bytes.Equal(khash1[:], khash2[:]) {
		t.Error("Keccak256: same input should produce same hash")
	}
}

func TestDifferentHashes(t *testing.T) {
	data := []byte("test data")

	shaHash := SHA256(data)
	keccakHash := Keccak256(data)

	if bytes.Equal(shaHash[:], keccakHash[:]) {
		t.Error("SHA256 and Keccak256 should produce different results")
	}
}
