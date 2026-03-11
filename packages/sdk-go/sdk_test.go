package uts

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/hex"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/lightsing/uts/packages/sdk-go/codec"
	"github.com/lightsing/uts/packages/sdk-go/crypto"
	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

type mockHTTPClient struct {
	doFunc func(req *http.Request) (*http.Response, error)
}

func (m *mockHTTPClient) Do(req *http.Request) (*http.Response, error) {
	return m.doFunc(req)
}

type mockBitcoinRPC struct {
	getBlockHashFunc   func(height int64) (string, error)
	getBlockHeaderFunc func(hash string) (*rpc.BlockHeader, error)
}

func (m *mockBitcoinRPC) GetBlockHash(height int64) (string, error) {
	return m.getBlockHashFunc(height)
}

func (m *mockBitcoinRPC) GetBlockHeader(hash string) (*rpc.BlockHeader, error) {
	return m.getBlockHeaderFunc(hash)
}

type mockEASClient struct {
	getAttestationFunc func(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error)
	getTimestampFunc   func(ctx context.Context, chainID uint64, data [32]byte) (uint64, error)
}

func (m *mockEASClient) GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*rpc.Attestation, error) {
	return m.getAttestationFunc(ctx, chainID, uid)
}

func (m *mockEASClient) GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error) {
	return m.getTimestampFunc(ctx, chainID, data)
}

func TestNewSDK(t *testing.T) {
	tests := []struct {
		name          string
		opts          []Option
		wantCalendars int
		wantTimeout   time.Duration
		wantQuorum    int
		wantNonceSize int
		wantHashAlg   HashAlgorithm
	}{
		{
			name:          "default values",
			opts:          nil,
			wantCalendars: 1,
			wantTimeout:   DefaultTimeout,
			wantQuorum:    1,
			wantNonceSize: DefaultNonceSize,
			wantHashAlg:   DefaultHashAlgorithm,
		},
		{
			name: "custom calendars",
			opts: []Option{
				WithCalendars("https://cal1.example.com", "https://cal2.example.com", "https://cal3.example.com"),
			},
			wantCalendars: 3,
			wantTimeout:   DefaultTimeout,
			wantQuorum:    2,
			wantNonceSize: DefaultNonceSize,
			wantHashAlg:   DefaultHashAlgorithm,
		},
		{
			name: "custom timeout",
			opts: []Option{
				WithTimeout(30 * time.Second),
			},
			wantCalendars: 1,
			wantTimeout:   30 * time.Second,
			wantQuorum:    1,
			wantNonceSize: DefaultNonceSize,
			wantHashAlg:   DefaultHashAlgorithm,
		},
		{
			name: "custom quorum",
			opts: []Option{
				WithCalendars("https://cal1.example.com", "https://cal2.example.com"),
				WithQuorum(2),
			},
			wantCalendars: 2,
			wantTimeout:   DefaultTimeout,
			wantQuorum:    2,
			wantNonceSize: DefaultNonceSize,
			wantHashAlg:   DefaultHashAlgorithm,
		},
		{
			name: "custom nonce size",
			opts: []Option{
				WithNonceSize(16),
			},
			wantCalendars: 1,
			wantTimeout:   DefaultTimeout,
			wantQuorum:    1,
			wantNonceSize: 16,
			wantHashAlg:   DefaultHashAlgorithm,
		},
		{
			name: "custom hash algorithm SHA256",
			opts: []Option{
				WithHashAlgorithm(HashSHA256),
			},
			wantCalendars: 1,
			wantTimeout:   DefaultTimeout,
			wantQuorum:    1,
			wantNonceSize: DefaultNonceSize,
			wantHashAlg:   HashSHA256,
		},
		{
			name: "multiple options",
			opts: []Option{
				WithCalendars("https://cal1.example.com", "https://cal2.example.com"),
				WithTimeout(5 * time.Second),
				WithQuorum(2),
				WithNonceSize(64),
				WithHashAlgorithm(HashSHA256),
			},
			wantCalendars: 2,
			wantTimeout:   5 * time.Second,
			wantQuorum:    2,
			wantNonceSize: 64,
			wantHashAlg:   HashSHA256,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sdk := NewSDK(tt.opts...)

			if got := len(sdk.Calendars()); got != tt.wantCalendars {
				t.Errorf("NewSDK() calendars = %v, want %v", got, tt.wantCalendars)
			}
			if got := sdk.Timeout(); got != tt.wantTimeout {
				t.Errorf("NewSDK() timeout = %v, want %v", got, tt.wantTimeout)
			}
			if got := sdk.Quorum(); got != tt.wantQuorum {
				t.Errorf("NewSDK() quorum = %v, want %v", got, tt.wantQuorum)
			}
			if sdk.nonceSize != tt.wantNonceSize {
				t.Errorf("NewSDK() nonceSize = %v, want %v", sdk.nonceSize, tt.wantNonceSize)
			}
			if sdk.hashAlgorithm != tt.wantHashAlg {
				t.Errorf("NewSDK() hashAlgorithm = %v, want %v", sdk.hashAlgorithm, tt.wantHashAlg)
			}
		})
	}
}

