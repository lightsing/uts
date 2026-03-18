// MIT License
//
// Copyright (c) 2025 UTS Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Apache License, Version 2.0
//
// Copyright (c) 2025 UTS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
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
	"github.com/lightsing/uts/packages/sdk-go/logging"
	"github.com/lightsing/uts/packages/sdk-go/rpc"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

var DefaultCalendars = []string{
	"https://lgm1.calendar.test.timestamps.now/",
	// Run by Peter Todd
	"https://a.pool.opentimestamps.org/",
	"https://b.pool.opentimestamps.org/",
	// Run by Riccardo Casatta
	"https://a.pool.eternitywall.com/",
	// Run by Bull Bitcoin
	"https://ots.btc.catallaxy.com/",
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

const DefaultMaxHttpResponseBytes = 1024 * 1024

type SDK struct {
	calendars            []string
	btcRPC               attestation.BitcoinRPCClient
	ethRPC               *rpc.EthereumClient
	timeout              time.Duration
	quorum               int
	nonceSize            int
	hashAlgorithm        HashAlgorithm
	httpClient           *http.Client
	maxHttpResponseBytes int64
	logger               *logging.Logger
}

type Option func(*SDK) error

func WithCalendars(urls ...string) Option {
	return func(s *SDK) error {
		s.calendars = urls
		return nil
	}
}

func WithBitcoinRPC(client attestation.BitcoinRPCClient) Option {
	return func(s *SDK) error {
		s.btcRPC = client
		return nil
	}
}

func WithEthereumRPC(chainID uint64, rpcURL string) Option {
	return func(s *SDK) error {
		if s.ethRPC == nil {
			s.ethRPC = rpc.NewEthereumClient()
		}
		return s.ethRPC.AddChain(chainID, rpcURL)
	}
}

func WithTimeout(d time.Duration) Option {
	return func(s *SDK) error {
		s.timeout = d
		return nil
	}
}

func WithQuorum(n int) Option {
	return func(s *SDK) error {
		s.quorum = n
		return nil
	}
}

func WithNonceSize(n int) Option {
	return func(s *SDK) error {
		s.nonceSize = n
		return nil
	}
}

func WithHashAlgorithm(alg HashAlgorithm) Option {
	return func(s *SDK) error {
		s.hashAlgorithm = alg
		return nil
	}
}

func WithLogger(logger *logging.Logger) Option {
	return func(s *SDK) error {
		s.logger = logger
		return nil
	}
}

func WithMaxHttpResponseBytes(n int64) Option {
	return func(s *SDK) error {
		s.maxHttpResponseBytes = n
		return nil
	}
}

func NewSDK(opts ...Option) (*SDK, error) {
	calendars := make([]string, len(DefaultCalendars))
	copy(calendars, DefaultCalendars)

	s := &SDK{
		calendars:     calendars,
		timeout:       DefaultTimeout,
		nonceSize:     DefaultNonceSize,
		hashAlgorithm: DefaultHashAlgorithm,
		httpClient: &http.Client{
			Timeout: DefaultTimeout,
		},
		logger:               logging.NewDefaultLogger(logging.LevelInfo),
		maxHttpResponseBytes: DefaultMaxHttpResponseBytes,
	}

	for _, opt := range opts {
		if err := opt(s); err != nil {
			return nil, err
		}
	}

	for i, url := range s.calendars {
		s.calendars[i] = string(bytes.TrimRight([]byte(url), "/"))
	}

	s.httpClient.Timeout = s.timeout

	if s.quorum == 0 {
		s.quorum = int(math.Ceil(float64(len(s.calendars)) * 0.66))
	}

	if s.ethRPC == nil {
		s.ethRPC = rpc.NewEthereumClient()
		for chainID, rpcURL := range rpc.DefaultRpcURLs {
			err := s.ethRPC.AddChain(chainID, rpcURL)
			if err != nil {
				s.logger.Warn(context.Background(), "Failed to add default Ethereum chain", "chain_id", chainID, "url", rpcURL, "error", err)
			}
		}
	}

	s.logger.Debug(context.Background(), "SDK initialized",
		"calendars", len(s.calendars),
		"timeout", s.timeout,
		"quorum", s.quorum,
		"hash_algorithm", s.hashAlgorithm,
	)

	return s, nil
}

func (s *SDK) Calendars() []string {
	calendars := make([]string, len(s.calendars))
	copy(calendars, s.calendars)
	return calendars
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

	reader := io.LimitReader(resp.Body, s.maxHttpResponseBytes+1)
	data, err := io.ReadAll(reader)
	if int64(len(data)) > s.maxHttpResponseBytes {
		return nil, errors.NewRemoteError(fmt.Sprintf("response from %s is too large", calendarURL), nil)
	}
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to read response from %s", calendarURL), err)
	}

	dec := codec.NewDecoder(data)
	return dec.ReadTimestamp()
}

