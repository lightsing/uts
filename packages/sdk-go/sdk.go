package uts

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"math"
	"net/http"
	"sync"
	"time"

	"github.com/lightsing/uts/packages/sdk-go/attestation"
	"github.com/lightsing/uts/packages/sdk-go/codec"
	"github.com/lightsing/uts/packages/sdk-go/crypto"
	"github.com/lightsing/uts/packages/sdk-go/errors"
	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

var DefaultCalendars = []string{
	"https://lgm1.test.timestamps.now/",
}

const (
	DefaultTimeout       = 10 * time.Second
	DefaultNonceSize     = 32
	DefaultHashAlgorithm = "keccak256"
)

type HashAlgorithm string

const (
	HashSHA256    HashAlgorithm = "sha256"
	HashKeccak256 HashAlgorithm = "keccak256"
)

type SDK struct {
	calendars     []string
	btcRPC        attestation.BitcoinRPCClient
	ethRPC        *rpc.EthereumClient
	timeout       time.Duration
	quorum        int
	nonceSize     int
	hashAlgorithm HashAlgorithm
	httpClient    *http.Client
}

type Option func(*SDK)

func WithCalendars(urls ...string) Option {
	return func(s *SDK) {
		s.calendars = urls
	}
}

func WithBitcoinRPC(client attestation.BitcoinRPCClient) Option {
	return func(s *SDK) {
		s.btcRPC = client
	}
}

func WithEthereumRPC(chainID uint64, rpcURL string) Option {
	return func(s *SDK) {
		if s.ethRPC == nil {
			s.ethRPC = rpc.NewEthereumClient()
		}
		s.ethRPC.AddChain(chainID, rpcURL)
	}
}

func WithTimeout(d time.Duration) Option {
	return func(s *SDK) {
		s.timeout = d
	}
}

func WithQuorum(n int) Option {
	return func(s *SDK) {
		s.quorum = n
	}
}

func WithNonceSize(n int) Option {
	return func(s *SDK) {
		s.nonceSize = n
	}
}

func WithHashAlgorithm(alg HashAlgorithm) Option {
	return func(s *SDK) {
		s.hashAlgorithm = alg
	}
}

func NewSDK(opts ...Option) *SDK {
	s := &SDK{
		calendars:     DefaultCalendars,
		timeout:       DefaultTimeout,
		nonceSize:     DefaultNonceSize,
		hashAlgorithm: DefaultHashAlgorithm,
		httpClient: &http.Client{
			Timeout: DefaultTimeout,
		},
	}

	for _, opt := range opts {
		opt(s)
	}

	if s.quorum == 0 {
		s.quorum = int(math.Ceil(float64(len(s.calendars)) * 0.66))
	}

	if s.ethRPC == nil {
		s.ethRPC = rpc.NewEthereumClient()
	}

	return s
}

func (s *SDK) Calendars() []string {
	return s.calendars
}

func (s *SDK) Timeout() time.Duration {
	return s.timeout
}

func (s *SDK) Quorum() int {
	return s.quorum
}

func (s *SDK) SetBitcoinRPC(client attestation.BitcoinRPCClient) {
	s.btcRPC = client
}

func (s *SDK) SetEthereumRPC(client *rpc.EthereumClient) {
	s.ethRPC = client
}

type calendarResponse struct {
	timestamp types.Timestamp
	err       error
	url       string
}

func (s *SDK) requestAttestation(ctx context.Context, calendarURL string, root []byte) (types.Timestamp, error) {
	url := calendarURL + "/digest"

	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(root))
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to create request for %s", calendarURL), err)
	}
	req.Header.Set("Accept", "application/vnd.opentimestamps.v1")

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to submit to calendar %s", calendarURL), err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, errors.NewRemoteError(
			fmt.Sprintf("calendar %s responded with status %d", calendarURL, resp.StatusCode),
			nil,
		)
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to read response from %s", calendarURL), err)
	}

	dec := codec.NewDecoder(data)
	return dec.ReadTimestamp()
}