func TestWithBitcoinRPC(t *testing.T) {
	mockBTC := &mockBitcoinRPC{
		getBlockHashFunc: func(height int64) (string, error) {
			return "test-hash", nil
		},
	}

	sdk := NewSDK(WithBitcoinRPC(mockBTC))
	if sdk.btcRPC == nil {
		t.Error("WithBitcoinRPC() did not set btcRPC")
	}

	hash, err := sdk.btcRPC.GetBlockHash(123)
	if err != nil || hash != "test-hash" {
		t.Errorf("WithBitcoinRPC() mock not working correctly")
	}
}

func TestWithEthereumRPC(t *testing.T) {
	sdk := NewSDK()
	if sdk.ethRPC == nil {
		t.Error("NewSDK() should initialize ethRPC")
	}
}

func TestSDK_SetBitcoinRPC(t *testing.T) {
	sdk := NewSDK()
	mockBTC := &mockBitcoinRPC{}

	sdk.SetBitcoinRPC(mockBTC)
	if sdk.btcRPC != mockBTC {
		t.Error("SetBitcoinRPC() did not set btcRPC correctly")
	}
}

func TestSDK_SetEthereumRPC(t *testing.T) {
	sdk := NewSDK()
	mockEth := rpc.NewEthereumClient()

	sdk.SetEthereumRPC(mockEth)
	if sdk.ethRPC != mockEth {
		t.Error("SetEthereumRPC() did not set ethRPC correctly")
	}
}

func TestExecuteStep(t *testing.T) {
	sdk := NewSDK()

	tests := []struct {
		name       string
		input      []byte
		step       types.Step
		wantOutput []byte
		wantErr    bool
		errContain string
	}{
		{
			name:       "AppendStep",
			input:      []byte("hello"),
			step:       types.NewAppendStep([]byte(" world"), nil),
			wantOutput: []byte("hello world"),
		},
		{
			name:       "PrependStep",
			input:      []byte("world"),
			step:       types.NewPrependStep([]byte("hello "), nil),
			wantOutput: []byte("hello world"),
		},
		{
			name:       "ReverseStep",
			input:      []byte("hello"),
			step:       types.NewReverseStep(nil),
			wantOutput: []byte("olleh"),
		},
		{
			name:       "HexlifyStep",
			input:      []byte{0x01, 0x02, 0xff},
			step:       types.NewHexlifyStep(nil),
			wantOutput: []byte("0102ff"),
		},
		{
			name:       "SHA256Step",
			input:      []byte("test"),
			step:       types.NewSHA256Step(nil),
			wantOutput: func() []byte { h := crypto.SHA256([]byte("test")); return h[:] }(),
		},
		{
			name:       "Keccak256Step",
			input:      []byte("test"),
			step:       types.NewKeccak256Step(nil),
			wantOutput: func() []byte { h := crypto.Keccak256([]byte("test")); return h[:] }(),
		},
		{
			name:       "SHA1Step not supported",
			input:      []byte("test"),
			step:       types.NewSHA1Step(nil),
			wantErr:    true,
			errContain: "SHA1 not supported",
		},
		{
			name:       "RIPEMD160Step not supported",
			input:      []byte("test"),
			step:       types.NewRIPEMD160Step(nil),
			wantErr:    true,
			errContain: "RIPEMD160 not supported",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			output, err := sdk.executeStep(tt.input, tt.step)
			if tt.wantErr {
				if err == nil {
					t.Errorf("executeStep() expected error, got nil")
				} else if !strings.Contains(err.Error(), tt.errContain) {
					t.Errorf("executeStep() error = %v, want containing %v", err, tt.errContain)
				}
				return
			}
			if err != nil {
				t.Errorf("executeStep() unexpected error = %v", err)
				return
			}
			if !bytes.Equal(output, tt.wantOutput) {
				t.Errorf("executeStep() = %x, want %x", output, tt.wantOutput)
			}
		})
	}
}

