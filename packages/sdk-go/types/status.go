package types

import "fmt"

type AttestationStatusKind int

const (
	StatusValid AttestationStatusKind = iota
	StatusInvalid
	StatusPending
	StatusUnknown
)

func (k AttestationStatusKind) String() string {
	switch k {
	case StatusValid:
		return "valid"
	case StatusInvalid:
		return "invalid"
	case StatusPending:
		return "pending"
	case StatusUnknown:
		return "unknown"
	default:
		return "unknown"
	}
}

type AttestationStatus struct {
	Attestation Attestation
	Status      AttestationStatusKind
	Error       error
	Info        map[string]interface{}
}

func NewAttestationStatus(att Attestation, status AttestationStatusKind, err error) *AttestationStatus {
	return &AttestationStatus{
		Attestation: att,
		Status:      status,
		Error:       err,
	}
}

func (s *AttestationStatus) String() string {
	if s.Error != nil {
		return fmt.Sprintf("%s: %s (%s)", s.Attestation, s.Status, s.Error)
	}
	return fmt.Sprintf("%s: %s", s.Attestation, s.Status)
}

type VerifyStatus int

const (
	VerifyValid VerifyStatus = iota
	VerifyPartialValid
	VerifyInvalid
	VerifyPending
)

func (s VerifyStatus) String() string {
	switch s {
	case VerifyValid:
		return "valid"
	case VerifyPartialValid:
		return "partial_valid"
	case VerifyInvalid:
		return "invalid"
	case VerifyPending:
		return "pending"
	default:
		return "unknown"
	}
}

type UpgradeStatus int

const (
	UpgradeUpgraded UpgradeStatus = iota
	UpgradePending
	UpgradeFailed
)

func (s UpgradeStatus) String() string {
	switch s {
	case UpgradeUpgraded:
		return "upgraded"
	case UpgradePending:
		return "pending"
	case UpgradeFailed:
		return "failed"
	default:
		return "unknown"
	}
}

type UpgradeResult struct {
	Status       UpgradeStatus
	Timestamp    *DetachedTimestamp
	Error        error
	Attestations []Attestation
}

func NewUpgradeResult(status UpgradeStatus, ts *DetachedTimestamp, err error) *UpgradeResult {
	return &UpgradeResult{
		Status:    status,
		Timestamp: ts,
		Error:     err,
	}
}

type VerificationResult struct {
	Status       VerifyStatus
	Attestations []*AttestationStatus
	Error        error
}

func NewVerificationResult(status VerifyStatus, atts []*AttestationStatus) *VerificationResult {
	return &VerificationResult{
		Status:       status,
		Attestations: atts,
	}
}

func (r *VerificationResult) HasValidAttestation() bool {
	for _, att := range r.Attestations {
		if att.Status == StatusValid {
			return true
		}
	}
	return false
}

func (r *VerificationResult) String() string {
	if r.Error != nil {
		return fmt.Sprintf("%s (error: %s)", r.Status, r.Error)
	}
	return fmt.Sprintf("%s (%d attestations)", r.Status, len(r.Attestations))
}
