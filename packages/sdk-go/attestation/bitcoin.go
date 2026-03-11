package attestation

import (
	"context"
	"encoding/hex"
	"fmt"

	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type BitcoinRPCClient interface {
	GetBlockHash(height int64) (string, error)
	GetBlockHeader(hash string) (*rpc.BlockHeader, error)
}

func VerifyBitcoin(ctx context.Context, client BitcoinRPCClient, digest []byte, att *types.BitcoinAttestation) *types.AttestationStatus {
	blockHash, err := client.GetBlockHash(int64(att.Height))
	if err != nil {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get block hash at height %d: %w", att.Height, err),
		}
	}

	header, err := client.GetBlockHeader(blockHash)
	if err != nil {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get block header for hash %s: %w", blockHash, err),
		}
	}

	merkleRootBytes, err := hex.DecodeString(header.MerkleRoot)
	if err != nil {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to decode merkle root %s: %w", header.MerkleRoot, err),
		}
	}

	reversedMerkleRoot := rpc.ReverseBytes(merkleRootBytes)

	if len(reversedMerkleRoot) != len(digest) {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("merkle root length mismatch: expected %d, got %d", len(digest), len(reversedMerkleRoot)),
		}
	}

	for i := range digest {
		if digest[i] != reversedMerkleRoot[i] {
			return &types.AttestationStatus{
				Attestation: att,
				Status:      types.StatusInvalid,
				Error:       fmt.Errorf("bitcoin attestation does not match the expected merkle root at height %d", att.Height),
			}
		}
	}

	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
	}
}