func TestAggregateResult(t *testing.T) {
	sdk := NewSDK()

	tests := []struct {
		name         string
		attestations []*types.AttestationStatus
		wantStatus   types.VerifyStatus
	}{
		{
			name:         "empty attestations",
			attestations: []*types.AttestationStatus{},
			wantStatus:   types.VerifyInvalid,
		},
		{
			name: "single valid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusValid, nil),
			},
			wantStatus: types.VerifyValid,
		},
		{
			name: "single pending",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.PendingAttestation{URI: "https://example.com"}, types.StatusPending, nil),
			},
			wantStatus: types.VerifyPending,
		},
		{
			name: "single invalid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusInvalid, nil),
			},
			wantStatus: types.VerifyInvalid,
		},
		{
			name: "single unknown",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusUnknown, nil),
			},
			wantStatus: types.VerifyInvalid,
		},
		{
			name: "multiple valid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusValid, nil),
				types.NewAttestationStatus(&types.EASAttestation{ChainID: 1}, types.StatusValid, nil),
			},
			wantStatus: types.VerifyValid,
		},
		{
			name: "valid and invalid gives partial valid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusValid, nil),
				types.NewAttestationStatus(&types.EASAttestation{ChainID: 1}, types.StatusInvalid, nil),
			},
			wantStatus: types.VerifyPartialValid,
		},
		{
			name: "valid and unknown gives partial valid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusValid, nil),
				types.NewAttestationStatus(&types.EASAttestation{ChainID: 1}, types.StatusUnknown, nil),
			},
			wantStatus: types.VerifyPartialValid,
		},
		{
			name: "pending takes precedence over invalid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.PendingAttestation{URI: "https://example.com"}, types.StatusPending, nil),
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusInvalid, nil),
			},
			wantStatus: types.VerifyPending,
		},
		{
			name: "only invalid",
			attestations: []*types.AttestationStatus{
				types.NewAttestationStatus(&types.BitcoinAttestation{Height: 100}, types.StatusInvalid, nil),
				types.NewAttestationStatus(&types.EASAttestation{ChainID: 1}, types.StatusInvalid, nil),
			},
			wantStatus: types.VerifyInvalid,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := sdk.aggregateResult(tt.attestations)
			if got != tt.wantStatus {
				t.Errorf("aggregateResult() = %v, want %v", got, tt.wantStatus)
			}
		})
	}
}

func createMockCalendarServer(t *testing.T, responseTimestamp types.Timestamp) *httptest.Server {
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/digest" && r.Method == "POST" {
			enc := codec.NewEncoder()
			if err := enc.WriteTimestamp(responseTimestamp); err != nil {
				t.Errorf("failed to encode timestamp: %v", err)
				http.Error(w, "internal error", http.StatusInternalServerError)
				return
			}
			w.Header().Set("Content-Type", "application/vnd.opentimestamps.v1")
			w.WriteHeader(http.StatusOK)
			w.Write(enc.Bytes())
			return
		}
		http.Error(w, "not found", http.StatusNotFound)
	}))
}

func createMockUpgradeServer(t *testing.T, commitment []byte, responseTimestamp types.Timestamp, statusCode int) *httptest.Server {
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		expectedPath := "/timestamp/" + hex.EncodeToString(commitment)
		if r.URL.Path == expectedPath && r.Method == "GET" {
			if statusCode == http.StatusNotFound {
				w.WriteHeader(http.StatusNotFound)
				return
			}
			if statusCode != http.StatusOK {
				w.WriteHeader(statusCode)
				return
			}
			enc := codec.NewEncoder()
			if err := enc.WriteTimestamp(responseTimestamp); err != nil {
				t.Errorf("failed to encode timestamp: %v", err)
				http.Error(w, "internal error", http.StatusInternalServerError)
				return
			}
			w.Header().Set("Content-Type", "application/vnd.opentimestamps.v1")
			w.WriteHeader(http.StatusOK)
			w.Write(enc.Bytes())
			return
		}
		http.Error(w, "not found", http.StatusNotFound)
	}))
}

