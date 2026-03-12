package attestation

import (
	"context"
	"encoding/hex"
	"fmt"

	"github.com/lightsing/uts/packages/sdk-go/logging"
	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type EASClient interface {
	GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error)
	GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error)
}

func VerifyEASAttestation(ctx context.Context, client EASClient, digest []byte, att *types.EASAttestation) *types.AttestationStatus {
	logger := logging.Default()
	logger.Debug(ctx, "VerifyEASAttestation: verifying", "chain_id", att.ChainID, "uid", hex.EncodeToString(att.UID[:]))

	if len(digest) != 32 {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("invalid digest length: expected 32, got %d", len(digest)),
		}
	}

	var digestHash [32]byte
	copy(digestHash[:], digest)

	easAtt, err := client.GetEASAttestation(ctx, att.ChainID, att.UID)
	if err != nil {
		logger.Warn(ctx, "VerifyEASAttestation: failed to get attestation", "chain_id", att.ChainID, "error", err)
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get attestation: %w", err),
		}
	}

	if easAtt.Schema != rpc.SchemaID {
		logger.Debug(ctx, "VerifyEASAttestation: invalid schema", "expected", hex.EncodeToString(rpc.SchemaID[:]), "got", hex.EncodeToString(easAtt.Schema[:]))
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("invalid schema: expected %x, got %x", rpc.SchemaID, easAtt.Schema),
		}
	}

	if easAtt.Revocable {
		logger.Debug(ctx, "VerifyEASAttestation: invalid - attestation is revocable")
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("attestation cannot be revocable"),
		}
	}

	if len(easAtt.Data) != 32 {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("invalid attestation data length: expected 32, got %d", len(easAtt.Data)),
		}
	}

	var attestedHash [32]byte
	copy(attestedHash[:], easAtt.Data)

	if attestedHash != digestHash {
		logger.Debug(ctx, "VerifyEASAttestation: invalid - hash mismatch")
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("attested hash mismatch: expected %x, got %x", digestHash, attestedHash),
		}
	}

	logger.Debug(ctx, "VerifyEASAttestation: valid", "chain_id", att.ChainID, "time", easAtt.Time)
	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
	}
}

func VerifyEASTimestamped(ctx context.Context, client EASClient, digest []byte, att *types.EASTimestamped) *types.AttestationStatus {
	logger := logging.Default()
	logger.Debug(ctx, "VerifyEASTimestamped: verifying", "chain_id", att.ChainID, "digest", hex.EncodeToString(digest))

	if len(digest) != 32 {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("invalid digest length: expected 32, got %d", len(digest)),
		}
	}

	var digestHash [32]byte
	copy(digestHash[:], digest)

	timestamp, err := client.GetTimestamp(ctx, att.ChainID, digestHash)
	if err != nil {
		logger.Warn(ctx, "VerifyEASTimestamped: failed to get timestamp", "chain_id", att.ChainID, "error", err)
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get timestamp: %w", err),
		}
	}

	if timestamp == 0 {
		logger.Debug(ctx, "VerifyEASTimestamped: invalid - timestamp not found")
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("timestamp not found for digest %x", digestHash),
		}
	}

	logger.Debug(ctx, "VerifyEASTimestamped: valid", "chain_id", att.ChainID, "timestamp", timestamp)
	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
		Info: map[string]interface{}{
			"timestamp": timestamp,
		},
	}
}
