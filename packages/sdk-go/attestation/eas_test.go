package attestation

import (
	"context"
	"errors"
	"testing"

	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type mockEASClient struct {
	attestation *rpc.Attestation
	timestamp   uint64
	attErr      error
	tsErr       error
}

func (m *mockEASClient) GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error) {
	if m.attErr != nil {
		return nil, m.attErr
	}
	return m.attestation, nil
}

func (m *mockEASClient) GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error) {
	if m.tsErr != nil {
		return 0, m.tsErr
	}
	return m.timestamp, nil
}

func TestVerifyEASAttestation_Valid(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{
		attestation: &rpc.Attestation{
			UID:       attUID,
			Schema:    rpc.SchemaID,
			Revocable: false,
			Data:      digest[:],
		},
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
}

func TestVerifyEASAttestation_InvalidSchema(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}
	wrongSchema := [32]byte{0xff, 0xff, 0xff, 0xff}

	mockClient := &mockEASClient{
		attestation: &rpc.Attestation{
			UID:       attUID,
			Schema:    wrongSchema,
			Revocable: false,
			Data:      digest[:],
		},
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for invalid schema")
	}
}

func TestVerifyEASAttestation_Revocable(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{
		attestation: &rpc.Attestation{
			UID:       attUID,
			Schema:    rpc.SchemaID,
			Revocable: true,
			Data:      digest[:],
		},
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for revocable attestation")
	}
}

func TestVerifyEASAttestation_MismatchedHash(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	wrongDigest := [32]byte{0xff, 0xfe, 0xfd, 0xfc}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{
		attestation: &rpc.Attestation{
			UID:       attUID,
			Schema:    rpc.SchemaID,
			Revocable: false,
			Data:      wrongDigest[:],
		},
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for mismatched hash")
	}
}

func TestVerifyEASAttestation_RPCError(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{
		attErr: errors.New("RPC error"),
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerifyEASTimestamped_Valid(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}

	mockClient := &mockEASClient{
		timestamp: 1234567890,
	}

	att := &types.EASTimestamped{
		ChainID: 534352,
	}

	status := VerifyEASTimestamped(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
	if status.Info == nil {
		t.Error("expected info to be set")
	}
	if ts, ok := status.Info["timestamp"].(uint64); !ok || ts != 1234567890 {
		t.Errorf("expected timestamp 1234567890, got %v", status.Info["timestamp"])
	}
}

func TestVerifyEASTimestamped_NotFound(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}

	mockClient := &mockEASClient{
		timestamp: 0,
	}

	att := &types.EASTimestamped{
		ChainID: 534352,
	}

	status := VerifyEASTimestamped(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for timestamp not found")
	}
}

func TestVerifyEASTimestamped_RPCError(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}

	mockClient := &mockEASClient{
		tsErr: errors.New("RPC error"),
	}

	att := &types.EASTimestamped{
		ChainID: 534352,
	}

	status := VerifyEASTimestamped(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerifyEASAttestation_InvalidDigestLength(t *testing.T) {
	digest := []byte{0x01, 0x02, 0x03}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest, att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for invalid digest length")
	}
}

func TestVerifyEASTimestamped_InvalidDigestLength(t *testing.T) {
	digest := []byte{0x01, 0x02, 0x03}

	mockClient := &mockEASClient{}

	att := &types.EASTimestamped{
		ChainID: 534352,
	}

	status := VerifyEASTimestamped(context.Background(), mockClient, digest, att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for invalid digest length")
	}
}

func TestVerifyEASAttestation_InvalidDataLength(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	mockClient := &mockEASClient{
		attestation: &rpc.Attestation{
			UID:       attUID,
			Schema:    rpc.SchemaID,
			Revocable: false,
			Data:      []byte{0x01, 0x02},
		},
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}

	status := VerifyEASAttestation(context.Background(), mockClient, digest[:], att)

	if status.Status != types.StatusInvalid {
		t.Errorf("expected status %v, got %v", types.StatusInvalid, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for invalid data length")
	}
}
