package codec

import (
	"io"

	"github.com/lightsing/uts/packages/sdk-go/errors"
)

func EncodeU32(n uint32) []byte {
	buf := make([]byte, 0, 5)
	for {
		b := byte(n & 0x7f)
		n >>= 7
		if n != 0 {
			b |= 0x80
		}
		buf = append(buf, b)
		if n == 0 {
			break
		}
	}
	return buf
}

func DecodeU32(r io.Reader) (uint32, error) {
	var result uint32
	var shift uint

	for {
		var b [1]byte
		if _, err := io.ReadFull(r, b[:]); err != nil {
			return 0, err
		}

		val := uint32(b[0] & 0x7f)
		if shift >= 28 {
			if shift == 28 && val > 0x0f {
				return 0, errors.ErrLEB128Overflow(32)
			}
			if shift > 28 {
				return 0, errors.ErrLEB128Overflow(32)
			}
		}

		result |= val << shift

		if b[0]&0x80 == 0 {
			break
		}

		shift += 7
	}

	return result, nil
}

func EncodeU64(n uint64) []byte {
	buf := make([]byte, 0, 10)
	for {
		b := byte(n & 0x7f)
		n >>= 7
		if n != 0 {
			b |= 0x80
		}
		buf = append(buf, b)
		if n == 0 {
			break
		}
	}
	return buf
}

func DecodeU64(r io.Reader) (uint64, error) {
	var result uint64
	var shift uint

	for {
		var b [1]byte
		if _, err := io.ReadFull(r, b[:]); err != nil {
			return 0, err
		}

		val := uint64(b[0] & 0x7f)
		if shift >= 63 {
			if shift == 63 && val > 0x01 {
				return 0, errors.ErrLEB128Overflow(64)
			}
			if shift > 63 {
				return 0, errors.ErrLEB128Overflow(64)
			}
		}

		result |= val << shift

		if b[0]&0x80 == 0 {
			break
		}

		shift += 7
	}

	return result, nil
}