func (s *SDK) Stamp(ctx context.Context, headers []*types.DigestHeader) ([]*types.DetachedTimestamp, error) {
	s.logger.Trace(ctx, "Stamp: enter", "digest_count", len(headers))

	if len(headers) == 0 {
		return nil, errors.NewSDKError(errors.ErrCodeEmptyRequests, "at least one digest header is required", nil)
	}

	s.logger.Debug(ctx, "Stamp: generating nonces", "count", len(headers))
	nonces := make([][]byte, len(headers))
	nonceDigests := make([][crypto.HashSize]byte, len(headers))

	for i, header := range headers {
		nonce := make([]byte, s.nonceSize)
		if _, err := rand.Read(nonce); err != nil {
			s.logger.Error(ctx, "Stamp: failed to generate nonce", "index", i, "error", err)
			return nil, errors.NewSDKError(errors.ErrCodeGeneric, "failed to generate nonce", map[string]interface{}{"error": err.Error()})
		}
		nonces[i] = nonce

		digest := header.DigestBytes()
		var nonceDigest [crypto.HashSize]byte
		switch s.hashAlgorithm {
		case HashSHA256:
			data := make([]byte, len(digest)+len(nonce))
			copy(data, digest)
			copy(data[len(digest):], nonce)
			nonceDigest = crypto.SHA256(data)
		case HashKeccak256:
			data := make([]byte, len(digest)+len(nonce))
			copy(data, digest)
			copy(data[len(digest):], nonce)
			nonceDigest = crypto.Keccak256(data)
		default:
			return nil, errors.NewSDKError(errors.ErrCodeUnsupported, fmt.Sprintf("unsupported hash algorithm: %s", s.hashAlgorithm), nil)
		}
		nonceDigests[i] = nonceDigest
	}

	s.logger.Debug(ctx, "Stamp: building merkle tree")
	tree, err := crypto.NewMerkleTree(nonceDigests)
	if err != nil {
		s.logger.Error(ctx, "Stamp: failed to build merkle tree", "error", err)
		return nil, errors.NewSDKError(errors.ErrCodeGeneric, "failed to build merkle tree", map[string]interface{}{"error": err.Error()})
	}
	root := tree.Root()
	s.logger.Trace(ctx, "Stamp: merkle tree built", "root", hex.EncodeToString(root[:]))

	ctx, cancel := context.WithTimeout(ctx, s.timeout)
	defer cancel()

	s.logger.Debug(ctx, "Stamp: submitting to calendars", "count", len(s.calendars), "quorum", s.quorum)
	respChan := make(chan calendarResponse, len(s.calendars))
	var wg sync.WaitGroup

	for _, calURL := range s.calendars {
		wg.Add(1)
		go func(url string) {
			defer wg.Done()
			s.logger.Trace(ctx, "Stamp: requesting attestation", "calendar", url)
			ts, err := s.requestAttestation(ctx, url, root[:])
			if err != nil {
				s.logger.Debug(ctx, "Stamp: calendar error", "calendar", url, "error", err)
			} else {
				s.logger.Debug(ctx, "Stamp: calendar success", "calendar", url)
			}
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

	s.logger.Debug(ctx, "Stamp: responses received", "success", len(successfulResponses), "quorum", s.quorum)
	if len(successfulResponses) < s.quorum {
		s.logger.Warn(ctx, "Stamp: quorum not reached", "success", len(successfulResponses), "quorum", s.quorum)
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
		s.logger.Trace(ctx, "Stamp: merging multiple responses into fork", "count", len(successfulResponses))
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
			return nil, errors.NewSDKError(errors.ErrCodeGeneric, "failed to generate proof for digest", map[string]interface{}{"digest": header.DigestBytes(), "error": err.Error()})
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
		data := make([]byte, len(input)+len(st.Data))
		copy(data, input)
		copy(data[len(input):], st.Data)
		return data, nil
	case *types.PrependStep:
		data := make([]byte, len(input)+len(st.Data))
		copy(data, st.Data)
		copy(data[len(st.Data):], input)
		return data, nil
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
		return nil, errors.NewSDKError(errors.ErrCodeUnsupported, "SHA1 not supported", nil)
	case *types.RIPEMD160Step:
		return nil, errors.NewSDKError(errors.ErrCodeUnsupported, "RIPEMD160 not supported", nil)
	default:
		return nil, errors.NewSDKError(errors.ErrCodeUnsupported, fmt.Sprintf("unsupported step type: %T", step), nil)
	}
}

func (s *SDK) verifyTimestamp(ctx context.Context, input []byte, ts types.Timestamp) ([]*types.AttestationStatus, error) {
	s.logger.Trace(ctx, "verifyTimestamp: enter", "input_len", len(input), "steps", len(ts))
	var attestations []*types.AttestationStatus
	current := input

	for i, step := range ts {
		s.logger.Trace(ctx, "verifyTimestamp: processing step", "index", i, "step_type", fmt.Sprintf("%T", step))
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
			s.logger.Trace(ctx, "verifyTimestamp: entering fork", "branches", len(st.Branches))
			for _, branch := range st.Branches {
				results, err := s.verifyTimestamp(ctx, current, branch)
				if err != nil {
					return nil, err
				}
				attestations = append(attestations, results...)
			}
		case *types.AttestationStep:
			s.logger.Debug(ctx, "verifyTimestamp: verifying attestation", "type", fmt.Sprintf("%T", st.Attestation))
			status := attestation.Verify(ctx, s.btcRPC, s.ethRPC, current, st.Attestation)
			s.logger.Debug(ctx, "verifyTimestamp: attestation result", "status", status.Status)
			attestations = append(attestations, status)
		default:
			return nil, fmt.Errorf("unsupported step type: %T", step)
		}
	}

	s.logger.Trace(ctx, "verifyTimestamp: complete", "attestations", len(attestations))
	return attestations, nil
}

func (s *SDK) Verify(ctx context.Context, stamp *types.DetachedTimestamp) (*types.VerificationResult, error) {
	s.logger.Trace(ctx, "Verify: enter", "digest_len", len(stamp.Header.DigestBytes()))
	input := stamp.Header.DigestBytes()

	attestations, err := s.verifyTimestamp(ctx, input, stamp.Timestamp)
	if err != nil {
		s.logger.Warn(ctx, "Verify: failed", "error", err)
		return nil, err
	}

	status := s.aggregateResult(attestations)
	s.logger.Debug(ctx, "Verify: complete", "status", status, "attestations", len(attestations))
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
	s.logger.Trace(ctx, "upgradeAttestation: fetching", "url", url)

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
		s.logger.Debug(ctx, "upgradeAttestation: not found", "uri", att.URI)
		return nil, nil
	}

	if resp.StatusCode != http.StatusOK {
		return nil, errors.NewRemoteError(
			fmt.Sprintf("calendar %s responded with status %d", att.URI, resp.StatusCode),
			nil,
		)
	}

	reader := io.LimitReader(resp.Body, s.maxHttpResponseBytes+1)
	data, err := io.ReadAll(reader)
	if int64(len(data)) > s.maxHttpResponseBytes {
		return nil, errors.NewRemoteError(fmt.Sprintf("response from %s is too large", att.URI), nil)
	}
	if err != nil {
		return nil, errors.NewRemoteError(fmt.Sprintf("failed to read response from %s", att.URI), err)
	}

	s.logger.Debug(ctx, "upgradeAttestation: received timestamp", "uri", att.URI, "bytes", len(data))
	dec := codec.NewDecoder(data)
	return dec.ReadTimestamp()
}

