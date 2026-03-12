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

type mockVerifyClients struct {
	btcRPC *mockBitcoinRPC
	eas    *mockEASClient
}

func newMockVerifyClients() *mockVerifyClients {
	return &mockVerifyClients{
		btcRPC: &mockBitcoinRPC{},
		eas:    &mockEASClient{},
	}
}

func (m *mockVerifyClients) GetBlockHash(height int64) (string, error) {
	return m.btcRPC.GetBlockHash(height)
}

func (m *mockVerifyClients) GetBlockHeader(hash string) (*rpc.BlockHeader, error) {
	return m.btcRPC.GetBlockHeader(hash)
}

func (m *mockVerifyClients) GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error) {
	return m.eas.GetEASAttestation(ctx, chainID, uid)
}

func (m *mockVerifyClients) GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error) {
	return m.eas.GetTimestamp(ctx, chainID, data)
}

func TestVerify_PendingAttestation(t *testing.T) {
	clients := newMockVerifyClients()
	att := &types.PendingAttestation{URI: "https://example.com/calendar"}

	status := Verify(context.Background(), clients, clients, nil, att)

	if status.Status != types.StatusPending {
		t.Errorf("expected status %v, got %v", types.StatusPending, status.Status)
	}
	if status.Attestation != att {
		t.Error("expected attestation to be set")
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
}

func TestVerify_BitcoinAttestation(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))
	merkleRoot := rpc.ReverseBytes(digest[:])
	merkleRootHex := hex.EncodeToString(merkleRoot)

	clients := newMockVerifyClients()
	clients.btcRPC.blockHash = "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2"
	clients.btcRPC.header = &rpc.BlockHeader{
		Hash:       "000000000000000000002a5c2f3f8c8e7d9a6b5e4f3d2c1b0a9f8e7d6c5b4a3a2",
		MerkleRoot: merkleRootHex,
		Height:     800000,
		Time:       1234567890,
	}

	att := &types.BitcoinAttestation{Height: 800000}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
}

func TestVerify_BitcoinAttestation_Error(t *testing.T) {
	digest := sha256.Sum256([]byte("test data"))

	clients := newMockVerifyClients()
	clients.btcRPC.hashErr = errors.New("block not found")

	att := &types.BitcoinAttestation{Height: 800000}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerify_EASAttestation(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	clients := newMockVerifyClients()
	clients.eas.attestation = &rpc.Attestation{
		UID:       attUID,
		Schema:    rpc.SchemaID,
		Revocable: false,
		Data:      digest[:],
	}

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
}

func TestVerify_EASAttestation_Error(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}
	attUID := [32]byte{0xaa, 0xbb, 0xcc, 0xdd}

	clients := newMockVerifyClients()
	clients.eas.attErr = errors.New("RPC error")

	att := &types.EASAttestation{
		ChainID: 534352,
		UID:     attUID,
	}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerify_EASTimestamped(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}

	clients := newMockVerifyClients()
	clients.eas.timestamp = 1234567890

	att := &types.EASTimestamped{ChainID: 534352}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusValid {
		t.Errorf("expected status %v, got %v", types.StatusValid, status.Status)
	}
	if status.Error != nil {
		t.Errorf("expected no error, got %v", status.Error)
	}
}

func TestVerify_EASTimestamped_Error(t *testing.T) {
	digest := [32]byte{0x01, 0x02, 0x03, 0x04}

	clients := newMockVerifyClients()
	clients.eas.tsErr = errors.New("RPC error")

	att := &types.EASTimestamped{ChainID: 534352}
	status := Verify(context.Background(), clients, clients, digest[:], att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Error == nil {
		t.Error("expected error for RPC failure")
	}
}

func TestVerify_UnknownAttestation(t *testing.T) {
	clients := newMockVerifyClients()
	att := types.NewUnknownAttestation([8]byte{0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99}, []byte{0x01, 0x02})

	status := Verify(context.Background(), clients, clients, []byte{0x01}, att)

	if status.Status != types.StatusUnknown {
		t.Errorf("expected status %v, got %v", types.StatusUnknown, status.Status)
	}
	if status.Attestation != att {
		t.Error("expected attestation to be set")
	}
}