func TestStamp(t *testing.T) {
	pendingAtt := &types.PendingAttestation{URI: "https://example.com"}
	ts := types.Timestamp{
		types.NewAttestationStep(pendingAtt),
	}

	server := createMockCalendarServer(t, ts)
	defer server.Close()

	tests := []struct {
		name       string
		opts       []Option
		headers    []*types.DigestHeader
		wantErr    bool
		errContain string
		validateFn func(t *testing.T, results []*types.DetachedTimestamp)
	}{
		{
			name: "single digest",
			opts: []Option{
				WithCalendars(server.URL),
				WithTimeout(5 * time.Second),
			},
			headers: []*types.DigestHeader{
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
			},
			wantErr: false,
			validateFn: func(t *testing.T, results []*types.DetachedTimestamp) {
				if len(results) != 1 {
					t.Errorf("expected 1 result, got %d", len(results))
					return
				}
				if results[0] == nil {
					t.Error("result should not be nil")
					return
				}
				if len(results[0].Timestamp) == 0 {
					t.Error("timestamp should not be empty")
				}
			},
		},
		{
			name: "multiple digests",
			opts: []Option{
				WithCalendars(server.URL),
				WithTimeout(5 * time.Second),
			},
			headers: []*types.DigestHeader{
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
			},
			wantErr: false,
			validateFn: func(t *testing.T, results []*types.DetachedTimestamp) {
				if len(results) != 3 {
					t.Errorf("expected 3 results, got %d", len(results))
					return
				}
				for i, r := range results {
					if r == nil {
						t.Errorf("result %d should not be nil", i)
						continue
					}
					if len(r.Timestamp) == 0 {
						t.Errorf("timestamp %d should not be empty", i)
					}
				}
			},
		},
		{
			name: "with SHA256 hash algorithm",
			opts: []Option{
				WithCalendars(server.URL),
				WithTimeout(5 * time.Second),
				WithHashAlgorithm(HashSHA256),
			},
			headers: []*types.DigestHeader{
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
			},
			wantErr: false,
			validateFn: func(t *testing.T, results []*types.DetachedTimestamp) {
				if len(results) != 1 {
					t.Errorf("expected 1 result, got %d", len(results))
					return
				}
				hasSHA256 := false
				for _, step := range results[0].Timestamp {
					if _, ok := step.(*types.SHA256Step); ok {
						hasSHA256 = true
						break
					}
				}
				if !hasSHA256 {
					t.Error("expected SHA256 step in timestamp")
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sdk := NewSDK(tt.opts...)

			results, err := sdk.Stamp(context.Background(), tt.headers)
			if tt.wantErr {
				if err == nil {
					t.Errorf("Stamp() expected error, got nil")
				}
				return
			}
			if err != nil {
				t.Errorf("Stamp() unexpected error = %v", err)
				return
			}

			if tt.validateFn != nil {
				tt.validateFn(t, results)
			}
		})
	}
}

func TestRequestAttestation_MalformedResponse(t *testing.T) {
	malformedServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/vnd.opentimestamps.v1")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("invalid data"))
	}))
	defer malformedServer.Close()

	sdk := NewSDK(
		WithCalendars(malformedServer.URL),
		WithTimeout(5*time.Second),
	)

	headers := []*types.DigestHeader{
		types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
	}

	_, err := sdk.Stamp(context.Background(), headers)
	if err == nil {
		t.Error("Stamp() expected error for malformed response, got nil")
	}
}

func TestUpgradeAttestation_CommitmentCalculation(t *testing.T) {
	digest := []byte("test digest for upgrade test")

	appendData := []byte{0x01, 0x02, 0x03, 0x04}

	pendingAtt := &types.PendingAttestation{URI: "https://cal.example.com"}

	var requestReceived bool
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		requestReceived = true
		w.WriteHeader(http.StatusNotFound)
	}))
	defer server.Close()

	pendingAtt.URI = server.URL

	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, digest),
		types.Timestamp{
			types.NewAppendStep(appendData, nil),
			types.NewSHA256Step(nil),
			types.NewAttestationStep(pendingAtt),
		},
	)

	sdk := NewSDK(WithTimeout(5 * time.Second))

	_, _ = sdk.Upgrade(context.Background(), stamp, false)

	if !requestReceived {
		t.Error("expected request to be sent to calendar server")
	}
}