func (s *SDK) Stamp(ctx context.Context, headers []*types.DigestHeader) ([]*types.DetachedTimestamp, error) {
	nonces := make([][]byte, len(headers))
	nonceDigests := make([][crypto.HashSize]byte, len(headers))

	for i, header := range headers {
		nonce := make([]byte, s.nonceSize)
		if _, err := rand.Read(nonce); err != nil {
			return nil, fmt.Errorf("failed to generate nonce: %w", err)
		}
		nonces[i] = nonce

		digest := header.DigestBytes()
		var nonceDigest [crypto.HashSize]byte
		switch s.hashAlgorithm {
		case HashSHA256:
			data := append(digest, nonce...)
			nonceDigest = crypto.SHA256(data)
		case HashKeccak256:
			data := append(digest, nonce...)
			nonceDigest = crypto.Keccak256(data)
		}
		nonceDigests[i] = nonceDigest
	}

	tree := crypto.NewMerkleTree(nonceDigests)
	root := tree.Root()

	ctx, cancel := context.WithTimeout(ctx, s.timeout)
	defer cancel()

	respChan := make(chan calendarResponse, len(s.calendars))
	var wg sync.WaitGroup

	for _, calURL := range s.calendars {
		wg.Add(1)
		go func(url string) {
			defer wg.Done()
			ts, err := s.requestAttestation(ctx, url, root[:])
			respChan <- calendarResponse{timestamp: ts, err: err, url: url}
		}(calURL)
	}

	go func() {
		wg.Wait()
		close(respChan)
	}()

	var successfulResponses []types.Timestamp
	for resp := range respChan {
		if resp.err == nil {
			successfulResponses = append(successfulResponses, resp.timestamp)
		}
	}

	if len(successfulResponses) < s.quorum {
		return nil, errors.NewRemoteError(
			fmt.Sprintf("only received %d valid responses from calendars, which does not meet the quorum of %d",
				len(successfulResponses), s.quorum),
			nil,
		)
	}

	var mergedTimestamp types.Timestamp
	if len(successfulResponses) == 1 {
		mergedTimestamp = successfulResponses[0]
	} else {
		mergedTimestamp = types.Timestamp{
			types.NewForkStep(successfulResponses),
		}
	}

	results := make([]*types.DetachedTimestamp, len(headers))
	for i, header := range headers {
		ts := make(types.Timestamp, 0, 2)

		ts = append(ts, types.NewAppendStep(nonces[i], nil))

		switch s.hashAlgorithm {
		case HashSHA256:
			ts = append(ts, types.NewSHA256Step(nil))
		case HashKeccak256:
			ts = append(ts, types.NewKeccak256Step(nil))
		}

		proof, err := tree.GetProof(nonceDigests[i])
		if err != nil {
			return nil, fmt.Errorf("failed to generate proof for digest %x: %w", header.DigestBytes(), err)
		}

		for _, step := range proof {
			prefix := []byte{crypto.InnerNodePrefix}
			if step.Position == crypto.PositionLeft {
				ts = append(ts,
					types.NewPrependStep(prefix, nil),
					types.NewAppendStep(step.Sibling[:], nil),
				)
				switch s.hashAlgorithm {
				case HashSHA256:
					ts = append(ts, types.NewSHA256Step(nil))
				case HashKeccak256:
					ts = append(ts, types.NewKeccak256Step(nil))
				}
			} else {
				ts = append(ts,
					types.NewPrependStep(step.Sibling[:], nil),
					types.NewPrependStep(prefix, nil),
				)
				switch s.hashAlgorithm {
				case HashSHA256:
					ts = append(ts, types.NewSHA256Step(nil))
				case HashKeccak256:
					ts = append(ts, types.NewKeccak256Step(nil))
				}
			}
		}

		ts = append(ts, mergedTimestamp...)

		results[i] = types.NewDetachedTimestamp(header, ts)
	}

	return results, nil
}

func (s *SDK) executeStep(input []byte, step types.Step) ([]byte, error) {
	switch st := step.(type) {
	case *types.AppendStep:
		return append(input, st.Data...), nil
	case *types.PrependStep:
		return append(st.Data, input...), nil
	case *types.ReverseStep:
		result := make([]byte, len(input))
		for i, b := range input {
			result[len(input)-1-i] = b
		}
		return result, nil
	case *types.HexlifyStep:
		hexStr := hex.EncodeToString(input)
		return []byte(hexStr), nil
	case *types.SHA256Step:
		hash := crypto.SHA256(input)
		return hash[:], nil
	case *types.Keccak256Step:
		hash := crypto.Keccak256(input)
		return hash[:], nil
	case *types.SHA1Step:
		return nil, fmt.Errorf("SHA1 not supported")
	case *types.RIPEMD160Step:
		return nil, fmt.Errorf("RIPEMD160 not supported")
	default:
		return nil, fmt.Errorf("unsupported step type: %T", step)
	}
}

func (s *SDK) verifyTimestamp(ctx context.Context, input []byte, ts types.Timestamp) ([]*types.AttestationStatus, error) {
	var attestations []*types.AttestationStatus
	current := input

	for _, step := range ts {
		switch st := step.(type) {
		case *types.AppendStep:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.PrependStep:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.ReverseStep:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.HexlifyStep:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.SHA256Step:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.Keccak256Step:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}
		case *types.SHA1Step:
			return nil, fmt.Errorf("SHA1 not supported")
		case *types.RIPEMD160Step:
			return nil, fmt.Errorf("RIPEMD160 not supported")
		case *types.ForkStep:
			for _, branch := range st.Branches {
				results, err := s.verifyTimestamp(ctx, input, branch)
				if err != nil {
					return nil, err
				}
				attestations = append(attestations, results...)
			}
		case *types.AttestationStep:
			status := attestation.Verify(ctx, s.btcRPC, s.ethRPC, current, st.Attestation)
			attestations = append(attestations, status)
		default:
			return nil, fmt.Errorf("unsupported step type: %T", step)
		}
	}

	return attestations, nil
}

