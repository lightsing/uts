package attestation

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"testing"

	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type mockBitcoinRPC struct {
	blockHash string
	header    *rpc.BlockHeader
	hashErr   error
	headerErr error
}

func (m *mockBitcoinRPC) GetBlockHash(height int64) (string, error) {
	if m.hashErr != nil {
		return "", m.hashErr
	}
	return m.blockHash, nil
}

func (m *mockBitcoinRPC) GetBlockHeader(hash string) (*rpc.BlockHeader, error) {
	if m.headerErr != nil {
		return nil, m.headerErr
	}
	return m.header, nil
}

func TestVerifyBitcoin_Success(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))
	merkleRoot := rpc.ReverseBytes(digest[:])
	merkleRootHex := hex.EncodeToString(merkleRoot)

	mockRPC := &mockBitcoinRPC{
		blockHash: "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
		header: &rpc.BlockHeader{
			Hash:       "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
			MerkleRoot: merkleRootHex,
			Height:     800000,
			Time:       1234567890,
		},
	}

	att := &types.BitcoinAttestation{Height: 800000}
	status := VerifyBitcoin(context.Background(), mockRPC, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
	if status.Attestation == nil {
		t.Error("expected attestation to be set")
	}
}

func TestVerifyBitcoin_MismatchedMerkleRoot(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))
	wrongDigest := sha256.Sum256([]byte("wrong data"))
	merkleRoot := rpc.ReverseBytes(wrongDigest[:])
	merkleRootHex := hex.EncodeToString(merkleRoot)

	mockRPC := &mockBitcoinRPC{
		blockHash: "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
		header: &rpc.BlockHeader{
			Hash:       "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
			MerkleRoot: merkleRootHex,
			Height:     800000,
			Time:       1234567890,
		},
	}

	att := &types.BitcoinAttestation{Height: 800000}
	status := VerifyBitcoin(context.Background(), mockRPC, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for mismatched merkle root")
	}
}

func TestVerifyBitcoin_GetBlockHashError(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))

	mockRPC := &mockBitcoinRPC{
		hashErr: errors.New("block not found"),
	}

	att := &types.BitcoinAttestation{Height: 800000}
	status := VerifyBitcoin(context.Background(), mockRPC, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerifyBitcoin_GetBlockHeaderError(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))

	mockRPC := &mockBitcoinRPC{
		blockHash: "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
		headerErr: errors.New("header not found"),
	}

	att := &types.BitcoinAttestation{Height: 800000}
	status := VerifyBitcoin(context.Background(), mockRPC, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}