func (s *SDK) upgradeTimestamp(ctx context.Context, input []byte, ts types.Timestamp, keepPending bool) ([]*types.UpgradeResult, error) {
	s.logger.Trace(ctx, "upgradeTimestamp: enter", "input_len", len(input), "steps", len(ts), "keep_pending", keepPending)
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
			s.logger.Trace(ctx, "upgradeTimestamp: entering fork", "branches", len(st.Branches))
			for _, branch := range st.Branches {
				branchResults, err := s.upgradeTimestamp(ctx, current, branch, keepPending)
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

			s.logger.Debug(ctx, "upgradeTimestamp: upgrading pending attestation", "uri", pendingAtt.URI)
			upgraded, err := s.upgradeAttestation(ctx, current, pendingAtt)
			if err != nil {
				s.logger.Warn(ctx, "upgradeTimestamp: upgrade failed", "uri", pendingAtt.URI, "error", err)
				results = append(results, &types.UpgradeResult{
					Status: types.UpgradeFailed,
					Error:  err,
				})
				continue
			}

			if upgraded == nil {
				s.logger.Debug(ctx, "upgradeTimestamp: still pending", "uri", pendingAtt.URI)
				results = append(results, &types.UpgradeResult{
					Status: types.UpgradePending,
				})
				continue
			}

			s.logger.Debug(ctx, "upgradeTimestamp: upgraded", "uri", pendingAtt.URI, "steps", len(upgraded))
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

	s.logger.Trace(ctx, "upgradeTimestamp: complete", "results", len(results))
	return results, nil
}

func (s *SDK) Upgrade(ctx context.Context, stamp *types.DetachedTimestamp, keepPending bool) ([]*types.UpgradeResult, error) {
	s.logger.Trace(ctx, "Upgrade: enter", "keep_pending", keepPending)
	input := stamp.Header.DigestBytes()
	results, err := s.upgradeTimestamp(ctx, input, stamp.Timestamp, keepPending)
	if err != nil {
		s.logger.Warn(ctx, "Upgrade: failed", "error", err)
		return nil, err
	}
	s.logger.Debug(ctx, "Upgrade: complete", "results", len(results))
	return results, nil
}

// ListPending returns all pending attestation URIs in the given detached timestamp.
func (s *SDK) ListPending(stamp *types.DetachedTimestamp) []string {
	return collectPendingAttestations(stamp.Timestamp)
}

func collectPendingAttestations(ts types.Timestamp) []string {
	var uris []string
	for _, step := range ts {
		switch st := step.(type) {
		case *types.AttestationStep:
			if pending, ok := st.Attestation.(*types.PendingAttestation); ok {
				uris = append(uris, pending.URI)
			}
		case *types.ForkStep:
			for _, branch := range st.Branches {
				uris = append(uris, collectPendingAttestations(branch)...)
			}
		}
	}
	return uris
}

// PurgeResult contains the result of a purge operation.
type PurgeResult struct {
	// Purged contains the URIs of the pending attestations that were removed.
	Purged []string
	// HasRemaining is true if the timestamp still has non-pending attestations.
	HasRemaining bool
}

// PurgePending removes all pending attestations from the given detached timestamp.
// It returns a PurgeResult with the purged URIs and whether any attestations remain.
func (s *SDK) PurgePending(stamp *types.DetachedTimestamp) *PurgeResult {
	pending := s.ListPending(stamp)
	if len(pending) == 0 {
		return &PurgeResult{Purged: nil, HasRemaining: true}
	}

	hasRemaining := purgeTimestamp(&stamp.Timestamp)
	return &PurgeResult{
		Purged:       pending,
		HasRemaining: hasRemaining,
	}
}

// purgeTimestamp recursively removes all pending attestation branches from a timestamp.
// It modifies the timestamp slice in place.
// Returns true if the timestamp still has non-pending content, false if it should be removed.
func purgeTimestamp(ts *types.Timestamp) bool {
	i := 0
	for i < len(*ts) {
		step := (*ts)[i]
		switch st := step.(type) {
		case *types.AttestationStep:
			if _, ok := st.Attestation.(*types.PendingAttestation); ok {
				// Remove pending attestation
				*ts = append((*ts)[:i], (*ts)[i+1:]...)
				continue
			}
			i++
		case *types.ForkStep:
			// Recursively purge each branch, remove empty ones
			j := 0
			for j < len(st.Branches) {
				if !purgeTimestamp(&st.Branches[j]) {
					st.Branches = append(st.Branches[:j], st.Branches[j+1:]...)
				} else {
					j++
				}
			}
			if len(st.Branches) == 0 {
				// All branches removed, remove the FORK step
				*ts = append((*ts)[:i], (*ts)[i+1:]...)
				continue
			} else if len(st.Branches) == 1 {
				// Collapse: replace FORK with its single remaining branch
				remaining := st.Branches[0]
				newTs := make(types.Timestamp, 0, len(*ts)-1+len(remaining))
				newTs = append(newTs, (*ts)[:i]...)
				newTs = append(newTs, remaining...)
				newTs = append(newTs, (*ts)[i+1:]...)
				*ts = newTs
				continue
			}
			i++
		default:
			i++
		}
	}
	return len(*ts) > 0
}