func (s *SDK) Verify(ctx context.Context, stamp *types.DetachedTimestamp) (*types.VerificationResult, error) {
	input := stamp.Header.DigestBytes()

	attestations, err := s.verifyTimestamp(ctx, input, stamp.Timestamp)
	if err != nil {
		return nil, err
	}

	status := s.aggregateResult(attestations)
	return types.NewVerificationResult(status, attestations), nil
}

func (s *SDK) aggregateResult(attestations []*types.AttestationStatus) types.VerifyStatus {
	counts := map[types.AttestationStatusKind]int{
		types.StatusValid:   0,
		types.StatusInvalid: 0,
		types.StatusPending: 0,
		types.StatusUnknown: 0,
	}

	for _, att := range attestations {
		counts[att.Status]++
	}

	if counts[types.StatusValid] > 0 {
		if counts[types.StatusInvalid] > 0 || counts[types.StatusUnknown] > 0 {
			return types.VerifyPartialValid
		}
		return types.VerifyValid
	}

	if counts[types.StatusPending] > 0 {
		return types.VerifyPending
	}

	return types.VerifyInvalid
}

func (s *SDK) upgradeAttestation(ctx context.Context, commitment []byte, att *types.PendingAttestation) (types.Timestamp, error) {
	url := att.URI + "/timestamp/" + hex.EncodeToString(commitment)

	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to create request for %s", att.URI), err)
	}
	req.Header.Set("Accept", "application/vnd.opentimestamps.v1")

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to fetch from calendar %s", att.URI), err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		return nil, nil
	}

	if resp.StatusCode != http.StatusOK {
		return nil, errors.NewRemoteError(
			fmt.Sprintf("calendar %s responded with status %d", att.URI, resp.StatusCode),
			nil,
		)
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to read response from %s", att.URI), err)
	}

	dec := codec.NewDecoder(data)
	return dec.ReadTimestamp()
}

func (s *SDK) upgradeTimestamp(ctx context.Context, input []byte, ts types.Timestamp, keepPending bool) ([]*types.UpgradeResult, error) {
	current := input
	var results []*types.UpgradeResult

	for i := 0; i < len(ts); i++ {
		step := ts[i]

		switch st := step.(type) {
		case *types.AppendStep, *types.PrependStep, *types.ReverseStep, *types.HexlifyStep,
			*types.SHA256Step, *types.Keccak256Step, *types.SHA1Step, *types.RIPEMD160Step:
			var err error
			current, err = s.executeStep(current, step)
			if err != nil {
				return nil, err
			}

		case *types.ForkStep:
			for _, branch := range st.Branches {
				branchResults, err := s.upgradeTimestamp(ctx, input, branch, keepPending)
				if err != nil {
					return nil, err
				}
				results = append(results, branchResults...)
			}

		case *types.AttestationStep:
			pendingAtt, ok := st.Attestation.(*types.PendingAttestation)
			if !ok {
				continue
			}

			upgraded, err := s.upgradeAttestation(ctx, current, pendingAtt)
			if err != nil {
				results = append(results, &types.UpgradeResult{
					Status: types.UpgradeFailed,
					Error:  err,
				})
				continue
			}

			if upgraded == nil {
				results = append(results, &types.UpgradeResult{
					Status: types.UpgradePending,
				})
				continue
			}

			if keepPending {
				ts[i] = types.NewForkStep([]types.Timestamp{
					{st},
					upgraded,
				})
			} else {
				newSteps := make(types.Timestamp, 0, len(ts)-1+len(upgraded))
				newSteps = append(newSteps, ts[:i]...)
				newSteps = append(newSteps, upgraded...)
				newSteps = append(newSteps, ts[i+1:]...)
				ts = newSteps
				i += len(upgraded) - 1
			}

			results = append(results, &types.UpgradeResult{
				Status: types.UpgradeUpgraded,
			})
		}
	}

	return results, nil
}

func (s *SDK) Upgrade(ctx context.Context, stamp *types.DetachedTimestamp, keepPending bool) ([]*types.UpgradeResult, error) {
	input := stamp.Header.DigestBytes()
	return s.upgradeTimestamp(ctx, input, stamp.Timestamp, keepPending)
}
