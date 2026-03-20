"""Cross-SDK compatibility tests using Rust SDK test vectors.

These tests verify that the Python SDK produces identical results to the Rust SDK
for encoding, decoding, and cryptographic operations.
"""

from __future__ import annotations

from uts_sdk._codec import Decoder, Encoder
from uts_sdk._types import DigestOp, ForkStep

SMALL_DETACHED_TIMESTAMP = bytes(
    b"\x00\x4f\x70\x65\x6e\x54\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x00\x00\x50\x72\x6f\x6f\x66\x00\xbf\x89\xe2\xe8\x84\xe8\x92"
    b"\x94\x01\x08\xa7\x0d\xfe\x69\xc5\xa0\xd6\x28\x16\x78\x1a\xbb\x6e\x17\x77\x85\x47\x18\x62\x4a\x0d\x19\x42\x31\xad\xb1\x4c"
    b"\x32\xee\x54\x38\xa4\xf0\x10\x7a\x46\x05\xde\x0a\x5b\x37\xcb\x21\x17\x59\xc6\x81\x2b\xfe\x2e\x08\xff\xf0\x10\x24\x4b\x79"
    b"\xd5\x78\xaa\x38\xe3\x4f\x42\x7b\x0f\x3e\xd2\x55\xa5\x08\xf1\x04\x58\xa4\xc2\x57\xf0\x08\xa1\xa9\x2c\x61\xd5\x41\x72\x06"
    b"\x00\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e\x2c\x2b\x68\x74\x74\x70\x73\x3a\x2f\x2f\x62\x6f\x62\x2e\x62\x74\x63\x2e\x63\x61\x6c"
    b"\x65\x6e\x64\x61\x72\x2e\x6f\x70\x65\x6e\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x2e\x6f\x72\x67\xf0\x10\xe0\x27\x85\x91"
    b"\xe2\x88\x68\x19\xba\x7b\x3d\xdd\x63\x2e\xd3\xfe\x08\xf1\x04\x58\xa4\xc2\x56\xf0\x08\x38\xf2\xc7\xf4\xba\xf4\xbc\xd7\x00"
    b"\x83\xdf\xe3\x0d\x2e\xf9\x0c\x8e\x2e\x2d\x68\x74\x74\x70\x73\x3a\x2f\x2f\x61\x6c\x69\x63\x65\x2e\x62\x74\x63\x2e\x63\x61"
    b"\x6c\x65\x6e\x64\x61\x72\x2e\x6f\x70\x65\x6e\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x73\x2e\x6f\x72\x67"
)


class TestCrossSDKDecoding:
    """Test decoding Rust SDK encoded timestamps."""

    def test_decode_small_detached_timestamp(self) -> None:
        """Decode a small detached timestamp from Rust SDK fixture."""
        decoded = Decoder.decode_detached(SMALL_DETACHED_TIMESTAMP, strict=True)

        assert decoded.header.kind == DigestOp.SHA256
        assert len(decoded.header.digest) == 32

        assert len(decoded.timestamp) == 3
        assert isinstance(decoded.timestamp[2], ForkStep)
        fork = decoded.timestamp[2]
        assert len(fork.steps) == 2

    def test_roundtrip_small_detached_timestamp(self) -> None:
        """Encode and decode round-trip matches original."""
        decoded = Decoder.decode_detached(SMALL_DETACHED_TIMESTAMP, strict=True)

        encoded = Encoder.encode_detached(decoded)

        assert encoded == SMALL_DETACHED_TIMESTAMP

    def test_decode_multiple_pending_attestations(self) -> None:
        """Timestamp has multiple pending attestations from different calendars."""
        from uts_sdk._types import AttestationStep, PendingAttestation

        decoded = Decoder.decode_detached(SMALL_DETACHED_TIMESTAMP, strict=True)

        fork = decoded.timestamp[2]
        assert isinstance(fork, ForkStep)

        urls = []
        for branch in fork.steps:
            for step in branch:
                if isinstance(step, AttestationStep):
                    attestation = step.attestation
                    if isinstance(attestation, PendingAttestation):
                        urls.append(attestation.url)

        assert len(urls) == 2
        assert any("bob.btc.calendar" in u for u in urls)
        assert any("alice.btc.calendar" in u for u in urls)
