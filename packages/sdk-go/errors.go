package uts

import "fmt"

type ErrorCode string

const (
	ErrCodeBadMagic          ErrorCode = "BadMagic"
	ErrCodeBadVersion        ErrorCode = "BadVersion"
	ErrCodeBadAttestationTag ErrorCode = "BadAttestationTag"
	ErrCodeLEB128Overflow    ErrorCode = "LEB128Overflow"
	ErrCodeBadOpCode         ErrorCode = "BadOpCode"
	ErrCodeExpectedDigestOp  ErrorCode = "ExpectedDigestOp"
	ErrCodeOutOfRange        ErrorCode = "OutOfRange"
	ErrCodeInvalidUriChar    ErrorCode = "InvalidUriChar"
	ErrCodeUriTooLong        ErrorCode = "UriTooLong"
	ErrCodeRecursionLimit    ErrorCode = "RecursionLimit"
	ErrCodeUnexpectedEof     ErrorCode = "UnexpectedEof"
	ErrCodeUsizeOverflow     ErrorCode = "UsizeOverflow"
	ErrCodeNoValue           ErrorCode = "NoValue"
	ErrCodePending           ErrorCode = "Pending"
	ErrCodeInvalidLength     ErrorCode = "InvalidLength"
	ErrCodeInvalidData       ErrorCode = "InvalidData"
	ErrCodeInvalidSchema     ErrorCode = "InvalidSchema"
	ErrCodeRevocableAttest   ErrorCode = "RevocableAttestation"
	ErrCodeMismatched        ErrorCode = "Mismatched"
	ErrCodeNotFound          ErrorCode = "NotFound"
	ErrCodeRpc               ErrorCode = "Rpc"
	ErrCodeRemote            ErrorCode = "Remote"
	ErrCodeDecode            ErrorCode = "Decode"
)

type SDKError struct {
	Code    ErrorCode
	Message string
	Context map[string]interface{}
}

func (e *SDKError) Error() string {
	if e.Context != nil {
		return fmt.Sprintf("%s: %s (%v)", e.Code, e.Message, e.Context)
	}
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

func newSDKError(code ErrorCode, msg string, ctx map[string]interface{}) *SDKError {
	return &SDKError{Code: code, Message: msg, Context: ctx}
}

type DecodeError struct {
	*SDKError
}

func NewDecodeError(code ErrorCode, msg string, ctx map[string]interface{}) *DecodeError {
	return &DecodeError{SDKError: newSDKError(code, msg, ctx)}
}

func ErrBadMagic() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeBadMagic, "bad magic bytes", nil)}
}

func ErrBadVersion() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeBadVersion, "bad version", nil)}
}

func ErrBadAttestationTag() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeBadAttestationTag, "bad attestation tag", nil)}
}

func ErrLEB128Overflow(bits uint32) *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeLEB128Overflow,
		fmt.Sprintf("read a LEB128 value overflows %d bits", bits),
		map[string]interface{}{"bits": bits})}
}

func ErrBadOpCode(code byte) *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeBadOpCode,
		fmt.Sprintf("unrecognized opcode: 0x%02x", code),
		map[string]interface{}{"code": code})}
}

func ErrExpectedDigestOp(op string) *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeExpectedDigestOp,
		fmt.Sprintf("expected digest opcode but got: %s", op),
		map[string]interface{}{"op": op})}
}

func ErrOutOfRange() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeOutOfRange, "read value out of range", nil)}
}

func ErrInvalidUriChar() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeInvalidUriChar, "invalid character in URI", nil)}
}

func ErrUriTooLong() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeUriTooLong, "URI too long", nil)}
}

func ErrRecursionLimit() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeRecursionLimit, "recursion limit reached", nil)}
}

func ErrUnexpectedEof() *DecodeError {
	return &DecodeError{SDKError: newSDKError(ErrCodeUnexpectedEof, "unexpected end of file", nil)}
}

type EncodeError struct {
	*SDKError
}

func NewEncodeError(code ErrorCode, msg string, ctx map[string]interface{}) *EncodeError {
	return &EncodeError{SDKError: newSDKError(code, msg, ctx)}
}

func ErrUsizeOverflow() *EncodeError {
	return &EncodeError{SDKError: newSDKError(ErrCodeUsizeOverflow,
		"tried to encode a usize exceeding u32::MAX", nil)}
}

func ErrEncodeInvalidUriChar() *EncodeError {
	return &EncodeError{SDKError: newSDKError(ErrCodeInvalidUriChar, "invalid character in URI", nil)}
}

func ErrEncodeUriTooLong() *EncodeError {
	return &EncodeError{SDKError: newSDKError(ErrCodeUriTooLong, "URI too long", nil)}
}

type VerifyError struct {
	*SDKError
	Inner error
}

func NewVerifyError(code ErrorCode, msg string, inner error) *VerifyError {
	return &VerifyError{SDKError: newSDKError(code, msg, nil), Inner: inner}
}

func ErrNoValue() *VerifyError {
	return &VerifyError{SDKError: newSDKError(ErrCodeNoValue, "raw attestation lacks a value", nil)}
}

func ErrPending() *VerifyError {
	return &VerifyError{SDKError: newSDKError(ErrCodePending,
		"attestation is still pending and cannot be verified yet", nil)}
}

func ErrVerifyBadAttestationTag() *VerifyError {
	return &VerifyError{SDKError: newSDKError(ErrCodeBadAttestationTag,
		"attestation is not the expected type", nil)}
}

func ErrVerifyDecode(inner *DecodeError) *VerifyError {
	return &VerifyError{
		SDKError: newSDKError(ErrCodeDecode,
			fmt.Sprintf("error decoding attestation: %s", inner.Message), nil),
		Inner: inner,
	}
}

type EASVerifierError struct {
	*SDKError
	Inner error
}

func NewEASVerifierError(code ErrorCode, msg string, inner error) *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(code, msg, nil), Inner: inner}
}

func ErrInvalidLength() *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(ErrCodeInvalidLength,
		"invalid value length for EAS attestation", nil)}
}

func ErrInvalidData(inner error) *EASVerifierError {
	return &EASVerifierError{
		SDKError: newSDKError(ErrCodeInvalidData, "invalid attestation data", nil),
		Inner:    inner,
	}
}

func ErrInvalidSchema() *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(ErrCodeInvalidSchema,
		"unexpected schema used for attestation", nil)}
}

func ErrRevocableAttestation() *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(ErrCodeRevocableAttest,
		"attestation cannot be revocable", nil)}
}

func ErrMismatched(expected, actual [32]byte) *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(ErrCodeMismatched,
		"attested hash is not equal to the expected hash",
		map[string]interface{}{"expected": expected, "actual": actual})}
}

func ErrNotFound() *EASVerifierError {
	return &EASVerifierError{SDKError: newSDKError(ErrCodeNotFound, "not found", nil)}
}

func ErrRpc(inner error) *EASVerifierError {
	return &EASVerifierError{
		SDKError: newSDKError(ErrCodeRpc, "RPC error", nil),
		Inner:    inner,
	}
}

func (e *EASVerifierError) IsFatal() bool {
	return e.Code != ErrCodeRpc
}

func (e *EASVerifierError) ShouldRetry() bool {
	return e.Code == ErrCodeRpc
}

type RemoteError struct {
	*SDKError
	Inner error
}

func NewRemoteError(msg string, inner error) *RemoteError {
	return &RemoteError{
		SDKError: newSDKError(ErrCodeRemote, msg, nil),
		Inner:    inner,
	}
}
