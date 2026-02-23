export enum ErrorCode {
  GENERAL_ERROR = 'GENERAL_ERROR',

  UNKNOWN_OP = 'UNKNOWN_OP',
  INVALID_FORK = 'INVALID_FORK',

  NEGATIVE_LEB128_INPUT = 'NEGATIVE_LEB128_INPUT',
  EXCEEDS_MAX_U32 = 'EXCEEDS_MAX_U32',

  INVALID_URI = 'INVALID_URI',

  LENGTH_MISMATCH = 'LENGTH_MISMATCH',
}

export class UTSError extends Error {
  public readonly code: ErrorCode

  // Optional: offset in the input data where the error occurred
  public readonly offset?: number
  // Optional fields for additional context
  public readonly context?: Record<string, any>

  constructor(
    code: ErrorCode,
    message: string,
    options?: { offset?: number; context?: Record<string, any> },
  ) {
    super(message)
    this.name = 'UTSError'
    this.code = code
    this.offset = options?.offset
    this.context = options?.context

    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, UTSError)
    }
  }
}

export class EncodeError extends UTSError {
  constructor(
    code: ErrorCode,
    message: string,
    options?: { offset?: number; context?: Record<string, any> },
  ) {
    super(code, `[Encode] ${message}`, options)
    this.name = 'EncodeError'
  }
}

export class DecodeError extends UTSError {
  constructor(
    code: ErrorCode,
    message: string,
    options?: { offset?: number; context?: Record<string, any> },
  ) {
    super(code, `[Decode] ${message}`, options)
    this.name = 'DecodeError'
  }
}
