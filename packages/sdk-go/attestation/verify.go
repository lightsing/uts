package attestation

import (
	"context"

	"github.com/lightsing/uts/packages/sdk-go/types"
)

func Verify(ctx context.Context, btcRPC BitcoinRPCClient, easClient EASClient, digest []byte, att types.Attestation) *types.AttestationStatus {
	switch a := att.(type) {
	case *types.PendingAttestation:
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusPending,
		}
	case *types.BitcoinAttestation:
		return VerifyBitcoin(ctx, btcRPC, digest, a)
	case *types.EASAttestation:
		return VerifyEASAttestation(ctx, easClient, digest, a)
	case *types.EASTimestamped:
		return VerifyEASTimestamped(ctx, easClient, digest, a)
	default:
		return &types.AttestationStatus{
			Attestation: att,
			Status:      types.StatusUnknown,
		}
	}
}
