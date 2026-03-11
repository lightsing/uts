# packages/sdk-py/tests/conftest.py
"""Common fixtures for tests."""

from __future__ import annotations

import pytest


@pytest.fixture
def sample_digest() -> bytes:
    """A sample SHA-256 digest for testing."""
    import hashlib

    return hashlib.sha256(b"test data").digest()


@pytest.fixture
def sample_pending_url() -> str:
    """A sample calendar URL for testing."""
    return "https://calendar.example.com"
