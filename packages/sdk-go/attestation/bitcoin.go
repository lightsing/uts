package attestation

import (
	"context"
	"encoding/hex"
	"fmt"

	"github.com/lightsing/uts/packages/sdk-go/logging"
	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type BitcoinRPCClient interface {
	GetBlockHash(height int64) (string, error)
	GetBlockHeader(hash string) (*rpc.BlockHeader, error)
}

func VerifyBitcoin(ctx context.Context, client BitcoinRPCClient, digest []byte, att *types.BitcoinAttestation) *types.AttestationStatus {
	logger := logging.Default()
	logger.Debug(ctx, "VerifyBitcoin: verifying", "height", att.Height, "digest", hex.EncodeToString(digest))

	blockHash, err := client.GetBlockHash(int64(att.Height))
	if err != nil {
		logger.Warn(ctx, "VerifyBitcoin: failed to get block hash", "height", att.Height, "error", err)
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get block hash at height %d: %w", att.Height, err),
		}
	}

	header, err := client.GetBlockHeader(blockHash)
	if err != nil {
		logger.Warn(ctx, "VerifyBitcoin: failed to get block header", "hash", blockHash, "error", err)
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
			logger.Debug(ctx, "VerifyBitcoin: invalid - merkle root mismatch", "height", att.Height)
			return &types.AttestationStatus{
				Attestation: att,
				Status:      types.StatusInvalid,
				Error:       fmt.Errorf("bitcoin attestation does not match the expected merkle root at height %d", att.Height),
			}
		}
	}

	logger.Debug(ctx, "VerifyBitcoin: valid", "height", att.Height)
	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
	}
}