func TestUpgrade_ConcurrentCalendars(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping concurrent test in short mode")
	}

	digest := make([]byte, 32)
	for i := range digest {
		digest[i] = byte(i)
	}

	callCount := 0
	upgraded := types.Timestamp{
		types.NewAttestationStep(&types.BitcoinAttestation{Height: 800000}),
	}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		enc := codec.NewEncoder()
		enc.WriteTimestamp(upgraded)
		w.Header().Set("Content-Type", "application/vnd.opentimestamps.v1")
		w.WriteHeader(http.StatusOK)
		w.Write(enc.Bytes())
	}))
	defer server.Close()

	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, digest),
		types.Timestamp{
			types.NewAttestationStep(&types.PendingAttestation{URI: server.URL}),
		},
	)

	sdk := NewSDK(WithTimeout(5 * time.Second))

	for i := 0; i < 10; i++ {
		results, err := sdk.Upgrade(context.Background(), stamp, false)
		if err != nil {
			t.Errorf("Upgrade() iteration %d error = %v", i, err)
			continue
		}
		if len(results) > 0 && results[0].Status == types.UpgradeUpgraded {
			callCount++
		}
	}

	if callCount != 10 {
		t.Errorf("expected 10 successful upgrades, got %d", callCount)
	}
}

func TestStamp_NonceGeneration(t *testing.T) {
	ts := types.Timestamp{
		types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
	}

	server := createMockCalendarServer(t, ts)
	defer server.Close()

	sdk := NewSDK(
		WithCalendars(server.URL),
		WithTimeout(5*time.Second),
		WithNonceSize(16),
	)

	headers := []*types.DigestHeader{
		types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
		types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
	}

	results, err := sdk.Stamp(context.Background(), headers)
	if err != nil {
		t.Fatalf("Stamp() error = %v", err)
	}

	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}

	appendSteps1 := extractAppendSteps(results[0].Timestamp)
	appendSteps2 := extractAppendSteps(results[1].Timestamp)

	if len(appendSteps1) == 0 || len(appendSteps2) == 0 {
		t.Fatal("expected at least one append step for nonce")
	}

	if bytes.Equal(appendSteps1[0], appendSteps2[0]) {
		t.Error("nonces should be different for different digests")
	}
}

func extractAppendSteps(ts types.Timestamp) [][]byte {
	var result [][]byte
	for _, step := range ts {
		if appendStep, ok := step.(*types.AppendStep); ok {
			result = append(result, appendStep.Data)
		}
	}
	return result
}

func TestVerify_WithHexlifyStep(t *testing.T) {
	digest := []byte{0xde, 0xad, 0xbe, 0xef}

	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, digest),
		types.Timestamp{
			types.NewHexlifyStep(nil),
			types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
		},
	)

	sdk := NewSDK()

	result, err := sdk.Verify(context.Background(), stamp)
	if err != nil {
		t.Errorf("Verify() unexpected error = %v", err)
		return
	}

	if result.Status != types.VerifyPending {
		t.Errorf("Verify() status = %v, want %v", result.Status, types.VerifyPending)
	}
}

func TestVerify_WithReverseStep(t *testing.T) {
	digest := []byte("hello")

	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, digest),
		types.Timestamp{
			types.NewReverseStep(nil),
			types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
		},
	)

	sdk := NewSDK()

	result, err := sdk.Verify(context.Background(), stamp)
	if err != nil {
		t.Errorf("Verify() unexpected error = %v", err)
		return
	}

	if result.Status != types.VerifyPending {
		t.Errorf("Verify() status = %v, want %v", result.Status, types.VerifyPending)
	}
}

func TestUpgrade_EmptyTimestamp(t *testing.T) {
	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
		types.Timestamp{},
	)

	sdk := NewSDK()

	results, err := sdk.Upgrade(context.Background(), stamp, false)
	if err != nil {
		t.Errorf("Upgrade() unexpected error = %v", err)
		return
	}

	if len(results) != 0 {
		t.Errorf("Upgrade() expected 0 results for empty timestamp, got %d", len(results))
	}
}

