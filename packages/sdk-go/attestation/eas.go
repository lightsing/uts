package attestation

import (
	"context"
	"fmt"

	"github.com/uts-dot/sdk-go/rpc"
	"github.com/uts-dot/sdk-go/types"
)

type EASClient interface {
	GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error)
	GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error)
}

func VerifyEASAttestation(ctx context.Context, client EASClient, digest []byte, att *types.EASAttestation) *types.AttestationStatus {
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
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get attestation: %w", err),
		}
	}

	if easAtt.Schema != rpc.SchemaID {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("invalid schema: expected %x, got %x", rpc.SchemaID, easAtt.Schema),
		}
	}

	if easAtt.Revocable {
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
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("attested hash mismatch: expected %x, got %x", digestHash, attestedHash),
		}
	}

	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
	}
}

func VerifyEASTimestamped(ctx context.Context, client EASClient, digest []byte, att *types.EASTimestamped) *types.AttestationStatus {
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
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
			Error:       fmt.Errorf("failed to get timestamp: %w", err),
		}
	}

	if timestamp == 0 {
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusInvalid,
			Error:       fmt.Errorf("timestamp not found for digest %x", digestHash),
		}
	}

	return &types.AttestationStatus{
		Attestation: att,
		Status:      types.StatusValid,
		Info: map[string]interface{}{
			"timestamp": timestamp,
		},
	}
}
