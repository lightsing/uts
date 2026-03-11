package uts

import (
	"github.com/lightsing/uts/packages/sdk-go/errors"
)

type ErrorCode = errors.ErrorCode

const (
	ErrCodeBadMagic          = errors.ErrCodeBadMagic
	ErrCodeBadVersion        = errors.ErrCodeBadVersion
	ErrCodeBadAttestationTag = errors.ErrCodeBadAttestationTag
	ErrCodeLEB128Overflow    = errors.ErrCodeLEB128Overflow
	ErrCodeBadOpCode         = errors.ErrCodeBadOpCode
	ErrCodeExpectedDigestOp  = errors.ErrCodeExpectedDigestOp
	ErrCodeOutOfRange        = errors.ErrCodeOutOfRange
	ErrCodeInvalidUriChar    = errors.ErrCodeInvalidUriChar
	ErrCodeUriTooLong        = errors.ErrCodeUriTooLong
	ErrCodeRecursionLimit    = errors.ErrCodeRecursionLimit
	ErrCodeUnexpectedEof     = errors.ErrCodeUnexpectedEof
	ErrCodeUsizeOverflow     = errors.ErrCodeUsizeOverflow
	ErrCodeNoValue           = errors.ErrCodeNoValue
	ErrCodePending           = errors.ErrCodePending
	ErrCodeInvalidLength     = errors.ErrCodeInvalidLength
	ErrCodeInvalidData       = errors.ErrCodeInvalidData
	ErrCodeInvalidSchema     = errors.ErrCodeInvalidSchema
	ErrCodeRevocableAttest   = errors.ErrCodeRevocableAttest
	ErrCodeMismatched        = errors.ErrCodeMismatched
	ErrCodeNotFound          = errors.ErrCodeNotFound
	ErrCodeRpc               = errors.ErrCodeRpc
	ErrCodeRemote            = errors.ErrCodeRemote
	ErrCodeDecode            = errors.ErrCodeDecode
)

type SDKError = errors.SDKError
type DecodeError = errors.DecodeError
type EncodeError = errors.EncodeError
type VerifyError = errors.VerifyError
type EASVerifierError = errors.EASVerifierError
type RemoteError = errors.RemoteError

var (
	NewDecodeError             = errors.NewDecodeError
	ErrBadMagic                = errors.ErrBadMagic
	ErrBadVersion              = errors.ErrBadVersion
	ErrBadAttestationTag       = errors.ErrBadAttestationTag
	ErrLEB128Overflow          = errors.ErrLEB128Overflow
	ErrBadOpCode               = errors.ErrBadOpCode
	ErrExpectedDigestOp        = errors.ErrExpectedDigestOp
	ErrOutOfRange              = errors.ErrOutOfRange
	ErrInvalidUriChar          = errors.ErrInvalidUriChar
	ErrUriTooLong              = errors.ErrUriTooLong
	ErrRecursionLimit          = errors.ErrRecursionLimit
	ErrUnexpectedEof           = errors.ErrUnexpectedEof
	NewEncodeError             = errors.NewEncodeError
	ErrUsizeOverflow           = errors.ErrUsizeOverflow
	ErrEncodeInvalidUriChar    = errors.ErrEncodeInvalidUriChar
	ErrEncodeUriTooLong        = errors.ErrEncodeUriTooLong
	NewVerifyError             = errors.NewVerifyError
	ErrNoValue                 = errors.ErrNoValue
	ErrPending                 = errors.ErrPending
	ErrVerifyBadAttestationTag = errors.ErrVerifyBadAttestationTag
	ErrVerifyDecode            = errors.ErrVerifyDecode
	NewEASVerifierError        = errors.NewEASVerifierError
	ErrInvalidLength           = errors.ErrInvalidLength
	ErrInvalidData             = errors.ErrInvalidData
	ErrInvalidSchema           = errors.ErrInvalidSchema
	ErrRevocableAttestation    = errors.ErrRevocableAttestation
	ErrMismatched              = errors.ErrMismatched
	ErrNotFound                = errors.ErrNotFound
	ErrRpc                     = errors.ErrRpc
	NewRemoteError             = errors.NewRemoteError
)