func TestStamp_Keccak256Hash(t *testing.T) {
	ts := types.Timestamp{
		types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
	}

	server := createMockCalendarServer(t, ts)
	defer server.Close()

	sdk := NewSDK(
		WithCalendars(server.URL),
		WithTimeout(5*time.Second),
		WithHashAlgorithm(HashKeccak256),
	)

	headers := []*types.DigestHeader{
		types.NewDigestHeader(types.DigestKECCAK256, make([]byte, 32)),
	}

	results, err := sdk.Stamp(context.Background(), headers)
	if err != nil {
		t.Fatalf("Stamp() error = %v", err)
	}

	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}

	hasKeccak := false
	for _, step := range results[0].Timestamp {
		if _, ok := step.(*types.Keccak256Step); ok {
			hasKeccak = true
			break
		}
	}

	if !hasKeccak {
		t.Error("expected Keccak256 step in timestamp")
	}
}

func TestRequestAttestation_HTTPErrorStatus(t *testing.T) {
	tests := []struct {
		name       string
		statusCode int
		wantErr    bool
	}{
		{"bad request", http.StatusBadRequest, true},
		{"not found", http.StatusNotFound, true},
		{"internal error", http.StatusInternalServerError, true},
		{"service unavailable", http.StatusServiceUnavailable, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(tt.statusCode)
			}))
			defer server.Close()

			sdk := NewSDK(
				WithCalendars(server.URL),
				WithTimeout(5*time.Second),
			)

			headers := []*types.DigestHeader{
				types.NewDigestHeader(types.DigestSHA256, make([]byte, 32)),
			}

			_, err := sdk.Stamp(context.Background(), headers)
			if !tt.wantErr && err != nil {
				t.Errorf("Stamp() unexpected error = %v", err)
			}
			if tt.wantErr && err == nil {
				t.Error("Stamp() expected error, got nil")
			}
		})
	}
}

func TestVerify_NilStamp(t *testing.T) {
	sdk := NewSDK()

	defer func() {
		if r := recover(); r == nil {
			t.Error("Verify() expected panic for nil stamp, got nil")
		}
	}()

	_, _ = sdk.Verify(context.Background(), nil)
}

func TestStamp_LargeDigestCount(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping large digest count test in short mode")
	}

	ts := types.Timestamp{
		types.NewAttestationStep(&types.PendingAttestation{URI: "https://example.com"}),
	}

	server := createMockCalendarServer(t, ts)
	defer server.Close()

	sdk := NewSDK(
		WithCalendars(server.URL),
		WithTimeout(30*time.Second),
	)

	numDigests := 100
	headers := make([]*types.DigestHeader, numDigests)
	for i := range headers {
		digest := make([]byte, 32)
		rand.Read(digest)
		headers[i] = types.NewDigestHeader(types.DigestSHA256, digest)
	}

	results, err := sdk.Stamp(context.Background(), headers)
	if err != nil {
		t.Fatalf("Stamp() error = %v", err)
	}

	if len(results) != numDigests {
		t.Errorf("expected %d results, got %d", numDigests, len(results))
	}
}

func TestUpgrade_MultiplePendingAttestations(t *testing.T) {
	digest := make([]byte, 32)

	upgraded := types.Timestamp{
		types.NewAttestationStep(&types.BitcoinAttestation{Height: 800000}),
	}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		enc := codec.NewEncoder()
		enc.WriteTimestamp(upgraded)
		w.Header().Set("Content-Type", "application/vnd.opentimestamps.v1")
		w.WriteHeader(http.StatusOK)
		w.Write(enc.Bytes())
	}))
	defer server.Close()

	stamp := types.NewDetachedTimestamp(
		types.NewDigestHeader(types.DigestSHA256, digest),
		types.Timestamp{
			types.NewSHA256Step(nil),
			types.NewAttestationStep(&types.PendingAttestation{URI: server.URL}),
			types.NewSHA256Step(nil),
			types.NewAttestationStep(&types.PendingAttestation{URI: server.URL}),
		},
	)

	sdk := NewSDK(WithTimeout(5 * time.Second))

	results, err := sdk.Upgrade(context.Background(), stamp, false)
	if err != nil {
		t.Errorf("Upgrade() unexpected error = %v", err)
		return
	}

	if len(results) != 2 {
		t.Errorf("expected 2 upgrade results, got %d", len(results))
	}

	for i, result := range results {
		if result.Status != types.UpgradeUpgraded {
			t.Errorf("result[%d] status = %v, want %v", i, result.Status, types.UpgradeUpgraded)
		}
	}
}

func readAllWithLimit(r io.Reader, limit int64) ([]byte, error) {
	return io.ReadAll(io.LimitReader(r, limit))
}
